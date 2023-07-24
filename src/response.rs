use serde::{Deserialize, Serialize};
use std::collections::LinkedList;
use worker::{Cors, Headers, Response};

fn build_headers() -> worker::Result<Headers> {
    let mut headers = Headers::new();
    headers.set("Content-Type", "application/json")?;
    Ok(headers)
}

pub fn build_response(response: RedirectorResponse, status_code: u16) -> worker::Result<Response> {
    Response::ok(serde_json::to_string(&response).unwrap())?
        .with_headers(build_headers()?)
        .with_status(status_code)
        .with_cors(&Cors::new().with_origins(vec!["*"]))
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RedirectorResponse {
    #[serde(rename = "resultUrl", skip_serializing_if = "Option::is_none")]
    pub result_url: Option<String>,

    #[serde(rename = "status")]
    pub response_status: ResponseType,

    #[serde(rename = "urls", skip_serializing_if = "Option::is_none")]
    pub redirect_urls: Option<LinkedList<String>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum ResponseType {
    #[serde(rename = "BAD_REQUEST")]
    BadRequest,

    #[serde(rename = "OK")]
    Ok,

    #[serde(rename = "URL_MALFORMED")]
    UrlMalformed,
}
