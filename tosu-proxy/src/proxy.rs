use actix_web::{HttpResponse, web};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::LazyLock;

static CLIENT: LazyLock<Client> = LazyLock::new(|| Client::new());

#[derive(Deserialize)]
pub struct ApiRequest {
    url: String,
    method: Option<String>,
    header: Option<HashMap<String, String>>,
    body: Option<Vec<u8>>,
}

#[derive(Serialize)]
pub struct ApiResponse {
    status: u16,
    headers: HashMap<String, String>,
    body: String,
}

/// 后端的代理接口
/// 接受请求 [ApiRequest]
pub async fn handler(req: web::Json<ApiRequest>) -> HttpResponse {
    let req = req.into_inner();
    let url = &req.url;
    let method = req.method.as_deref().unwrap_or("GET");
    let mut request_builder = match method.to_uppercase().as_str() {
        "GET" => CLIENT.get(url),
        "POST" => CLIENT.post(url),
        "PUT" => CLIENT.put(url),
        "DELETE" => CLIENT.delete(url),
        "PATCH" => CLIENT.patch(url),
        _ => return HttpResponse::BadRequest().body("Unsupported HTTP method"),
    };

    if let Some(header) = &req.header {
        for (key, val) in header {
            request_builder = request_builder.header(key, val);
        }
    }

    if let Some(body) = &req.body {
        request_builder = request_builder.body(body.clone());
    }

    match request_builder.send().await {
        Ok(response) => {
            let status = response.status().as_u16();
            let headers = response
                .headers()
                .iter()
                .map(|(key, value)| (key.to_string(), value.to_str().unwrap_or("").to_string()))
                .collect();
            let body = response.text().await.unwrap_or_default();

            HttpResponse::Ok().json(ApiResponse {
                status,
                headers,
                body,
            })
        }
        Err(err) => HttpResponse::InternalServerError().body(format!("Request failed: {:?}", err)),
    }
}
