use salvo::http::StatusCode;
use salvo::prelude::*;

use crate::util::read_audio_length;

#[handler]
async fn get_audio_length(req: &mut Request, res: &mut Response) {
    let path_query = req.query::<String>("path");
    if path_query.is_none() {
        res.status_code(StatusCode::BAD_REQUEST);
        res.render(Text::Plain(
            "Query parameter 'path' is required.".to_string(),
        ));
        return;
    }
    let path_value = path_query.unwrap();
    let length_result = read_audio_length(&path_value).await;
    if let Err(err) = length_result {
        res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
        res.render(Text::Plain(format!("Failed to read audio length: {}", err)));
        return;
    }
    let length = length_result.unwrap();
    res.render(Text::Plain(length.to_string()));
}

pub fn get_audio_route() -> Router {
    Router::with_path("audio/len").get(get_audio_length)
}
