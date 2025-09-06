use serde::{Deserialize, Serialize};

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct BlockItem {
    pub bid: u32,
    pub sid: u32,
    pub title: String,
}
