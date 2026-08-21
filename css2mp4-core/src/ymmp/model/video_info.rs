use serde::{Deserialize, Serialize};

/// タイムラインの動画フォーマット設定。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoInfo {
    #[serde(rename = "FPS")]
    pub fps: u32,
    #[serde(rename = "Hz")]
    pub hz: u32,
    #[serde(rename = "Width")]
    pub width: u32,
    #[serde(rename = "Height")]
    pub height: u32,
    #[serde(rename = "BackgroundColor")]
    pub background_color: String,
}
