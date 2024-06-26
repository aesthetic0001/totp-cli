use std::collections::HashMap;
use std::path::PathBuf;
use std::process::exit;
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};

#[derive(Parser, Debug)]
#[clap(name = "totp", version = "0.1.0", author = "aesthetic0001")]
struct Args {
    #[clap(subcommand)]
    cmd: SubCommand,
}

trait TotpHotp {
    fn get_hotp(&self, counter: u64) -> String;
    fn get_totp(&self) -> String;
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct Key {
    secret: String,
    digits: u8,
    period: u8
}

#[derive(Subcommand, Debug)]
enum SubCommand {
    Add {
        #[clap(long, short = 'k')]
        key: String,
        // no way to specify a default value for a string conditionally based on whether input for key includes otpauth://
        #[clap(long, short = 'n')]
        name: String,
        #[clap(long, short = 's', default_value_t = 6)]
        size: u8,
        #[clap(long, short = 'p', default_value_t = 30)]
        period: u8
    },
    Remove {
        #[clap(long, short = 'n')]
        name: String
    },
    List {},
    Get {
        #[clap(long, short = 'n')]
        name: String
    },
    Import {
        #[clap(long, short = 'f')]
        file: PathBuf
    },
    Rename {
        #[clap(long, short = 'o')]
        old: String,
        #[clap(long, short = 'n')]
        new: String
    }
}

impl TotpHotp for Key {
    fn get_hotp(&self, counter: u64) -> String {
        let secret = base32::decode(base32::Alphabet::RFC4648 { padding: false }, &self.secret).unwrap();
        let mut counter_bytes = [0; 8];
        counter_bytes.copy_from_slice(&counter.to_be_bytes());

        let hmac = hmac_sha1::hmac_sha1(&secret, &counter_bytes);
        let offset = (hmac[hmac.len() - 1] & 0xf) as usize;

        let code = ((u32::from(hmac[offset]) & 0x7f) << 24
            | (u32::from(hmac[offset + 1]) & 0xff) << 16
            | (u32::from(hmac[offset + 2]) & 0xff) << 8
            | (u32::from(hmac[offset + 3]) & 0xff))
            % 10u32.pow(u32::from(self.digits));
        format!("{:01$}", code, self.digits as usize)
    }
    fn get_totp(&self) -> String {
        self.get_hotp((chrono::Utc::now().timestamp() as u64) / self.period as u64)
    }
}

fn parse_otpauth(url: String) -> (String, Key) {
    let uri = url::Url::parse(&url).unwrap();
    let mode = uri.host_str().unwrap();
    assert_eq!(mode, "totp");
    let name = uri.path();
    let secret = match uri.query_pairs().find(|(k, _)| k == "secret") {
        Some((_, v)) => v.to_string(),
        None => {
            println!("No secret found!");
            exit(1)
        }
    };
    let digits = match uri.query_pairs().find(|(k, _)| k == "digits") {
        Some((_, v)) => v.parse::<u8>().unwrap(),
        None => 6u8
    };
    let period = match uri.query_pairs().find(|(k, _)| k == "period") {
        Some((_, v)) => v.parse::<u8>().unwrap(),
        None => 30u8
    };
    let chars = name.chars().collect::<Vec<char>>();
    let name = chars[1..].iter().collect::<String>();
    // make sure that name gets rid of the url encoded characters
    let name = urlencoding::decode(&name).unwrap();
    (name.parse().unwrap(), Key { secret, digits, period })
}

fn main() {
    let install_dir = dirs::home_dir().unwrap().join(".totp");
    if !install_dir.exists() {
        println!("Creating directory {:?}!", install_dir);
        std::fs::create_dir_all(&install_dir).unwrap();
    }
    let args = Args::parse();
    let save_path = install_dir.join("2fa.json");
    let saved = std::fs::read_to_string(&save_path).unwrap_or_default();
    let mut accounts: HashMap<String, Key> = serde_json::from_str(&saved).unwrap_or_default();

    match args.cmd {
        SubCommand::Add { key, name, size, period } => {
            if accounts.contains_key(&name) {
                eprintln!("TOTP for {} already exists!", name);
                return;
            }
            if key.starts_with("otpauth://") {
                let (name, key) = parse_otpauth(key);
                if accounts.contains_key(&name) {
                    eprintln!("TOTP for {} already exists!", name);
                    return;
                }
                accounts.insert(name, key);
            } else {
                accounts.insert(name, Key { secret: key, digits: size, period });
            }
            println!("TOTP added!");
            let json = serde_json::to_string(&accounts).unwrap();
            std::fs::write(&save_path, json).unwrap();
        },
        SubCommand::Remove { name } => {
            if accounts.remove(&name).is_none() {
                eprintln!("TOTP for {} does not exist!", name);
                return;
            }
            let json = serde_json::to_string(&accounts).unwrap();
            std::fs::write(&save_path, json).unwrap();
            println!("TOTP removed!");
        },
        SubCommand::List {} => {
            for (name, key) in accounts.iter() {
                println!("{}: {} (next code in {}s)", name, key.get_totp(), key.period as u64 - (chrono::Utc::now().timestamp() as u64) % key.period as u64);
            }
        },
        SubCommand::Get { name } => {
            if let Some(key) = accounts.get(&name) {
                println!("Copied to clipboard!");
                cli_clipboard::set_contents(key.get_totp()).unwrap();
            } else {
                eprintln!("{} does not exist!", name);
            }
        }
        SubCommand::Import { file } => {
            let file = std::fs::read_to_string(file).unwrap();
            let lines = file.lines();
            let mut ctr = 0;
            for line in lines {
                if line.starts_with("otpauth://") {
                    let (name, key) = parse_otpauth(line.to_string());
                    if accounts.contains_key(&name) {
                        eprintln!("TOTP for {} already exists!", name);
                        return;
                    }
                    accounts.insert(name, key);
                    ctr += 1;
                }
            }
            let json = serde_json::to_string(&accounts).unwrap();
            std::fs::write(&save_path, json).unwrap();
            println!("Successfully imported {} TOTP values!", ctr);
        },
        SubCommand::Rename { old, new } => {
            if let Some(key) = accounts.remove(&old) {
                accounts.insert(new.clone(), key);
                let json = serde_json::to_string(&accounts).unwrap();
                std::fs::write(&save_path, json).unwrap();
                println!("Successfully renamed {} to {}!", old, new);
            } else {
                eprintln!("{} does not exist!", old);
            }
        }
    }
}
