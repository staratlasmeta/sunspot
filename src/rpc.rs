use crate::CLI;
use axum::http::Response;
use hudsucker::RequestOrResponse;
use hyper::header::CONTENT_TYPE;
use hyper::{Body, Request};

pub(crate) async fn handle_rpc_request(req: Request<Body>) -> RequestOrResponse {
    let res = reqwest::Client::new()
        .post(CLI.rpc.as_str())
        .body(req.into_body())
        .header(CONTENT_TYPE, "application/json")
        .send()
        .await
        .unwrap();
    // let content_type = res
    //     .headers()
    //     .get("content-type")
    //     .unwrap()
    //     .to_str()
    //     .unwrap()
    //     .to_string();
    let mut response = Response::new(Body::empty());
    *response.headers_mut() = res.headers().clone();
    // return if content_type.contains("application/json") {
    let bytes = res.bytes().await.unwrap();
    *response.body_mut() = bytes.into();
    response.into()
    // } else if content_type.contains("text/plain") {
    //     let text = res.bytes().await.unwrap();
    //     *response.body_mut() = text.into();
    //     response.into()
    // } else {
    //     let bytes = res.bytes().await.unwrap();
    //     *response.body_mut() = bytes.into();
    //     response.into()
    // };
}
