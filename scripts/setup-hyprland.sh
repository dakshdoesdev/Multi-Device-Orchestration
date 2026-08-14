#!/bin/bash
# Setup virtual display for Hyprland
# This script creates a virtual output that you can stream to your tablet

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}  Tablet Display - Hyprland Setup${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""

# Check if running under Hyprland
if [ -z "$HYPRLAND_INSTANCE_SIGNATURE" ]; then
    echo -e "${RED}Error: Not running under Hyprland!${NC}"
    echo "Please run this script from a Hyprland session."
    exit 1
fi

# Get IP address (compatible method)
get_ip() {
    # Try WiFi first, then Ethernet
    WIFI_IP=$(ip addr show wlan0 2>/dev/null | grep "inet " | awk '{print $2}' | cut -d'/' -f1 | head -1)
    if [ -n "$WIFI_IP" ]; then
        echo "$WIFI_IP"
        return
    fi
    
    ETH_IP=$(ip addr show enp2s0 2>/dev/null | grep "inet " | awk '{print $2}' | cut -d'/' -f1 | head -1)
    if [ -n "$ETH_IP" ]; then
        echo "$ETH_IP"
        return
    fi
    
    # Fallback
    ip addr show | grep "inet " | grep -v "127.0.0.1" | awk '{print $2}' | cut -d'/' -f1 | head -1
}

# Get tablet resolution from user
echo -e "${YELLOW}What resolution should the virtual display use?${NC}"
echo "Common tablet resolutions:"
echo "  1) 1920x1080 (Full HD)"
echo "  2) 1920x1200 (iPad Pro 12.9 inch)"
echo "  3) 2360x1640 (iPad Pro 11 inch)"
echo "  4) 2732x2048 (iPad Pro 12.9 inch high-res)"
echo "  5) Custom"
echo ""
read -p "Choice [1-5]: " choice

case $choice in
    1) RESOLUTION="1920x1080" ;;
    2) RESOLUTION="1920x1200" ;;
    3) RESOLUTION="2360x1640" ;;
    4) RESOLUTION="2732x2048" ;;
    5) 
        read -p "Enter resolution (e.g., 1920x1080): " RESOLUTION
        ;;
    *) 
        echo -e "${YELLOW}Invalid choice, using 1920x1080${NC}"
        RESOLUTION="1920x1080"
        ;;
esac

# Parse width and height
WIDTH=$(echo $RESOLUTION | cut -d'x' -f1)
HEIGHT=$(echo $RESOLUTION | cut -d'x' -f2)

echo ""
echo -e "${BLUE}Creating virtual display with resolution: ${GREEN}${RESOLUTION}${NC}"

# Create virtual output
OUTPUT_NAME="TABLET-1"
echo -e "${BLUE}Creating headless output: ${GREEN}${OUTPUT_NAME}${NC}"

# Check if output already exists
if hyprctl monitors | grep -q "Monitor ${OUTPUT_NAME}"; then
    echo -e "${YELLOW}Virtual display ${OUTPUT_NAME} already exists${NC}"
    read -p "Remove and recreate? [y/N]: " recreate
    if [[ $recreate =~ ^[Yy]$ ]]; then
        hyprctl output remove "${OUTPUT_NAME}" 2>/dev/null || true
        sleep 0.5
        hyprctl output create headless "${OUTPUT_NAME}"
    fi
else
    hyprctl output create headless "${OUTPUT_NAME}"
fi

# Get current monitor layout to position the virtual display
# Get the first monitor's resolution
MAIN_MONITOR_INFO=$(hyprctl monitors | grep -A5 "Monitor eDP-1\|Monitor DP-\|Monitor HDMI-A")
MAIN_WIDTH=$(hyprctl monitors -j 2>/dev/null | python3 -c "import sys,json; d=json.load(sys.stdin); print(int(d[0]['width']))" 2>/dev/null || echo "1920")
MAIN_HEIGHT=$(hyprctl monitors -j 2>/dev/null | python3 -c "import sys,json; d=json.load(sys.stdin); print(int(d[0]['height']))" 2>/dev/null || echo "1080")

# Fallback if python method fails
if [ -z "$MAIN_WIDTH" ] || [ "$MAIN_WIDTH" = "0" ]; then
    MAIN_WIDTH=$(hyprctl monitors | grep "at " | head -1 | grep -o "[0-9]*x[0-9]*" | head -1 | cut -d'x' -f1)
fi

# Final fallback
if [ -z "$MAIN_WIDTH" ] || [ "$MAIN_WIDTH" = "0" ]; then
    MAIN_WIDTH="1920"
