use salvo::prelude::*;
use salvo::serve_static::StaticDir;

static STATIC_FILE_PATHS: [&str; 3] = [
    "./static/lyrics",
    "./static",
    "./",
];

#[handler]
async fn root_redirect(res: &mut Response) {
    res.render(Redirect::permanent("/lyrics"));
}

pub fn get_file_route() -> Router {
    let static_handler = StaticDir::new(STATIC_FILE_PATHS)
        .defaults("index.html")
        .fallback("index.html")
        .auto_list(false);

    Router::new()
        .push(Router::with_path("lyrics").goal(static_handler))
        .push(Router::with_path("/").get(root_redirect))
}
