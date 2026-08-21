use std::collections::HashMap;
use std::path::Path;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::Result;
use crate::ymmp::io::{load_ymmp_from_file, save_ymmp_to_file};
use crate::ymmp::model::item::Item;
use crate::ymmp::model::timeline::Timeline;

/// ゆっくりMovieMaker4 のプロジェクトファイル全体を表すデータモデル。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YmmpProject {
    #[serde(rename = "FilePath")]
    pub file_path: Option<String>,
    #[serde(rename = "Timelines")]
    pub timelines: Vec<Timeline>,
    /// 未知のトップレベルフィールドを保持。
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

impl YmmpProject {
    /// ファイルパスから YMM4 プロジェクトをロードします（UTF-8 BOM対応）。
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        load_ymmp_from_file(path)
    }

    /// YMM4 プロジェクトをファイルへ保存します（UTF-8 BOM付き）。
    pub fn save(&self, path: impl AsRef<Path>) -> Result<()> {
        save_ymmp_to_file(self, path)
    }

    /// 指定されたタイムラインインデックスとアイテムインデックスのアイテムへの可変参照を取得します。
    pub fn item_mut(&mut self, timeline_index: usize, item_index: usize) -> Option<&mut Item> {
        self.timelines
            .get_mut(timeline_index)
            .and_then(|t| t.items.get_mut(item_index))
    }
}
