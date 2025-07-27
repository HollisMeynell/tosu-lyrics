use serde::{Deserialize, Serialize};

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
enum BlockItem {
    Bid { bid: u32 },
    Sid { sid: u32 },
    Title { title: String },
}
