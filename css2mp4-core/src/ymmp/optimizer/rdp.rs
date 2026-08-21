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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rdp_simplify_constant_and_linear() {
        let linear: Vec<f64> = (0..100).map(|i| i as f64 * 2.0).collect();
        let keyframes = rdp_simplify_1d(&linear, 0.1);
        assert_eq!(keyframes, vec![0, 99]);
    }
}
