use std::collections::LinkedList;

use worker::*;
use worker::Method::Get;

use crate::response::{build_response, RedirectorResponse, ResponseType};

mod response;

#[event(fetch)]
async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    let router = Router::new();

    router
        .get_async("/:url", |_, ctx| async move {
            let Some(input) = ctx.param("url") else {
                return build_response(
                    RedirectorResponse {
                        redirect_urls: None,
                        result_url: None,
                        response_status: ResponseType::BadRequest,
                    },
                    400,
                );
            };

            let link_result = urlencoding::decode(input);

            let Ok(link) = link_result else {
                return build_response(
                    RedirectorResponse {
                        redirect_urls: None,
                        result_url: None,
                        response_status: ResponseType::UrlMalformed,
                    },
                    400,
                );
            };

            if !(link.starts_with("https://") || link.starts_with("http://")) {
                return build_response(
                    RedirectorResponse {
                        redirect_urls: None,
                        result_url: None,
                        response_status: ResponseType::UrlMalformed,
                    },
                    400,
                );
            }

            let mut redirect_urls = LinkedList::new();
            let mut headers = Headers::new();
            headers.set("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/115.0.0.0 Safari/537.36")?;

            let mut request_init = RequestInit::new();
            request_init
                .with_method(Get)
                .with_headers(headers)
                .with_redirect(RequestRedirect::Manual);

            redirect_urls.push_back(link.as_ref().to_string());

            let url = Url::parse(link.as_ref())?;
            let request = Request::new_with_init(url.as_str(), &request_init)?;
            let mut response = Fetch::Request(request).send().await?;

            while response.status_code() == 301 || response.status_code() == 302 {
                let Some(location) = response.headers().get("location")? else {
                    break;
                };

                redirect_urls.push_back(location.to_string());

                let new_request = Request::new_with_init(location.as_str(), &request_init)?;
                response = Fetch::Request(new_request).send().await?;
            }

            let result_url = redirect_urls.back().cloned();

            if redirect_urls.len() == 1 {
                build_response(
                    RedirectorResponse {
                        redirect_urls: None,
                        result_url,
                        response_status: ResponseType::Ok,
                    },
                    200,
                )
            } else {
                build_response(
                    RedirectorResponse {
                        redirect_urls: Some(redirect_urls),
                        result_url,
                        response_status: ResponseType::Ok,
                    },
                    200,
                )
            }
        })
        .run(req, env)
        .await
}
