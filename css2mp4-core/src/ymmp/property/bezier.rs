use serde::{Deserialize, Serialize};

/// ベジェ曲線によるイージングデータ。
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
