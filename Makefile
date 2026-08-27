CARGO = cargo
BINARY_NAME = codexion_live
TARGET_DIR = target
DEBUG_BIN = $(TARGET_DIR)/debug/$(BINARY_NAME)
RELEASE_BIN = $(TARGET_DIR)/release/$(BINARY_NAME)

.PHONY: all
all: release

.PHONY: release
release:
	$(CARGO) build --release
	cp $(RELEASE_BIN) ./$(BINARY_NAME)

.PHONY: debug
debug:
	$(CARGO) build
	cp $(DEBUG_BIN) ./$(BINARY_NAME)

.PHONY: cargo-run
cargo-run:
	$(CARGO) run

.PHONY: test
test:
	$(CARGO) test

.PHONY: fmt-check
fmt-check:
	$(CARGO) fmt -- --check

.PHONY: fmt
fmt:
	$(CARGO) fmt

.PHONY: lint
lint:
	$(CARGO) clippy -- -D warnings

.PHONY: clean
clean:
	$(CARGO) clean
	rm -f ./$(BINARY_NAME)

.PHONY: run
run:
	./$(BINARY_NAME)