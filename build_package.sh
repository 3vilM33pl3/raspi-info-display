#!/bin/bash

# Build script for info-display Debian package using cargo-deb

set -e

echo "Building info-display Debian package with cargo-deb..."

# Install cargo-deb if not already installed
if ! cargo deb --version &> /dev/null; then
    echo "Installing cargo-deb..."
    cargo install cargo-deb
fi

# Clean previous builds
cargo clean

# Build the Debian package (this will also build the release binary)
echo "Building Debian package..."
cargo deb

echo "Package built successfully!"
echo "Install with: sudo dpkg -i target/debian/info-display_*.deb"
echo "Start service with: sudo systemctl start info-display.service"
echo "Check status with: sudo systemctl status info-display.service"
