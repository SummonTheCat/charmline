#!/usr/bin/env bash
set -e

# --- Parse Args --- #
BOT_KEY=""
while [[ $# -gt 0 ]]; do
    case "$1" in
        -k|--key)
            BOT_KEY="$2"
            shift 2
            ;;
        *)
            echo "Unknown argument: $1"
            echo "Usage: $0 --key <CHARMLINE_BOT_KEY>"
            exit 1
            ;;
    esac
done

if [ -z "$BOT_KEY" ]; then
    echo "Error: Missing required --key argument (CHARMLINE_BOT_KEY)."
    echo "Usage: $0 --key <CHARMLINE_BOT_KEY>"
    exit 1
fi

# --- Determine paths --- #
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
CARGO_ROOT="$(realpath "$SCRIPT_DIR/..")"

# Verify Cargo.toml exists
if [ ! -f "$CARGO_ROOT/Cargo.toml" ]; then
    echo "Error: Cargo.toml not found in $CARGO_ROOT"
    exit 1
fi

# Get crate name from Cargo.toml
CRATE_NAME=$(grep '^name\s*=' "$CARGO_ROOT/Cargo.toml" | head -1 | cut -d '"' -f2)
if [ -z "$CRATE_NAME" ]; then
    CRATE_NAME=$(basename "$CARGO_ROOT")
fi

INSTALL_DIR="/opt/$CRATE_NAME"
SERVICE_FILE="/etc/systemd/system/$CRATE_NAME.service"
BINARY_PATH="$INSTALL_DIR/$CRATE_NAME"

echo "Building $CRATE_NAME..."
cd "$CARGO_ROOT"
cargo build --release

BUILD_EXE="$CARGO_ROOT/target/release/$CRATE_NAME"
if [ ! -f "$BUILD_EXE" ]; then
    echo "Error: build output not found at $BUILD_EXE"
    exit 1
fi

echo "Installing to $INSTALL_DIR..."
sudo rm -rf "$INSTALL_DIR"
sudo mkdir -p "$INSTALL_DIR"
sudo cp "$BUILD_EXE" "$BINARY_PATH"
sudo chmod +x "$BINARY_PATH"

# Copy static and cfg directories
for dir in static cfg; do
    SRC_DIR="$CARGO_ROOT/$dir"
    DEST_DIR="$INSTALL_DIR/$dir"
    if [ -d "$SRC_DIR" ]; then
        sudo mkdir -p "$DEST_DIR"
        sudo rsync -a --delete "$SRC_DIR"/ "$DEST_DIR"/
        echo "Copied $dir/ → $DEST_DIR"
    else
        echo "No $dir directory found, skipping."
    fi
done

echo "Creating systemd service..."
sudo tee "$SERVICE_FILE" > /dev/null <<EOF
[Unit]
Description=$CRATE_NAME Service
After=network.target

[Service]
Type=simple
ExecStart=$BINARY_PATH
WorkingDirectory=$INSTALL_DIR
Restart=always
RestartSec=5
User=$(whoami)
Environment=RUST_LOG=info
Environment=CHARMLINE_BOT_KEY=$BOT_KEY

[Install]
WantedBy=multi-user.target
EOF

echo "Enabling and starting service..."
sudo systemctl daemon-reload
sudo systemctl enable "$CRATE_NAME.service"
sudo systemctl restart "$CRATE_NAME.service"

echo "Service installed and running."
sudo systemctl status "$CRATE_NAME.service" --no-pager

# Verification: list installed files
echo
echo "Verifying installation contents:"
echo "--------------------------------"
sudo ls -R "$INSTALL_DIR"

echo
echo "Installation complete for $CRATE_NAME."
echo "Binary: $BINARY_PATH"
