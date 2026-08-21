use serde::{Deserialize, Serialize};

use crate::ymmp::property::bezier::Bezier;

/// アニメーション可能な数値プロパティ（位置・拡大率・回転・不透明度など）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimatableProperty {
    #[serde(rename = "Values")]
    pub values: Vec<ValueEntry>,
    /// アニメーション区間の長さ（通常 0.0）。
    #[serde(rename = "Span")]
    pub span: f64,
    /// 移動方式（"なし", "直線移動", "加減速移動" など）。
    #[serde(rename = "AnimationType")]
    pub animation_type: String,
    #[serde(rename = "Bezier")]
    pub bezier: Bezier,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValueEntry {
    #[serde(rename = "Value")]
    pub value: f64,
}

impl AnimatableProperty {
    /// 静的な値（アニメーションなし）として生成する。
    pub fn from_static(value: f64) -> Self {
        AnimatableProperty {
            values: vec![ValueEntry { value }],
            span: 0.0,
            animation_type: "なし".to_string(),
            bezier: Bezier::default_linear(),
        }
    }

    /// 複数フレームのサンプリング値をキーフレームとして生成する。
    pub fn from_keyframes(values: &[f64]) -> Self {
        AnimatableProperty {
            values: values.iter().map(|&v| ValueEntry { value: v }).collect(),
            span: 0.0,
            animation_type: if values.len() > 1 {
                "直線移動".to_string()
            } else {
                "なし".to_string()
            },
            bezier: Bezier::default_linear(),
        }
    }
}
