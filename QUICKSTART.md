# Quick Start

## 1. Clone And Configure

```bash
git clone https://github.com/dakshdoesdev/Multi-Device-Orchestration.git
cd Multi-Device-Orchestration
cp .env.example .env
```

## 2. Create Virtual Display

Run this once inside a Hyprland session:

```bash
./scripts/setup-hyprland.sh
```

This creates a virtual output named `TABLET-1` and writes local settings to `.env`.

## 3. Start The Server

```bash
./scripts/start-server.sh
```

Or directly:

```bash
cargo run --release
```

## 4. Connect A Tablet

Open the printed URL on another device:

```text
http://YOUR_LAPTOP_IP:8080
```

## Useful Checks

```bash
hyprctl monitors | grep TABLET
curl http://localhost:8080/status
curl -o frame.jpg http://localhost:8080/frame.jpg
```

## Troubleshooting

If no image appears:

- Confirm `grim` is installed.
- Confirm `TABLET_OUTPUT` in `.env` matches the Hyprland output name.
- Confirm both devices are on the same network.
- Allow port `8080` through your firewall if needed.
