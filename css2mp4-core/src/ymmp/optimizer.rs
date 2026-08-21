/// 時系列データの間引きとキーフレーム抽出を行うオプティマイザ。
use std::collections::BTreeSet;

/// 1次元の時系列データ (frame_index, value) に対して Ramer-Douglas-Peucker (RDP) アルゴリズムを適用し、
/// 重要なフレーム番号（インデックス）のみを抽出します。
pub fn rdp_simplify_1d(values: &[f64], epsilon: f64) -> Vec<usize> {
    if values.len() <= 2 {
        return (0..values.len()).collect();
    }

    let mut keep = vec![false; values.len()];
    keep[0] = true;
    keep[values.len() - 1] = true;

    fn rdp_recursive(values: &[f64], start: usize, end: usize, epsilon: f64, keep: &mut [bool]) {
        if end <= start + 1 {
            return;
        }

        let start_val = values[start];
        let end_val = values[end];
        let dx = (end - start) as f64;
        let dy = end_val - start_val;
        let line_len_sq = dx * dx + dy * dy;

        let mut max_dist = 0.0;
        let mut index = start;

        for i in (start + 1)..end {
            // 点と直線の偏差（直線補間からの値のズレ）
            let dist = if line_len_sq == 0.0 {
                (values[i] - start_val).abs()
            } else {
                let t = (i - start) as f64 / dx;
                let expected = start_val + t * dy;
                (values[i] - expected).abs()
            };

            if dist > max_dist {
                max_dist = dist;
                index = i;
            }
        }

        if max_dist > epsilon {
            keep[index] = true;
            rdp_recursive(values, start, index, epsilon, keep);
            rdp_recursive(values, index, end, epsilon, keep);
        }
    }

    rdp_recursive(values, 0, values.len() - 1, epsilon, &mut keep);

    keep.iter()
        .enumerate()
        .filter_map(|(i, &k)| if k { Some(i) } else { None })
        .collect()
}

/// 極大値・極小値（動きの折り返し点・ピーク）を検出します。
pub fn find_extrema(values: &[f64], threshold: f64) -> Vec<usize> {
    let mut extrema = Vec::new();
    if values.len() < 3 {
        return extrema;
    }

    for i in 1..(values.len() - 1) {
        let prev = values[i - 1];
        let curr = values[i];
        let next = values[i + 1];

        let d1 = curr - prev;
        let d2 = next - curr;

        // 符号が反転する点（山または谷）
        if (d1 * d2 < 0.0) && (d1.abs() > threshold || d2.abs() > threshold) {
            extrema.push(i);
        }
    }

    extrema
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rdp_simplify_constant_and_linear() {
        // 直線データは始点と終点のみ抽出されること
        let linear: Vec<f64> = (0..100).map(|i| i as f64 * 2.0).collect();
        let keyframes = rdp_simplify_1d(&linear, 0.1);
        assert_eq!(keyframes, vec![0, 99]);
    }

    #[test]
    fn test_rdp_simplify_bounce_curve() {
        // バウンスカーブ（途中で反転する点がある）
        let mut bounce = vec![0.0; 100];
        for i in 0..50 {
            bounce[i] = i as f64 * 2.0; // 0 -> 100
        }
        for i in 50..100 {
            bounce[i] = 100.0 - (i - 50) as f64 * 2.0; // 100 -> 0
        }

        let keyframes = rdp_simplify_1d(&bounce, 1.0);
        assert!(keyframes.contains(&0));
        assert!(keyframes.contains(&50)); // ピーク
        assert!(keyframes.contains(&99));
        assert!(keyframes.len() <= 5);
    }
}
