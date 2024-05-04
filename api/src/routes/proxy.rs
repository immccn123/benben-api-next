use reqwest::{Client, StatusCode};
use rocket::response::Responder;

#[derive(Debug, Clone, Responder)]
enum ProxyResponse {
    #[response(status = 200, content_type = "image/png")]
    Ok(Vec<u8>),
    #[response(status = 404)]
    NotFound(()),
    #[response(status = 500)]
    InternalServerError(()),
}

impl From<reqwest::Error> for ProxyResponse {
    fn from(_value: reqwest::Error) -> Self {
        Self::InternalServerError(())
    }
}

#[get("/proxy/userPic/<filename>")]
async fn get_userpic(filename: &str) -> Result<ProxyResponse, ProxyResponse> {
    if !filename.ends_with(".png") {
        return Ok(ProxyResponse::NotFound(()));
    }

    let client = Client::new();
    let url = format!("https://cdn.luogu.com.cn/upload/usericon/{}", filename);

    match client.get(&url).send().await {
        Ok(response) => {
            match response.status() {
                StatusCode::OK => (),
                StatusCode::NOT_FOUND => return Ok(ProxyResponse::NotFound(())),
                _ => return Ok(ProxyResponse::InternalServerError(())),
            }

            if let Some(content_type) = response.headers().get("content-type") {
                if content_type != "image/png" {
                    return Ok(ProxyResponse::NotFound(()));
                }
            }

            let bytes = response.bytes().await?.to_vec();
            Ok(ProxyResponse::Ok(bytes))
        }
        Err(_) => Err(ProxyResponse::InternalServerError(())),
    }
}

export_routes![get_userpic];
