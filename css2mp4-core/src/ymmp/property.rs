use serde::{Deserialize, Serialize};

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bezier {
    #[serde(rename = "Points")]
    pub points: Vec<BezierPoint>,
    #[serde(rename = "IsQuadratic")]
    pub is_quadratic: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BezierPoint {
    #[serde(rename = "Point")]
    pub point: Point,
    #[serde(rename = "ControlPoint1")]
    pub control_point1: Point,
    #[serde(rename = "ControlPoint2")]
    pub control_point2: Point,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Point {
    #[serde(rename = "X")]
    pub x: f64,
    #[serde(rename = "Y")]
    pub y: f64,
}

impl Bezier {
    /// デフォルトのリニアなイージングカーブ（2点、コントロールポイント ±0.3）。
    pub fn default_linear() -> Self {
        Bezier {
            points: vec![
                BezierPoint {
                    point: Point { x: 0.0, y: 0.0 },
                    control_point1: Point { x: -0.3, y: -0.3 },
                    control_point2: Point { x: 0.3, y: 0.3 },
                },
                BezierPoint {
                    point: Point { x: 1.0, y: 1.0 },
                    control_point1: Point { x: -0.3, y: -0.3 },
                    control_point2: Point { x: 0.3, y: 0.3 },
                },
            ],
            is_quadratic: false,
        }
    }
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
    ///
    /// YMM4 の Enum（YukkuriMovieMaker.Commons.AnimationType）に合わせて
    /// 複数値時は "直線移動"、単一値時は "なし" を指定します。
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
