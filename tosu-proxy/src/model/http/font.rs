use crate::service::font_service::FontInfo;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FontInfoResponse {
    pub items: Vec<FontInfo>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UploadFontResponse {
    pub ok: bool,
    pub font: FontInfo,
}
