use anyhow::Result;
use clap::Parser;
use futures_util::{SinkExt, StreamExt};
use std::collections::VecDeque;
use std::io::IsTerminal;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{broadcast, RwLock};
use tokio_tungstenite::tungstenite::Message;
use tracing::{error, info, warn};

#[derive(Parser)]
#[command(name = "codexion_live")]
#[command(about = "A bridge reading stdin and broadcasting to WebSockets")]
struct Cli {
    #[arg(short, long, default_value = "8080")]
    port: u16,

    #[arg(long, default_value = "127.0.0.1")]
    host: String,

    #[arg(short, long, default_value = "10000")]
    max_history: usize,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    if std::io::stdin().is_terminal() {
        warn!("codexion_live is designed to be used with a pipe. It is waiting for manual input.");
    }

    let addr = format!("{}:{}", cli.host, cli.port);
    let (tx, _rx) = broadcast::channel::<String>(1000);
    let tx_shared = Arc::new(tx);
    let history = Arc::new(RwLock::new(VecDeque::with_capacity(cli.max_history)));

    let listener = TcpListener::bind(&addr).await?;
    info!("Codexion Live Bridge started.");
    info!("WebSocket server listening on ws://{}", addr);

    let tx_clone = tx_shared.clone();
    let history_clone = history.clone();
    let max_hist = cli.max_history;

    let stdin_task = tokio::spawn(async move {
        let stdin = tokio::io::stdin();
        let mut reader = BufReader::new(stdin).lines();

        while let Ok(Some(line)) = reader.next_line().await {
            info!("{}", line);
            
            let mut hist = history_clone.write().await;
            if hist.len() >= max_hist {
                hist.pop_front();
            }
            hist.push_back(line.clone());
            
            let _ = tx_clone.send(line);
        }
        info!("End of stdin stream.");
    });

    loop {
        tokio::select! {
            result = listener.accept() => {
                match result {
                    Ok((stream, _)) => {
                        let tx_conn = tx_shared.clone();
                        let history_conn = history.clone();
                        tokio::spawn(handle_connection(stream, tx_conn, history_conn));
                    }
                    Err(e) => {
                        error!("Error accepting connection: {}", e);
                    }
                }
            }
            _ = tokio::signal::ctrl_c() => {
                info!("Received SIGINT, shutting down...");
                break;
            }
        }
    }

    stdin_task.abort();
    Ok(())
}

async fn handle_connection(
    stream: TcpStream,
    tx: Arc<broadcast::Sender<String>>,
    history: Arc<RwLock<VecDeque<String>>>,
) {
    let ws_stream = match tokio_tungstenite::accept_async(stream).await {
        Ok(ws) => ws,
        Err(e) => {
            error!("WebSocket handshake failed: {}", e);
            return;
        }
    };

    info!("New WebSocket connection established.");
    let (mut sender, mut _receiver) = ws_stream.split();
    let mut rx = tx.subscribe();

    let hist = history.read().await;
    if !hist.is_empty() {
        let bulk = hist.iter().cloned().collect::<Vec<_>>().join("\n");
        if let Err(e) = sender.send(Message::Text(bulk)).await {
            error!("Failed to send history, client disconnected: {}", e);
            return;
        }
    }
    drop(hist);

    tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if let Err(_) = sender.send(Message::Text(msg)).await {
                break;
            }
        }
        info!("WebSocket connection closed.");
    });
}
