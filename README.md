<h1 align="center">ttychat</h1>

<div align="center">

[![Rust](https://img.shields.io/badge/Rust-1.75+-DEA584?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Ratatui](https://img.shields.io/badge/Ratatui-TUI-CC1F2C?style=for-the-badge)](https://github.com/ratatui/ratatui)
[![Tokio](https://img.shields.io/badge/Tokio-Async-000000?style=for-the-badge&logo=tokio)](https://tokio.rs/)
[![License](https://img.shields.io/badge/License-GPL_3.0-blue.svg?style=for-the-badge)](LICENSE)
<br>
<br>

</div>

ttychat is a lightweight terminal chat client built for [ttychatd](https://github.com/madss-bin/ttychatd).

## Features

- **Persistent Connection Management**: Save and navigate between multiple server profiles with a dual-pane interface.
- **Robust Rendering**: Manual line wrapping and scrolling logic designed for various terminal dimensions and high-density chat logs.
- **Low Overhead**: Compiled binary with minimal runtime dependencies, ensuring low CPU and memory usage.
- **Multi-Platform**: Native support for Linux distros (Arch, Debian, Fedora, etc.) and Windows.

## Installation

### Linux

Run the automated installation script:

```bash
curl -fsSL https://raw.githubusercontent.com/madss-bin/ttychat/main/install.sh | bash
```

The script detects your distribution (Arch, Debian, Ubuntu, Fedora, openSUSE), installs necessary build dependencies, and compiles the project from source. Binary is installed to `/usr/local/bin/ttychat`.

Arch Linux users can also use the provided PKGBUILD in `pkg/arch/`.

### Windows

Run the PowerShell installation script:

```powershell
powershell -ExecutionPolicy ByPass -File install.ps1
```

Installs the binary to `$HOME\AppData\Local\ttychat\bin`.

# From Releases (Pre-built Binaries)

1. Go to the [Releases page](https://github.com/madss-bin/ttychat/releases).

## Usage

Launch the client to reveal the connection screen:

```bash
ttychat
```

## Self-Hosting

The client connects to a ttychat server. To host your own instance, refer to the [ttychatd](https://github.com/madss-bin/ttychatd) repository for server configuration and deployment instructions.

## Requirements

- **Rust 1.75+**: Required for building from source.
- **OpenSSL**: Required for TLS certificate verification.
- **Build Tools**: gcc-libs and standard headers for your platform.

## License

This project is licensed under the GNU General Public License v3.0. See the [LICENSE](LICENSE) file for details.

## Troubleshooting

**Identity Reset**  
If you need to wipe your local identity and configuration, use the "Wipe Local Identity" option in the connection screen settings or delete the `~/.config/ttychat` directory.

### Uninstall

To remove the software and associated files:

```bash
./uninstall.sh
```