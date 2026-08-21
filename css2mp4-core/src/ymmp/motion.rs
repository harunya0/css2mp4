use serde_json::json;

use crate::error::Result;
use crate::ymmp::model::Item;
use crate::ymmp::property::{AnimatableProperty, Bezier, ValueEntry};

/// CSS アニメーションから 1 フレームごとにサンプリングした時系列データ。
#[derive(Debug, Clone, Default)]
pub struct MotionSamples {
    /// 水平移動量 (px)（YMM4 の `X` に対応）。
    pub translate_x: Vec<f64>,
    /// 垂直移動量 (px)（YMM4 の `Y` に対応）。
    pub translate_y: Vec<f64>,
    /// 拡大率 (%)（YMM4 の `Zoom` に対応）。
    pub zoom_percent: Vec<f64>,
    /// 回転角 (度)（YMM4 の `Rotation` に対応）。
    pub rotation_deg: Vec<f64>,
    /// 不透明度 (%)（YMM4 の `Opacity` に対応）。
    pub opacity_percent: Vec<f64>,
}

impl MotionSamples {
    /// サンプリング結果を、指定アイテムの X/Y/Zoom/Rotation/Opacity および KeyFrames に上書きする。
    ///
    /// YMM4 の仕様に基づき、全キーフレーム数に合わせてアイテムの `KeyFrames`（中間点リスト）
    /// を更新し、各アニメーションプロパティの要素数を `KeyFrames.Count + 1` に揃えます。
    pub fn overwrite_item(&self, item: &mut Item, _span_seconds: f64) -> Result<()> {
        let total_frames = self
            .translate_x
            .len()
            .max(self.translate_y.len())
            .max(self.zoom_percent.len())
            .max(self.rotation_deg.len())
            .max(self.opacity_percent.len());

        if total_frames == 0 {
            return Ok(());
        }

        // 1. アイテムの KeyFrames を更新
        let keyframe_count = total_frames.saturating_sub(1);
        let frames: Vec<i64> = (1..total_frames as i64).collect();

        item.fields.insert(
            "KeyFrames".to_string(),
            json!({
                "Frames": frames,
                "Count": keyframe_count,
            }),
        );

        // 2. 各サンプリング値を total_frames の長さに揃えて適用するヘルパー
        let pad_and_apply =
            |item: &mut Item, key: &str, samples: &[f64], default_val: f64| -> Result<()> {
                let existing_val = item
                    .get_animatable(key)
                    .and_then(|p| p.values.first().map(|v| v.value))
                    .unwrap_or(default_val);

                let padded: Vec<f64> = if samples.is_empty() {
                    vec![existing_val; total_frames]
                } else if samples.len() == total_frames {
                    samples.to_vec()
                } else {
                    let last = *samples.last().unwrap_or(&existing_val);
                    let mut v = samples.to_vec();
                    v.resize(total_frames, last);
                    v
                };

                let is_constant = padded.windows(2).all(|w| (w[0] - w[1]).abs() < 1e-6);
                let animation_type = if is_constant || total_frames <= 1 {
                    "なし".to_string()
                } else {
                    "直線移動".to_string()
                };

                let prop = AnimatableProperty {
                    values: padded.iter().map(|&v| ValueEntry { value: v }).collect(),
                    span: 0.0,
                    animation_type,
                    bezier: Bezier::default_linear(),
                };
                item.set_animatable(key, &prop)
            };

        pad_and_apply(item, "X", &self.translate_x, 0.0)?;
        pad_and_apply(item, "Y", &self.translate_y, 0.0)?;
        pad_and_apply(item, "Zoom", &self.zoom_percent, 100.0)?;
        pad_and_apply(item, "Rotation", &self.rotation_deg, 0.0)?;
        pad_and_apply(item, "Opacity", &self.opacity_percent, 100.0)?;

        // 3. アイテム内のその他の既存 AnimatableProperty（Z, Volume, Pan 等）も
        //    中間点数と要素数が合わないと YMM4 で破損エラーになるため、元の AnimationType を維持してパディング
        let keys_to_sync: Vec<String> = item
            .fields
            .iter()
            .filter_map(|(k, v)| {
                if k != "X" && k != "Y" && k != "Zoom" && k != "Rotation" && k != "Opacity" {
                    if let Ok(prop) = serde_json::from_value::<AnimatableProperty>(v.clone()) {
                        if prop.values.len() != total_frames {
                            return Some(k.clone());
                        }
                    }
                }
                None
            })
            .collect();

        for key in keys_to_sync {
            if let Some(mut prop) = item.get_animatable(&key) {
                let first_val = prop.values.first().map(|v| v.value).unwrap_or(0.0);
                prop.values = vec![ValueEntry { value: first_val }; total_frames];
                item.set_animatable(&key, &prop)?;
            }
        }

        Ok(())
    }
}
