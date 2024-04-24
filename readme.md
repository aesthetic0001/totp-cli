# totp

A simple CLI tool to store 2FA tokens and retrieve time based OTPs when needed

### Installation on mac/linux

```bash
curl -s https://raw.githubusercontent.com/aesthetic0001/totp-cli/main/install.sh | bash
```

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