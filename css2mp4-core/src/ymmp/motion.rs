use serde_json::json;

use crate::error::Result;
use crate::ymmp::model::Item;
use crate::ymmp::optimizer::extract_essential_keyframes;
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
    /// サンプリング結果を、最適化されたキーフレーム（中間点）として指定アイテムに上書きする。
    ///
    /// デフォルトの許容誤差 (`0.5`) を用いて不要な中間点を間引きます。
    pub fn overwrite_item(&self, item: &mut Item, span_seconds: f64) -> Result<()> {
        self.overwrite_item_with_tolerance(item, span_seconds, 0.5)
    }

    /// 許容誤差 `tolerance` を指定して、キーフレームを間引きつつアイテムに上書きする。
    ///
    /// - `tolerance <= 0.0`: 間引きを行わず全フレームを中間点として登録します。
    /// - `tolerance > 0.0`: RDP アルゴリズムと極値検出により、動きの主要なキーフレームのみを抽出します。
    pub fn overwrite_item_with_tolerance(
        &self,
        item: &mut Item,
        _span_seconds: f64,
        tolerance: f64,
    ) -> Result<()> {
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

        // 1. 抽出するキーフレームのインデックスリスト（0-indexed）を決定
        let keyframe_indices: Vec<usize> = if tolerance <= 0.0 {
            (0..total_frames).collect()
        } else {
            extract_essential_keyframes(
                &self.translate_x,
                &self.translate_y,
                &self.zoom_percent,
                &self.rotation_deg,
                &self.opacity_percent,
                tolerance,
            )
        };

        let num_points = keyframe_indices.len();
        if num_points == 0 {
            return Ok(());
        }

        // 2. YMM4 の KeyFrames（中間点リスト）を更新
        // KeyFrames.Frames は 0 番目を除いた中間点のフレーム番号（1..）
        let frames: Vec<i64> = keyframe_indices
            .iter()
            .skip(1)
            .map(|&idx| idx as i64)
            .collect();
        let keyframe_count = frames.len();

        item.fields.insert(
            "KeyFrames".to_string(),
            json!({
                "Frames": frames,
                "Count": keyframe_count,
            }),
        );

        // 3. 各プロパティからキーフレームインデックスに対応する値をサンプリングして適用
        let pad_and_apply =
            |item: &mut Item, key: &str, samples: &[f64], default_val: f64| -> Result<()> {
                let existing_val = item
                    .get_animatable(key)
                    .and_then(|p| p.values.first().map(|v| v.value))
                    .unwrap_or(default_val);

                let key_values: Vec<f64> = keyframe_indices
                    .iter()
                    .map(|&idx| {
                        if idx < samples.len() {
                            samples[idx]
                        } else {
                            *samples.last().unwrap_or(&existing_val)
                        }
                    })
                    .collect();

                let is_constant = key_values.windows(2).all(|w| (w[0] - w[1]).abs() < 1e-6);
                let animation_type = if is_constant || num_points <= 1 {
                    "なし".to_string()
                } else {
                    "直線移動".to_string()
                };

                let prop = AnimatableProperty {
                    values: key_values.iter().map(|&v| ValueEntry { value: v }).collect(),
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

        // 4. その他のプロパティ（Volume, Pan, Z 等）も中間点数と一致させる
        let keys_to_sync: Vec<String> = item
            .fields
            .iter()
            .filter_map(|(k, v)| {
                if k != "X" && k != "Y" && k != "Zoom" && k != "Rotation" && k != "Opacity" {
                    if let Ok(prop) = serde_json::from_value::<AnimatableProperty>(v.clone()) {
                        if prop.values.len() != num_points {
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
                prop.values = vec![ValueEntry { value: first_val }; num_points];
                item.set_animatable(&key, &prop)?;
            }
        }

        Ok(())
    }
}
