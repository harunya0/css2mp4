use serde::Deserialize;

/// 1フレーム分のサンプリング結果。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StyleSample {
    /// CSS `opacity`（0.0〜1.0）。
    pub opacity: f64,
    /// `transform` 行列から抽出した水平移動量（px）。
    pub translate_x: f64,
    /// `transform` 行列から抽出した垂直移動量（px）。
    pub translate_y: f64,
    /// `transform` 行列から抽出した回転角（度）。
    pub rotation_deg: f64,
    /// `transform` 行列から抽出した拡大率（1.0 = 等倍）。
    pub scale: f64,
}

#[derive(Deserialize)]
pub(crate) struct RawStyleSample {
    pub opacity: f64,
    pub tx: f64,
    pub ty: f64,
    #[serde(rename = "angleDeg")]
    pub angle_deg: f64,
    pub scale: f64,
}

impl From<RawStyleSample> for StyleSample {
    fn from(raw: RawStyleSample) -> Self {
        Self {
            opacity: raw.opacity,
            translate_x: raw.tx,
            translate_y: raw.ty,
            rotation_deg: raw.angle_deg,
            scale: raw.scale,
        }
    }
}

/// 指定セレクタの要素から `opacity` および `transform` を抽出・分解する JS スクリプトを生成する。
pub(crate) fn build_sampling_script(selector: &str) -> String {
    format!(
        r#"(() => {{
            const el = document.querySelector({selector});
            if (!el) {{ throw new Error('element not found: ' + {selector}); }}
            const cs = getComputedStyle(el);
            const opacity = parseFloat(cs.opacity);
            let tx = 0, ty = 0, angleDeg = 0, scale = 1;
            const t = cs.transform;
            if (t && t !== 'none') {{
                const m = new DOMMatrixReadOnly(t);
                tx = m.m41;
                ty = m.m42;
                scale = Math.sqrt(m.m11 * m.m11 + m.m12 * m.m12);
                angleDeg = Math.atan2(m.m12, m.m11) * (180 / Math.PI);
            }}
            return {{ opacity, tx, ty, angleDeg, scale }};
        }})()"#,
        selector = serde_json::to_string(selector).unwrap_or_else(|_| "\"body\"".to_string()),
    )
}
