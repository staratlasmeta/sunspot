use crate::{RPC_CLIENT, TOKEN_LIST};
use hudsucker::RequestOrResponse;
use hyper::header::{ACCESS_CONTROL_ALLOW_HEADERS, ACCESS_CONTROL_ALLOW_ORIGIN, CONTENT_TYPE};
use hyper::{Body, Request, Response};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::to_string;
use solana_account_decoder::parse_token::UiTokenAccount;
use solana_account_decoder::UiAccountData;
use solana_client::rpc_request::TokenAccountsFilter;
use solana_sdk::native_token::lamports_to_sol;
use solana_sdk::pubkey::Pubkey;

pub fn extract_address(url: &str) -> Option<&str> {
    let re =
        Regex::new(r"wallet-api\.solflare\.com/v3/portfolio/tokens/(?P<address>[^/?]+)").unwrap();
    re.captures(url)
        .and_then(|cap| cap.name("address").map(|m| m.as_str()))
}

#[derive(Debug, PartialEq, PartialOrd, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokensResponse {
    pub tokens: Vec<Token>,
    pub value: TokensValue,
    pub sol_value: TokensValue,
    pub errors: Vec<String>,
}

#[derive(Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Token {
    pub name: Option<String>,
    pub symbol: Option<String>,
    pub decimals: u8,
    pub mint: String,
    pub image_uri: Option<String>,
    pub accounts: Vec<TokenAccount>,
    pub coingecko_id: Option<String>,
    pub total_ui_amount: Option<f64>,
    pub verified: Option<bool>,
    pub price: Option<TokenAccountPrice>,
    pub sol_price: Option<TokenAccountSolPrice>,
}

impl Default for Token {
    fn default() -> Self {
        Self {
            name: None,
            symbol: None,
            decimals: 0,
            mint: Pubkey::default().to_string(),
            image_uri: None,
            accounts: vec![],
            coingecko_id: None,
            total_ui_amount: None,
            verified: None,
            price: None,
            sol_price: None,
        }
    }
}

impl Token {
    pub fn sol(key: Pubkey, amount: u64) -> Self {
        let ui_amount = lamports_to_sol(amount);
        Self {
            name: Some("Solana".to_string()),
            symbol: Some("SOL".to_string()),
            decimals: 9,
            mint: Pubkey::default().to_string(),
            image_uri: Some("https://raw.githubusercontent.com/trustwallet/assets/master/blockchains/solana/info/logo.png".to_string()),
            accounts: vec![
                TokenAccount {
                    pubkey: key.to_string(),
                    amount: amount.to_string(),
                    ui_amount: Some(ui_amount),
                    delegation: None,
                }
            ],
            verified: Some(true),
            ..Default::default()
        }
    }
}

#[derive(Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenAccount {
    pub pubkey: String,
    pub amount: String,
    pub ui_amount: Option<f64>,
    pub delegation: Option<Delegation>,
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct Delegation {}

impl Default for TokenAccount {
    fn default() -> Self {
        Self {
            pubkey: Pubkey::default().to_string(),
            amount: "0".to_string(),
            ui_amount: None,
            delegation: None,
        }
    }
}
#[derive(Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenAccountPrice {
    pub price: f64,
    pub change: f64,
    pub usd_price: f64,
    pub usd_hange: f64,
    pub currency: String,
}

#[derive(Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct TokenAccountSolPrice {
    pub price: f64,
    pub change: f64,
}

#[derive(Debug, PartialEq, PartialOrd, Serialize, Deserialize, Default)]
pub struct TokensValue {
    pub total: f64,
    pub change: f64,
    pub percentage: f64,
}

pub async fn handle_tokens_request(_req: Request<Body>, key: Pubkey) -> RequestOrResponse {
    let get_balance = RPC_CLIENT.get_balance(&key);
    let get_token_accounts_by_owner =
        RPC_CLIENT.get_token_accounts_by_owner(&key, TokenAccountsFilter::ProgramId(spl_token::ID));
    let (balance, token_accounts) = tokio::join!(get_balance, get_token_accounts_by_owner);
    let balance = balance.unwrap_or_default();
    let token_accounts = token_accounts.unwrap_or_default();
    let mut tokens = vec![Token::sol(key, balance)];
    let mut token_accounts: Vec<Token> = token_accounts
        .into_iter()
        .filter_map(|rpc_token| {
            let account_data = rpc_token.account.data;
            if let UiAccountData::Json(parsed) = account_data {
                #[derive(Debug, Serialize, Deserialize)]
                struct UiTokenAccountReturn {
                    info: UiTokenAccount,
                }
                let parsed: UiTokenAccountReturn = serde_json::from_value(parsed.parsed).unwrap();
                let ui_token_account = parsed.info;
                let token_config = TOKEN_LIST.get(&ui_token_account.mint);
                let token = Token {
                    name: token_config.and_then(|token_config| token_config.name.clone()),
                    symbol: token_config.and_then(|token_config| token_config.symbol.clone()),
                    decimals: ui_token_account.token_amount.decimals,
                    mint: ui_token_account.mint,
                    image_uri: token_config.and_then(|token_config| token_config.image_uri.clone()),
                    accounts: vec![TokenAccount {
                        pubkey: rpc_token.pubkey,
                        amount: ui_token_account.token_amount.amount,
                        ui_amount: ui_token_account.token_amount.ui_amount,
                        delegation: None,
                    }],
                    verified: token_config.is_some().into(),
                    ..Default::default()
                };
                return Some(token);
            }
            None
        })
        .collect();
    tokens.append(&mut token_accounts);
    let mut response = Response::new(Body::empty());
    response
        .headers_mut()
        .insert(CONTENT_TYPE, "application/json".parse().unwrap());
    response
        .headers_mut()
        .insert(ACCESS_CONTROL_ALLOW_ORIGIN, "*".parse().unwrap());
    response
        .headers_mut()
        .insert(ACCESS_CONTROL_ALLOW_HEADERS, "*".parse().unwrap());
    let tokens = TokensResponse {
        tokens,
        ..Default::default()
    };
    *response.body_mut() = Body::from(to_string(&tokens).unwrap());
    response.into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract() {
        let url = "https://wallet-api.solflare.com/v2/portfolio/tokens/7MMRAQ9dbHFVRHtWaNVrZKi9XNP4ji9uP2qi9RQ5ngEE?network=mainnet&currency=USD";
        match extract_address(url) {
            Some(address) => assert_eq!(address, "7MMRAQ9dbHFVRHtWaNVrZKi9XNP4ji9uP2qi9RQ5ngEE"),
            None => panic!("No match found"),
        };
    }
}
