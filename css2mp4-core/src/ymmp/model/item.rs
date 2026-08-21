use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::Result;
use crate::ymmp::property::AnimatableProperty;

/// タイムライン上に配置される 1 個のアイテム（動画、画像、テキスト等）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Item {
    #[serde(rename = "$type")]
    pub type_name: String,
    #[serde(rename = "Frame")]
    pub frame: i64,
    #[serde(rename = "Layer")]
    pub layer: i64,
    #[serde(rename = "Length")]
    pub length: i64,
    /// その他の動的フィールド（X, Y, Zoom, Rotation, Opacity, KeyFrames 等）。
    #[serde(flatten)]
    pub fields: HashMap<String, Value>,
}

impl Item {
    /// 指定されたキーのアニメーション可能プロパティを取得します。
    pub fn get_animatable(&self, key: &str) -> Option<AnimatableProperty> {
        let value = self.fields.get(key)?;
        serde_json::from_value(value.clone()).ok()
    }

    /// 指定されたキーにアニメーション可能プロパティを設定します。
    pub fn set_animatable(&mut self, key: &str, prop: &AnimatableProperty) -> Result<()> {
        let json_value = serde_json::to_value(prop)?;
        self.fields.insert(key.to_string(), json_value);
        Ok(())
    }
}
