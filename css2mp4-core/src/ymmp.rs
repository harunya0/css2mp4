//! YMM4（ゆっくりMovieMaker4）プロジェクトファイル（.ymmp）の読み書きと、
//! CSSアニメーションからサンプリングしたモーションを既存アイテムへ
//! 上書きするための処理。
//!
//! # スキーマについて
//! `.ymmp` は圧縮されたJSON（UTF-8 BOM付き・改行なし）。
//! 以下のスキーマは実際のYMM4プロジェクトファイル
//! （`tests/fixtures/sample.ymmp`、単一の `VideoItem` を含む）を
//! 解析して確認したフィールドに基づく。
//!
//! ```text
//! {
//!   "FilePath": "...",
//!   "SelectedTimelineIndex": 0,
//!   "Timelines": [ { "ID", "Name", "VideoInfo", "VerticalLine", "Items": [...] } ],
//!   "Characters": [],
//!   "CollapsedGroups": [],
//!   "LayoutXml": "...",
//!   "ToolStates": { ... }
//! }
//! ```
//!
//! アイテム内の `X` / `Y` / `Z` / `Opacity` / `Zoom` / `Rotation` などの
//! アニメーション可能なプロパティは共通して以下の形をとる：
//! ```text
//! {
//!   "Values": [ { "Value": 100.0 } ],
//!   "Span": 0.0,
//!   "AnimationType": "なし",
//!   "Bezier": { "Points": [...], "IsQuadratic": false }
//! }
//! ```
//!
//! ## 未確認の部分（要検証）
//! サンプルファイルはすべて `AnimationType: "なし"` / `Values` が1要素の
//! **静的な値**だった。CSSアニメーションを複数キーフレームとして書き出す際の
//! `AnimationType` の実際の文字列や、複数 `Values` と `Span` の対応関係
//! （区間全体の秒数なのか、フレーム数なのか等）は未確認。
//! `AnimationType::CustomGuess` として仮実装しており、実際にYMM4で
//! パラメータにアニメーションを追加保存した `.ymmp` を入手し次第、
//! [`AnimatableProperty::from_keyframes`] を修正する必要がある。

use std::path::Path;

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::error::Result;

/// UTF-8 BOM（YMM4が出力するファイルに付与されている）。
const UTF8_BOM: &str = "\u{feff}";

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Timeline {
    #[serde(rename = "ID")]
    pub id: String,
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "VideoInfo")]
    pub video_info: VideoInfo,
    /// 縦線（BPMガイド等）の設定。今回のツールでは触らないためパススルー。
    #[serde(rename = "VerticalLine")]
    pub vertical_line: Value,
    #[serde(rename = "Items")]
    pub items: Vec<Item>,
    /// `LayerSettings` / `CurrentFrame` / `Length` / `MaxLayer` など、
    /// 今回のツールでは編集しないその他のタイムライン情報。
    /// ラウンドトリップでの欠落を防ぐため保持するのみ。
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

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

/// タイムライン上のアイテム。
///
/// 具体的なアイテム種別（VideoItem / ImageItem / TextItem 等）ごとに
/// フィールド構成が大きく異なり、かつプラグイン由来のエフェクトも
/// 任意の `$type` を持ちうるため、厳密な型は定義せず
/// `$type` + フィールドマップとして保持する。
/// これにより未知のフィールドを欠落させずに読み書きできる（ラウンドトリップ安全）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Item {
    #[serde(rename = "$type")]
    pub type_name: String,
    #[serde(flatten)]
    pub fields: Map<String, Value>,
}

impl Item {
    /// アニメーション可能なプロパティ（`X` / `Y` / `Opacity` 等）を
    /// 取得する。存在しない、または形が合わない場合は `None`。
    pub fn get_animatable(&self, key: &str) -> Option<AnimatableProperty> {
        self.fields
            .get(key)
            .and_then(|v| serde_json::from_value(v.clone()).ok())
    }

