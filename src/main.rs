use std::collections::HashMap;
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
    size: u8,
    period: u64
}

#[derive(Subcommand, Debug)]
enum SubCommand {
    Add {
        #[clap(long, short = 'k')]
        key: String,
        #[clap(long, short = 'n')]
        name: String,
        #[clap(long, short = 's', default_value_t = 6)]
        size: u8,
        #[clap(long, short = 'p', default_value_t = 30)]
        period: u64
    },
    Remove {
        #[clap(long, short = 'n')]
        name: String
    },
    List {},
    Get {
        #[clap(long, short = 'n')]
        name: String
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
            % 10u32.pow(u32::from(self.size));
        format!("{:01$}", code, self.size as usize)
    }
    fn get_totp(&self) -> String {
        self.get_hotp((chrono::Utc::now().timestamp() as u64) / &self.period)
    }
}

fn main() {
    let args = Args::parse();
    let saved = std::fs::read_to_string("2fa.json").unwrap_or_default();
    let mut accounts: HashMap<String, Key> = serde_json::from_str(&saved).unwrap_or_default();

    match args.cmd {
        SubCommand::Add { key, name, size, period } => {
            if accounts.contains_key(&name) {
                eprintln!("TOTP for {} already exists!", name);
                return;
            }
            accounts.insert(name, Key { secret: key, size, period });
            let json = serde_json::to_string(&accounts).unwrap();
            std::fs::write("2fa.json", json).unwrap();
            println!("TOTP added!");
        },
        SubCommand::Remove { name } => {
            if accounts.remove(&name).is_none() {
                eprintln!("TOTP for {} does not exist!", name);
                return;
            }
            let json = serde_json::to_string(&accounts).unwrap();
            std::fs::write("2fa.json", json).unwrap();
            println!("TOTP removed!");
        },
        SubCommand::List {} => {
            for (name, key) in accounts.iter() {
                println!("{}: {} (next code in {}s)", name, key.get_totp(), key.period - (chrono::Utc::now().timestamp() as u64) % key.period);
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
    }
}
