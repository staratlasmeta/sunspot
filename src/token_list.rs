use crate::TOKEN_LIST;
use hudsucker::RequestOrResponse;
use hyper::body::to_bytes;
use hyper::header::{ACCESS_CONTROL_ALLOW_HEADERS, ACCESS_CONTROL_ALLOW_ORIGIN, CONTENT_TYPE};
use hyper::{Body, Request, Response};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Serialize, Deserialize)]
struct TokenList {
    address: String,
    name: Option<String>,
    symbol: Option<String>,
    #[serde(rename = "logoURI")]
    logo_uri: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct TokenListMintRequest {
    pub addresses: Vec<String>,
}

pub async fn handle_token_list_request(req: Request<Body>) -> RequestOrResponse {
    match *req.method() {
        hyper::Method::POST => {
            let body_bytes = to_bytes(req.into_body()).await.unwrap();
            let token_req = serde_json::from_slice::<TokenListMintRequest>(&body_bytes).unwrap();
            let tokens: Vec<TokenList> = token_req
                .addresses
                .iter()
                .filter_map(|address| {
                    TOKEN_LIST.get(address).map(|token| TokenList {
                        address: address.to_string(),
                        name: token.name.clone(),
                        symbol: token.symbol.clone(),
                        logo_uri: token.image_uri.clone(),
                    })
                })
                .collect();
            let res_body = json!({
                "content": tokens
            });
            let res = Response::builder()
                .header(CONTENT_TYPE, "application/json")
                .header(ACCESS_CONTROL_ALLOW_ORIGIN, "*")
                .header(ACCESS_CONTROL_ALLOW_HEADERS, "*")
                .status(200)
                .body(Body::from(res_body.to_string()))
                .unwrap();
            res.into()
        }
        _ => req.into(),
    }
}