    /// アニメーション可能なプロパティを上書きする。
    /// キーが存在しない場合は新規に追加する。
    pub fn set_animatable(&mut self, key: &str, prop: &AnimatableProperty) -> Result<()> {
        let value = serde_json::to_value(prop)?;
        self.fields.insert(key.to_string(), value);
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimatableProperty {
    #[serde(rename = "Values")]
    pub values: Vec<ValueEntry>,
    /// アニメーション区間の長さ。単位・意味は未確認（TODO参照）。
    #[serde(rename = "Span")]
    pub span: f64,
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
    /// サンプルファイル中で確認できたデフォルトのイージングカーブ
    /// （直線的な2点、コントロールポイントは ±0.3）。
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
    /// 静的な値（アニメーションなし）として構築する。
    /// サンプルファイルで確認済みの確実な形。
    pub fn from_static(value: f64) -> Self {
        AnimatableProperty {
            values: vec![ValueEntry { value }],
            span: 0.0,
            animation_type: "なし".to_string(),
            bezier: Bezier::default_linear(),
        }
    }

    /// CSSアニメーションからサンプリングした複数フレーム分の値を
    /// キーフレームとして書き込む。
    ///
    /// # 未確認事項（要検証・TODO）
    /// `AnimationType` に何を指定すべきか、`Span` が区間長を秒/フレーム
    /// のどちらで表すか、`Values` の各要素がどう時間軸に配置されるかは
    /// 実際に動いている `.ymmp` サンプルで未検証。
    /// 現状は最も自然に読める仮説（`Span` を区間長・秒、`Values` を
    /// 等間隔のキーフレームとみなす）で実装している。
    /// 実サンプル入手後にここを修正すること。
    pub fn from_keyframes(values: &[f64], span_seconds: f64) -> Self {
        AnimatableProperty {
            values: values.iter().map(|&v| ValueEntry { value: v }).collect(),
            span: span_seconds,
            // TODO: 実際のAnimationType文字列を要確認。
            animation_type: "自動".to_string(),
            bezier: Bezier::default_linear(),
        }
    }
}

impl YmmpProject {
    /// `.ymmp` ファイルを読み込む（UTF-8 BOM付きを想定）。
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let raw = std::fs::read_to_string(path)?;
        let stripped = raw.strip_prefix(UTF8_BOM).unwrap_or(&raw);
        let project = serde_json::from_str(stripped)?;
        Ok(project)
    }

    /// `.ymmp` ファイルとして書き出す（UTF-8 BOM付き・改行なしの
    /// 元ファイルと同じ圧縮形式を維持する）。
    pub fn save(&self, path: impl AsRef<Path>) -> Result<()> {
        let json = serde_json::to_string(self)?;
        let mut out = String::with_capacity(json.len() + UTF8_BOM.len());
        out.push_str(UTF8_BOM);
        out.push_str(&json);
        std::fs::write(path, out)?;
        Ok(())
    }

    /// 指定タイムライン内の、指定インデックスのアイテムを取得する（可変参照）。
    pub fn item_mut(&mut self, timeline_index: usize, item_index: usize) -> Option<&mut Item> {
        self.timelines
            .get_mut(timeline_index)?
            .items
            .get_mut(item_index)
    }

    /// 指定タイムライン内から、`Remark`（アイテム名）で最初に一致した
    /// アイテムを取得する（可変参照）。
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

/// CSSアニメーションから1フレームごとにサンプリングした値。
/// `capture::FrameCapturer` 側での取得を想定。
#[derive(Debug, Clone, Default)]
pub struct MotionSamples {
    /// px単位の水平移動量（YMM4の `X` にそのまま対応させる想定）。
    pub translate_x: Vec<f64>,
    /// px単位の垂直移動量（YMM4の `Y` に対応）。
    pub translate_y: Vec<f64>,
    /// 拡大率（%）。CSS `scale` を 100 倍したもの（YMM4の `Zoom` に対応）。
    pub zoom_percent: Vec<f64>,
    /// 回転角（度）。YMM4の `Rotation` に対応。
    pub rotation_deg: Vec<f64>,
    /// 不透明度（%）。CSS `opacity`(0-1) を 100 倍したもの。
    pub opacity_percent: Vec<f64>,
}

impl MotionSamples {
    /// サンプリング結果を、指定アイテムの X/Y/Zoom/Rotation/Opacity に
    /// 上書きする。`span_seconds` はアニメーション区間の長さ（秒）。
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
