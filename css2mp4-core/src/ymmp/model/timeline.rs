use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::ymmp::model::item::Item;
use crate::ymmp::model::video_info::VideoInfo;

/// YMM4 の 1 つのタイムライン。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Timeline {
    #[serde(rename = "ID")]
    pub id: String,
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "VideoInfo")]
    pub video_info: VideoInfo,
    #[serde(rename = "Items")]
    pub items: Vec<Item>,
    /// 未知のフィールドを保持し、再書き出し時のデータ損失を防止。
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}
