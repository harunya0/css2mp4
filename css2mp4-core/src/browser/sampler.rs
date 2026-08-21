use chromiumoxide::Page;
use serde::Deserialize;

use crate::browser::matrix::decompose_2d_matrix;
use crate::error::{Error, Result};

/// 1 フレーム内での要素の物理量（移動量、スケール、回転、不透明度）。
#[derive(Debug, Clone, Default)]
pub struct ComputedSample {
    pub translate_x: f64,
    pub translate_y: f64,
    pub scale: f64,
    pub rotation_deg: f64,
    pub opacity: f64,
}

#[derive(Debug, Deserialize)]
struct RawStyleResponse {
    opacity: f64,
    m11: f64,
    m12: f64,
    m21: f64,
    m22: f64,
    m41: f64,
    m42: f64,
}

/// ページ内の指定セレクタの要素の Computed Style を取得し、行列分解してサンプリングします。
pub async fn sample_element_style(page: &Page, selector: &str) -> Result<ComputedSample> {
    let script = format!(
        r#"(() => {{
            const el = document.querySelector({selector_json});
            if (!el) return null;
            const style = window.getComputedStyle(el);
            const transform = style.transform === 'none' ? 'matrix(1, 0, 0, 1, 0, 0)' : style.transform;
            const matrix = new DOMMatrixReadOnly(transform);
            const opacity = parseFloat(style.opacity) || 1.0;
            return {{
                opacity: opacity,
                m11: matrix.m11,
                m12: matrix.m12,
                m21: matrix.m21,
                m22: matrix.m22,
                m41: matrix.m41,
                m42: matrix.m42,
            }};
        }})()"#,
        selector_json = serde_json::to_string(selector).unwrap_or_default()
    );

    let raw: Option<RawStyleResponse> = page.evaluate(script).await?.into_value()?;
    let raw = raw.ok_or_else(|| Error::ElementNotFound(selector.to_string()))?;

    let (scale_x, scale_y, rotation_deg) =
        decompose_2d_matrix(raw.m11, raw.m12, raw.m21, raw.m22);

    Ok(ComputedSample {
        translate_x: raw.m41,
        translate_y: raw.m42,
        scale: (scale_x + scale_y) / 2.0,
        rotation_deg,
        opacity: raw.opacity,
    })
}
