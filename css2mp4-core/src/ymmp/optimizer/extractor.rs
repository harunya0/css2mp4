use std::collections::BTreeSet;

use crate::ymmp::optimizer::extrema::find_extrema;
use crate::ymmp::optimizer::rdp::rdp_simplify_1d;

/// 複数のモーション系列（X, Y, Zoom, Rotation, Opacity）から、
/// 共通して必要なキーフレーム（中間点）のフレームインデックス一覧を抽出します。
pub fn extract_essential_keyframes(
    translate_x: &[f64],
    translate_y: &[f64],
    zoom: &[f64],
    rotation: &[f64],
    opacity: &[f64],
    tolerance: f64,
) -> Vec<usize> {
    let total_frames = translate_x
        .len()
        .max(translate_y.len())
        .max(zoom.len())
        .max(rotation.len())
        .max(opacity.len());

    if total_frames <= 2 {
        return (0..total_frames).collect();
    }

    let mut keyframe_set = BTreeSet::new();
    keyframe_set.insert(0);
    keyframe_set.insert(total_frames - 1);

    // 許容誤差（プロパティの単位に応じたスケール）
    let add_keyframes = |set: &mut BTreeSet<usize>, values: &[f64], tol: f64| {
        if values.len() >= 2 {
            for idx in rdp_simplify_1d(values, tol) {
                set.insert(idx);
            }
            for idx in find_extrema(values, tol * 0.5) {
                set.insert(idx);
            }
        }
    };

    // 位置: tolerance px
    add_keyframes(&mut keyframe_set, translate_x, tolerance);
    add_keyframes(&mut keyframe_set, translate_y, tolerance);

    // 拡大率: tolerance %
    add_keyframes(&mut keyframe_set, zoom, tolerance);

    // 回転: tolerance * 0.5 度
    add_keyframes(&mut keyframe_set, rotation, tolerance * 0.5);

    // 不透明度: tolerance * 1.0 %
    add_keyframes(&mut keyframe_set, opacity, tolerance);

    keyframe_set.into_iter().collect()
}
