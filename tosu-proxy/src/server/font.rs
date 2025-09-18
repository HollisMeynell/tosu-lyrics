use crate::config::{
    CONFIG_ENDPOINT_FONT, CONFIG_ENDPOINT_FONT_DOWNLOAD, CONFIG_ENDPOINT_FONT_UPLOAD,
};
use salvo::fs::NamedFile;
use salvo::http::header::CONTENT_TYPE;
use salvo::prelude::*;
use std::env;
use std::path::PathBuf;
use tokio::fs::File;

const FONT_FILE: &str = "font";

fn get_font_path() -> PathBuf {
    let directory = env::current_dir().expect("cannot get current directory");
    directory.join(FONT_FILE)
}

async fn get_font_file() -> File {
    let path = get_font_path();
    if path.exists() {
        File::open(path).await.expect("cannot open font")
    } else {
        File::create(path).await.expect("cannot create file")
    }
}

#[handler]
async fn upload_font(req: &mut Request, res: &mut Response) {
    let file = match req.first_file().await {
        Some(f) => f,
        None => {
            res.render(StatusCode::BAD_REQUEST);
            return;
        }
    };
    let mut origin = File::open(file.path()).await.expect("cannot open font");
    let mut target = get_font_file().await;
    tokio::io::copy(&mut origin, &mut target)
        .await
        .expect("cannot copy");
    res.render(StatusCode::OK);
}

#[handler]
async fn download_font(req: &mut Request, res: &mut Response) {
    let path = get_font_path();
    if !path.exists() {
        res.render(StatusCode::NOT_FOUND);
        return;
    }
    if let Ok(file) = NamedFile::open(path).await {
        let mut header = req.headers_mut();
        header.insert(CONTENT_TYPE, "font/collection".parse().unwrap());
        file.send(header, res).await;
    } else {
        res.render(StatusCode::NOT_FOUND);
    }
}

pub fn get_font_route() -> Router {
    Router::with_path(CONFIG_ENDPOINT_FONT)
        .push(Router::with_path(CONFIG_ENDPOINT_FONT_UPLOAD).post(upload_font))
        .push(Router::with_path(CONFIG_ENDPOINT_FONT_DOWNLOAD).get(download_font))
}
