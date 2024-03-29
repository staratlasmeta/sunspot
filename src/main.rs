mod config;
mod portfolio;
mod rpc;
mod token_list;

use anyhow::{bail, Context, Result};
use clap::Parser;
use hudsucker::{
    async_trait::async_trait,
    certificate_authority::OpensslAuthority,
    hyper::{Body, Request},
    openssl::{hash::MessageDigest, pkey::PKey, x509::X509},
    tokio_tungstenite::tungstenite::Message,
    *,
};
use hyper::Response;
use lazy_static::lazy_static;
use solana_sdk::pubkey::Pubkey;
use std::io::Read;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::str::FromStr;
use tracing::*;

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to install CTRL+C signal handler");
}

#[derive(Clone)]
struct LogHandler;

// Wallet API
// *wallet-api.solflare.com/*

// RPC
// *solflare.network*

// RPC Failover
// *failover.solflare.com*

// Token List
// *token-list-api.solana.cloud/v1/mints*

fn sunspot_config() -> Result<PathBuf> {
    home::home_dir()
        .context("Failed to get home directory")
        .map(|home| home.join(".config").join("sunspot"))
}

const RPC_URLS: [&str; 4] = [
    "solflare.network",
    "failover.solflare.com",
    "wallet-api.solflare.com/v2/tx/rpc",
    "api.testnet.solana.com",
];

#[async_trait]
impl HttpHandler for LogHandler {
    async fn handle_request(
        &mut self,
        _ctx: &HttpContext,
        req: Request<Body>,
    ) -> RequestOrResponse {
        if req.method() != "CONNECT" {
            let uri = req.uri().to_string();
            if RPC_URLS.iter().any(|rpc| uri.contains(rpc)) {
                info!("Rpc request: {uri}");
                return rpc::handle_rpc_request(req).await;
            } else if let Some(address) = portfolio::tokens::extract_address(uri.as_str()) {
                info!("Tokens request: {uri}");
                return match Pubkey::from_str(address) {
                    Ok(address) => portfolio::tokens::handle_tokens_request(req, address).await,
                    Err(e) => {
                        error!("Failed to parse address from {uri}: {e}");
                        req.into()
                    }
                };
            } else if uri.contains("token-list-api.solana.cloud/v1/mints") {
                info!("Token list request: {uri}");
                return token_list::handle_token_list_request(req).await;
            } else {
                info!("Unknown request: {uri}");
            }
            req.into()
        } else {
            req.into()
        }
    }

    async fn handle_response(&mut self, _ctx: &HttpContext, res: Response<Body>) -> Response<Body> {
        res
    }
}

#[async_trait]
impl WebSocketHandler for LogHandler {
    async fn handle_message(&mut self, _ctx: &WebSocketContext, msg: Message) -> Option<Message> {
        println!("{:?}", msg);
        Some(msg)
    }
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[clap(
        short = 'k',
        long,
        default_value = sunspot_config().unwrap().join("certs/sunspot.key").into_os_string()
    )]
    pub private_key: PathBuf,
    #[clap(short, long, default_value = sunspot_config().unwrap().join("certs/sunspot.pem").into_os_string())]
    pub certificate: PathBuf,
    /// Pulls token list from ~/.config/sunspot/tokens/<TOKEN_NAME>.json
    #[clap(short, long)]
    pub token_list_name: Option<String>,
    #[clap(short, long, default_value = "127.0.0.1:6969")]
    pub socket_address: SocketAddr,
    pub rpc: String,
}

impl Cli {
    fn rpc_client(&self) -> solana_client::nonblocking::rpc_client::RpcClient {
        solana_client::nonblocking::rpc_client::RpcClient::new(self.rpc.clone())
    }

    fn token_list(&self) -> Result<config::TokenList> {
        let token_list = CLI
            .token_list_name
            .as_ref()
            .map(|path| {
                let sunspot_config = sunspot_config()?;
                let path = sunspot_config.join("tokens").join(format!("{path}.json"));
                if !path.is_file() {
                    bail!("Token list not found at {}!", path.display());
                }
                let file = std::fs::File::open(path)?;
                serde_json::from_reader(file).context("Failed to parse token list file")
            })
            .transpose()?;
        Ok(token_list.unwrap_or_default())
    }
}

lazy_static! {
    pub static ref CLI: Cli = Cli::parse();
    pub static ref RPC_CLIENT: solana_client::nonblocking::rpc_client::RpcClient = CLI.rpc_client();
    pub static ref TOKEN_LIST: config::TokenList = CLI.token_list().unwrap();
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    lazy_static::initialize(&CLI);
    lazy_static::initialize(&TOKEN_LIST);
    lazy_static::initialize(&RPC_CLIENT);

    let private_key_bytes: Vec<u8> = {
        if !CLI.private_key.is_file() {
            bail!(
                "Private key file not found at {}!",
                CLI.private_key.display()
            );
        }
        let mut file =
            std::fs::File::open(&CLI.private_key).context("Failed to open private key file")?;
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)
            .context("Failed to read private key file")?;
        bytes
    };
    let ca_cert_bytes: Vec<u8> = {
        if !CLI.certificate.is_file() {
            bail!(
                "CA certificate file not found at {}!",
                CLI.certificate.display()
            );
        }
        let mut file =
            std::fs::File::open(&CLI.certificate).context("Failed to open CA certificate file")?;
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)
            .context("Failed to read CA certificate file")?;
        bytes
    };
    let private_key =
        PKey::private_key_from_pem(&private_key_bytes).context("Failed to parse private key")?;
    let ca_cert = X509::from_pem(&ca_cert_bytes).context("Failed to parse CA certificate")?;

    let ca = OpensslAuthority::new(private_key, ca_cert, MessageDigest::sha256(), 1_000);

    let proxy = Proxy::builder()
        .with_addr(CLI.socket_address)
        .with_rustls_client()
        .with_ca(ca)
        .with_http_handler(LogHandler)
        .build();

    println!("Starting Sunspot Proxy on {}", CLI.socket_address);
    println!("Listening...");

    if let Err(e) = proxy.start(shutdown_signal()).await {
        bail!("{}", e);
    }
    Ok(())
}
