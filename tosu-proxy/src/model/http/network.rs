use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerAddressesResponse {
    pub host: String,
    pub port: u16,
    pub addresses: Vec<NetAddress>,
}

/// 单个可访问地址；`preferred` 仅标记排序首位的推荐项。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NetAddress {
    pub address: String,
    pub name: String,
    pub preferred: bool,
}
