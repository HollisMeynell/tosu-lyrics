use crate::config::{CONFIG_ENDPOINT_SERVER_ADDRESSES, GLOBAL_CONFIG};
use crate::model::http::network::ServerAddressesResponse;
use crate::service::network_service::list_net_addresses;
use salvo::prelude::*;

#[handler]
async fn get_server_addresses(res: &mut Response) {
    res.render(Json(ServerAddressesResponse {
        host: GLOBAL_CONFIG.server.clone(),
        port: GLOBAL_CONFIG.port,
        addresses: list_net_addresses(),
    }));
}

pub fn get_network_route() -> Router {
    Router::with_path(CONFIG_ENDPOINT_SERVER_ADDRESSES).get(get_server_addresses)
}
