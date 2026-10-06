use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DisplayClientDto {
    pub id: String,
    pub identity: Option<String>,
    pub connected_at: i64,
    pub user_agent: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientListResponse {
    pub total: usize,
    pub items: Vec<DisplayClientDto>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BlinkResponse {
    pub ok: bool,
    pub blinked: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplySettingsResponse {
    pub ok: bool,
    pub clients: usize,
    pub keys: Vec<String>,
    pub sent: usize,
}
