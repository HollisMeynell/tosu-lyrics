use crate::server::Assets;
use salvo::serve_static::static_embed;

pub fn get_file_route() -> salvo::Router {
    salvo::Router::with_path("{*path}").get(
        static_embed::<Assets>()
            .defaults(["index.html"])
            .fallback("lyrics/index.html"),
    )
}
