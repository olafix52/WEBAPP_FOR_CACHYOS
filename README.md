# CachyOS Web App Manager

A lightweight, native GTK4/Libadwaita application for creating and managing isolated web applications on Linux (Arch/CachyOS). Inspired by Linux Mint's `webapp-manager`.


## Features

- **Native UI**: Built with Rust, GTK4, and Libadwaita for a modern, consistent GNOME look.
- **Browser Support**: Automatically detects installed browsers:
  - Firefox (LibreWolf, etc.)
  - Chromium
  - Brave
  - Vivaldi
  - Google Chrome
- **Profile Isolation**: 
  - Creates dedicated data directories for Chromium-based apps.
  - Generates separate profiles/instances for Firefox-based apps.
- **Icon Management**:
  - Local file selection.
  - **Automatic favicon download** from the target URL.
- **System Integration**: Generates standard `.desktop` files for seamless integration with your desktop environment (KDE, GNOME, etc.).

## Installation

### Dependencies

You need to have `gtk4`, `libadwaita`, and `rust` installed on your system.

**Arch Linux / CachyOS:**
```bash
sudo pacman -S gtk4 libadwaita rust base-devel
```

**Fedora:**
```bash
sudo dnf install gtk4-devel libadwaita-devel cargo openssl-devel
```

**Ubuntu/Debian:**
```bash
sudo apt install libgtk-4-dev libadwaita-1-dev cargo libssl-dev
```

### Building from Source

1. Clone the repository:
   ```bash
   git clone https://github.com/yourusername/cachyos-webapp-manager.git
   cd cachyos-webapp-manager
   ```

2. Build and run:
   ```bash
   cargo run
   ```

3. Build for release:
   ```bash
   cargo build --release
   ```
   The binary will be located at `target/release/cachyos_web_app_manager`.

## Usage

1. Launch the application.
2. Enter the **Name** of your web app (e.g., "Discord").
3. Enter the **URL** (e.g., "https://discord.com/app").
4. Select an **Icon**:
   - Click "Choose..." to select a local file.
   - Or click **Download** to automatically fetch the favicon from the URL.
5. Select your preferred **Browser** and **Category**.
6. Click **Create Web App**.

The new application will appear in your system's application menu.

## License

MIT License. See [LICENSE](LICENSE) for details.
