use crate::error::Result;
use crate::ymmp::model::Item;
use crate::ymmp::property::AnimatableProperty;

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
    /// サンプリング結果を、指定アイテムの X/Y/Zoom/Rotation/Opacity に上書きする。
    pub fn overwrite_item(&self, item: &mut Item, span_seconds: f64) -> Result<()> {
        let apply = |item: &mut Item, key: &str, values: &[f64]| -> Result<()> {
            if values.is_empty() {
                return Ok(());
            }
            let prop = if values.len() == 1 {
                AnimatableProperty::from_static(values[0])
            } else {
                AnimatableProperty::from_keyframes(values, span_seconds)
            };
            item.set_animatable(key, &prop)
        };

        apply(item, "X", &self.translate_x)?;
        apply(item, "Y", &self.translate_y)?;
        apply(item, "Zoom", &self.zoom_percent)?;
        apply(item, "Rotation", &self.rotation_deg)?;
        apply(item, "Opacity", &self.opacity_percent)?;
        Ok(())
    }
}
