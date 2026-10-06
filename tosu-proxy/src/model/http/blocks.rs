use crate::service::block_service::BlockRule;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockListResponse {
    pub total: usize,
    pub items: Vec<BlockRule>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteBlockResponse {
    pub removed: bool,
    pub rule: BlockRule,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClearBlocksResponse {
    pub removed: u64,
}
