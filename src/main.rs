use std::env;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{broadcast, RwLock};
use tokio_tungstenite::tungstenite::Message;
use futures_util::{SinkExt, StreamExt};
use std::io::IsTerminal;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.iter().any(|arg| arg == "--help" || arg == "-h") {
        println!("Usage: ./codexion ARGS | ./codexion_live\n");
        println!("codexion_live is a bridge to a web visualizer.");
        println!("It reads data from standard input (stdin) and broadcasts it to connected WebSocket clients.\n");
        println!("Options:");
        println!("  -h, --help    Show this help message");
        return Ok(());
    }

    if std::io::stdin().is_terminal() {
        eprintln!("⚠️ WARNING: codexion_live is designed to be used with a pipe.");
        eprintln!("Expected usage: ./codexion ARGS | ./codexion_live");
        eprintln!("It is currently waiting for manual input from the terminal.\n");
    }

    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let addr = format!("127.0.0.1:{}", port);
    
    let (tx, _rx) = broadcast::channel::<String>(1000);
    let tx_shared = Arc::new(tx);

    let history = Arc::new(RwLock::new(Vec::new()));

    let listener = TcpListener::bind(&addr).await?;
    println!("Codexion Live Bridge started.");
    println!("WebSocket server listening on ws://{}", addr);
    println!("Reading from stdin...");

    let tx_clone = tx_shared.clone();
    let history_clone = history.clone();
    
    tokio::spawn(async move {
        let stdin = tokio::io::stdin();
        let mut reader = BufReader::new(stdin).lines();
        
        while let Ok(Some(line)) = reader.next_line().await {
            println!("{}", line);
            
            history_clone.write().await.push(line.clone());
            
            let _ = tx_clone.send(line);
        }
        
        println!("End of stdin stream.");
    });

    while let Ok((stream, _)) = listener.accept().await {
        let tx_conn = tx_shared.clone();
        let history_conn = history.clone();
        tokio::spawn(handle_connection(stream, tx_conn, history_conn));
    }

    Ok(())
}

async fn handle_connection(stream: TcpStream, tx: Arc<broadcast::Sender<String>>, history: Arc<RwLock<Vec<String>>>) {
    let ws_stream = match tokio_tungstenite::accept_async(stream).await {
        Ok(ws) => ws,
        Err(e) => {
            eprintln!("Error during the websocket handshake: {}", e);
            return;
        }
    };

    println!("New WebSocket connection established.");
    let (mut sender, mut _receiver) = ws_stream.split();
    
    let mut rx = tx.subscribe();

    let hist = history.read().await;
    if !hist.is_empty() {
        let bulk_message = hist.join("\n");
        if sender.send(Message::Text(bulk_message)).await.is_err() {
            println!("Failed to send history, client disconnected.");
            return;
        }
    }
    drop(hist); 

    tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if sender.send(Message::Text(msg)).await.is_err() {
                break;
            }
        }
        println!("WebSocket connection closed.");
    });
}
