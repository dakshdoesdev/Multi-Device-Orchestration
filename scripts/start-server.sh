#!/bin/bash
# Start the tablet display server

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

cd "$PROJECT_DIR"

# Get all IP addresses
get_ips() {
    # Get WiFi IP (for tablet connection)
    WIFI_IP=$(ip addr show wlan0 2>/dev/null | grep "inet " | awk '{print $2}' | cut -d'/' -f1 | head -1)
    # Get Ethernet IP
    ETH_IP=$(ip addr show enp2s0 2>/dev/null | grep "inet " | awk '{print $2}' | cut -d'/' -f1 | head -1)
    # Fallback to any non-localhost IP
    ANY_IP=$(ip addr show | grep "inet " | grep -v "127.0.0.1" | awk '{print $2}' | cut -d'/' -f1 | head -1)
    
    # Prefer WiFi for tablet
    TABLET_IP="${WIFI_IP:-${ETH_IP:-$ANY_IP}}"
}

# Load environment
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}  Tablet Display Server${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""

# Check if built
if [ ! -f "target/release/tablet-display" ] && [ ! -f "target/debug/tablet-display" ]; then
    echo -e "${YELLOW}Building server for the first time...${NC}"
    echo "This may take a few minutes."
    echo ""
    cargo build --release
    echo ""
fi

# Load .env if exists
if [ -f ".env" ]; then
    set -a
    source .env
    set +a
    echo -e "${GREEN}✓ Loaded configuration from .env${NC}"
    echo "  Output: ${TABLET_OUTPUT:-TABLET-1}"
    echo "  Resolution: ${TABLET_RESOLUTION:-1920x1080}"
    echo ""
fi

# Get IPs
get_ips

# Check if virtual display exists
if command -v hyprctl &> /dev/null && [ -n "$HYPRLAND_INSTANCE_SIGNATURE" ]; then
    if ! hyprctl monitors 2>/dev/null | grep -q "^Monitor ${TABLET_OUTPUT:-TABLET-1}"; then
        echo -e "${YELLOW}⚠️  Virtual display not found!${NC}"
        echo "Run: ./scripts/setup-hyprland.sh"
        echo ""
    fi
fi

# Check for NVIDIA/NVENC
if command -v nvidia-smi &> /dev/null; then
    echo -e "${GREEN}✓ NVIDIA GPU detected${NC}"
    GPU_INFO=$(nvidia-smi --query-gpu=name --format=csv,noheader | head -1)
    echo "  GPU: ${GPU_INFO}"
    
    # Check for NVENC support
    if ffmpeg -encoders 2>/dev/null | grep -q h264_nvenc; then
        echo -e "${GREEN}✓ NVENC available for hardware encoding${NC}"
        export TABLET_ENCODER=nvenc
    else
        echo -e "${YELLOW}⚠️  NVENC not available, will use software encoding${NC}"
    fi
else
    echo -e "${YELLOW}No NVIDIA GPU detected, using software encoding${NC}"
fi

echo ""
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}Starting server...${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""
echo "  Local:    ${CYAN}http://localhost:8080${NC}"

if [ -n "$WIFI_IP" ]; then
    echo "  WiFi:     ${CYAN}http://${WIFI_IP}:8080${NC} ${GREEN}← Use this for tablet${NC}"
fi
if [ -n "$ETH_IP" ] && [ "$ETH_IP" != "$WIFI_IP" ]; then
    echo "  Ethernet: ${CYAN}http://${ETH_IP}:8080${NC}"
fi

if [ -n "$TABLET_IP" ]; then
    echo ""
    echo -e "  ${GREEN}On your tablet, open:${NC}"
    echo -e "  ${GREEN}http://${TABLET_IP}:8080${NC}"
else
    echo ""
    echo -e "  ${YELLOW}Could not detect network IP${NC}"
    echo -e "  ${YELLOW}Check connection and try again${NC}"
fi

echo ""
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""

# Run with release build if available
if [ -f "target/release/tablet-display" ]; then
    RUST_LOG=info ./target/release/tablet-display
else
    RUST_LOG=info cargo run --release
fi
