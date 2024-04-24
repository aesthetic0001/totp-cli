# totp

A simple CLI tool to store 2FA tokens and retrieve time based OTPs when needed

### Quick Installation (MacOS/Linux)

```bash
curl -s https://raw.githubusercontent.com/aesthetic0001/totp-cli/main/install.sh | bash
```

### Manual installation
1. Download the latest release from [here](https://github.com/aesthetic0001/totp-cli/releases/latest)
2. Add the binary to your PATH
3. Run `totp` to verify installation

### Usage

```bash
Usage: totp <COMMAND>

Commands:
  add     
  remove  
  list    
  help    Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

### Building

```bash
cargo build --release
```