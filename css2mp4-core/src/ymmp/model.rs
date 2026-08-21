use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::error::Result;
use crate::ymmp::property::AnimatableProperty;

/// YMM4 プロジェクト全体のルート構造。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YmmpProject {
    #[serde(rename = "FilePath")]
    pub file_path: String,
    #[serde(rename = "SelectedTimelineIndex")]
    pub selected_timeline_index: i32,
    #[serde(rename = "Timelines")]
    pub timelines: Vec<Timeline>,
    #[serde(rename = "Characters")]
    pub characters: Vec<Value>,
    #[serde(rename = "CollapsedGroups")]
    pub collapsed_groups: Vec<Value>,
    #[serde(rename = "LayoutXml")]
    pub layout_xml: String,
    #[serde(rename = "ToolStates")]
    pub tool_states: Value,
}

/// YMM4 タイムライン。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Timeline {
    #[serde(rename = "ID")]
    pub id: String,
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "VideoInfo")]
    pub video_info: VideoInfo,
    /// 縦線（BPM ガイド等）の設定。パススルーで保持。
    #[serde(rename = "VerticalLine")]
    pub vertical_line: Value,
    #[serde(rename = "Items")]
    pub items: Vec<Item>,
    /// 未知のフィールドやツール拡張フィールドの欠落を防ぐため保持。
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

/// プロジェクトの動画解像度・フレームレート情報。
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

/// タイムライン上のアイテム（VideoItem / ImageItem / TextItem 等）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Item {
    #[serde(rename = "$type")]
    pub type_name: String,
    #[serde(flatten)]
    pub fields: Map<String, Value>,
}

impl Item {
    /// アニメーション可能なプロパティ（`X` / `Y` / `Opacity` 等）を取得する。
    pub fn get_animatable(&self, key: &str) -> Option<AnimatableProperty> {
        self.fields
            .get(key)
            .and_then(|v| serde_json::from_value(v.clone()).ok())
    }

    /// アニメーション可能なプロパティを上書きまたは新規設定する。
    pub fn set_animatable(&mut self, key: &str, prop: &AnimatableProperty) -> Result<()> {
        let value = serde_json::to_value(prop)?;
        self.fields.insert(key.to_string(), value);
        Ok(())
    }
}

impl YmmpProject {
    /// 指定タイムライン内の指定インデックスのアイテムを取得する（可変参照）。
    pub fn item_mut(&mut self, timeline_index: usize, item_index: usize) -> Option<&mut Item> {
        self.timelines
            .get_mut(timeline_index)?
            .items
            .get_mut(item_index)
    }

    /// 指定タイムライン内から `Remark`（アイテム名）で最初に一致したアイテムを取得する（可変参照）。
    pub fn item_by_remark_mut(&mut self, timeline_index: usize, remark: &str) -> Option<&mut Item> {
        self.timelines.get_mut(timeline_index)?.items.iter_mut().find(|item| {
            item.fields
                .get("Remark")
                .and_then(|v| v.as_str())
                .map(|r| r == remark)
                .unwrap_or(false)
        })
    }
}
