use crate::CLI;
use axum::http::Response;
use hudsucker::RequestOrResponse;
use hyper::header::{ACCESS_CONTROL_ALLOW_HEADERS, ACCESS_CONTROL_ALLOW_ORIGIN, CONTENT_TYPE};
use hyper::{Body, Request};

pub(crate) async fn handle_rpc_request(req: Request<Body>) -> RequestOrResponse {
    let method = req.method();
    let client = reqwest::Client::new();
    let req_type = req.headers().get(CONTENT_TYPE);
    let mut res = client.request(method.clone(), CLI.rpc.as_str());
    if let Some(req_type) = req_type {
        res = res.header(CONTENT_TYPE, req_type);
    }
    let res = res.body(req.into_body()).send().await.unwrap();
    let content_type = res.headers().get(CONTENT_TYPE).unwrap().clone();
    let bytes = res.bytes().await.unwrap();
    let mut response = Response::new(bytes.into());
    response.headers_mut().insert(CONTENT_TYPE, content_type);
    response
        .headers_mut()
        .insert(ACCESS_CONTROL_ALLOW_ORIGIN, "*".parse().unwrap());
    response
        .headers_mut()
        .insert(ACCESS_CONTROL_ALLOW_HEADERS, "*".parse().unwrap());
    response.into()
}
