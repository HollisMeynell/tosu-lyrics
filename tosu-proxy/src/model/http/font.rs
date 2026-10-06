use crate::service::font_service::FontEntry;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FontListResponse {
    pub items: Vec<FontEntry>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UploadFontResponse {
    pub ok: bool,
    pub font: FontEntry,
}