fi
if [ -z "$MAIN_HEIGHT" ] || [ "$MAIN_HEIGHT" = "0" ]; then
    MAIN_HEIGHT="1080"
fi

# Ask user for position
echo ""
echo -e "${YELLOW}Where should the virtual display be positioned?${NC}"
echo "  1) Left of main monitor"
echo "  2) Right of main monitor (default)"
echo ""
read -p "Choice [1-2]: " position_choice

# Calculate position based on choice
case $position_choice in
    1)
        # Position to the left: negative X coordinate
        TABLET_X=$((0 - WIDTH))
        TABLET_Y=0
        echo -e "${BLUE}Positioning virtual display to the LEFT of main monitor${NC}"
        ;;
    2|*)
        # Position to the right: positive X coordinate (main monitor width)
        TABLET_X=$MAIN_WIDTH
        TABLET_Y=0
        echo -e "${BLUE}Positioning virtual display to the RIGHT of main monitor${NC}"
        ;;
esac

echo -e "${BLUE}Main monitor: ${GREEN}${MAIN_WIDTH}x${MAIN_HEIGHT}${NC}"
echo -e "${BLUE}Virtual display position: ${GREEN}${TABLET_X}x${TABLET_Y}${NC}"

# Position the virtual display
hyprctl keyword monitor "${OUTPUT_NAME}, ${RESOLUTION}@60, ${TABLET_X}x${TABLET_Y}, 1"

# Store configuration
echo "TABLET_OUTPUT=${OUTPUT_NAME}" > "${PROJECT_DIR}/.env"
echo "TABLET_RESOLUTION=${RESOLUTION}" >> "${PROJECT_DIR}/.env"
echo "TABLET_WIDTH=${WIDTH}" >> "${PROJECT_DIR}/.env"
echo "TABLET_HEIGHT=${HEIGHT}" >> "${PROJECT_DIR}/.env"
echo "TABLET_POSITION_X=${TABLET_X}" >> "${PROJECT_DIR}/.env"
echo "TABLET_POSITION_Y=${TABLET_Y}" >> "${PROJECT_DIR}/.env"

echo ""
echo -e "${GREEN}✅ Virtual display created successfully!${NC}"
echo ""
echo -e "${BLUE}Virtual Display Info:${NC}"
echo "  Name: ${OUTPUT_NAME}"
echo "  Resolution: ${RESOLUTION}"
if [ "$TABLET_X" -lt 0 ]; then
    echo "  Position: ${TABLET_X}x${TABLET_Y} (left of main display)"
else
    echo "  Position: ${TABLET_X}x${TABLET_Y} (right of main display)"
fi
echo ""

# Get IP for display
IP=$(get_ip)

echo -e "${YELLOW}Next steps:${NC}"
echo "  1. Start the server: cd ${PROJECT_DIR} && ./scripts/start-server.sh"
if [ -n "$IP" ]; then
    echo "  2. On your tablet, open: http://${IP}:8080"
else
    echo "  2. On your tablet, open: http://YOUR_IP:8080"
fi
echo "  3. Drag windows to the virtual display using Super+Shift+Right"
echo ""
echo -e "${BLUE}Useful commands:${NC}"
echo "  Move window to tablet:  hyprctl dispatch movewindowtoright"
echo "  List monitors:          hyprctl monitors"
echo "  Remove virtual display: hyprctl output remove ${OUTPUT_NAME}"
echo ""

# Offer to add to hyprland.conf (commented out by default)
read -p "Add virtual display to hyprland.conf (commented)? [y/N]: " add_config
if [[ $add_config =~ ^[Yy]$ ]]; then
    CONFIG_FILE="$HOME/.config/hypr/monitors.conf"
    
    # Backup
    cp "$CONFIG_FILE" "$CONFIG_FILE.backup.$(date +%s)"
    
    # Add virtual monitor config (commented out - user can uncomment if needed)
    echo "" >> "$CONFIG_FILE"
    echo "# Virtual display for tablet - uncomment to auto-create on startup" >> "$CONFIG_FILE"
    echo "# exec-once = hyprctl output create headless ${OUTPUT_NAME}" >> "$CONFIG_FILE"
    echo "# monitor = ${OUTPUT_NAME}, ${RESOLUTION}@60, ${MAIN_WIDTH}x0, 1" >> "$CONFIG_FILE"
    
    echo -e "${GREEN}✅ Added (commented) to ${CONFIG_FILE}${NC}"
    echo -e "${YELLOW}Note: Uncomment lines in monitors.conf to auto-create on startup${NC}"
fi

echo ""
echo -e "${GREEN}Setup complete!${NC}"
