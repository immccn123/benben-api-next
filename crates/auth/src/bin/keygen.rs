use base64ct::{Base64UrlUnpadded, Encoding};
use clap::{Parser, Subcommand};
use serde_json::json;

#[derive(Parser)]
#[command(name = "keygen", about = "SSO key generation")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    #[command(about = "Generate all server keys (client + JWT)")]
    Server,
    #[command(about = "Generate downstream app credential")]
    App {
        #[arg(short, long)]
        client_id: String,
        #[arg(long)]
        callback_url: String,
    },
}

fn main() {
    dotenvy::dotenv().ok();
    match Cli::parse().command {
        Command::Server => gen_server_keys(),
        Command::App {
            client_id,
            callback_url,
        } => gen_app(&client_id, &callback_url),
    }
}

fn gen_server_keys() {
    let client = auth::crypto::keys::generate_p256_key_pair();
    let jwt = auth::crypto::keys::generate_p256_key_pair();

    println!("=== Server Keys (.env) ===");
    println!();
    println!("--- Client Verification (public) ---");
    println!(
        "AUTH_CLIENT_VERIFY_KEY={}",
        client.public_key_pem.trim().replace('\n', "\\n")
    );
    println!();
    println!("--- JWT Signing/Verification ---");
    println!(
        "AUTH_JWT_SIGN_KEY={}",
        jwt.private_key_pem.trim().replace('\n', "\\n")
    );
    println!(
        "AUTH_JWT_VERIFY_KEY={}",
        jwt.public_key_pem.trim().replace('\n', "\\n")
    );
    println!();
    println!("--- Client Signing Key (for keygen only, NOT in .env) ---");
    println!("{}", client.private_key_pem.trim());
    println!();
    println!("Set it when running keygen app:");
    println!(
        "  AUTH_CLIENT_SIGN_KEY='...' cargo run --bin keygen -- app -c <id> --callback-url <url>"
    );
}

fn gen_app(client_id: &str, callback_url: &str) {
    let payload = json!({
        "client_id": client_id,
        "callback_url": callback_url,
    });

    let client_sign_key = std::env::var("AUTH_CLIENT_SIGN_KEY").unwrap_or_default();
    if client_sign_key.is_empty() {
        eprintln!("AUTH_CLIENT_SIGN_KEY not set. Credential will NOT be signed.");
        let payload_str = serde_json::to_string(&payload).unwrap();
        println!(
            "{}",
            Base64UrlUnpadded::encode_string(payload_str.as_bytes())
        );
        return;
    }

    let payload_str = serde_json::to_string(&payload).unwrap();
    let payload_b64 = Base64UrlUnpadded::encode_string(payload_str.as_bytes());
    let sk: p256::ecdsa::SigningKey =
        pkcs8::DecodePrivateKey::from_pkcs8_pem(&client_sign_key).unwrap();
    use signature::Signer;
    let sig: p256::ecdsa::Signature = sk.sign(payload_str.as_bytes());
    let sig_b64 = Base64UrlUnpadded::encode_string(&sig.to_bytes());
    let credential = format!("{payload_b64}.{sig_b64}");

    println!("=== App Credential ===");
    println!("client_id: {client_id}");
    println!("callback_url: {callback_url}");
    println!();
    println!("Credential (use in /flow/auth?credential=...&callback_url=...):");
    println!("{credential}");
}
