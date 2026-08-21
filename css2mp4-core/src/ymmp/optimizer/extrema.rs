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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_extrema() {
        let wave = vec![0.0, 10.0, 20.0, 15.0, 10.0, 5.0, 8.0, 12.0];
        let extrema = find_extrema(&wave, 1.0);
        // index 2 (20.0: peak), index 5 (5.0: trough)
        assert_eq!(extrema, vec![2, 5]);
    }
}
