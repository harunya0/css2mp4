/// 2D/3D Transform 行列の分解と幾何学的計算を行うモジュール。

/// `DOMMatrixReadOnly` の成分 (m11, m12, m21, m22) からスケール (X, Y) と回転角 (度) を分解・算出します。
pub fn decompose_2d_matrix(
    m11: f64,
    m12: f64,
    m21: f64,
    m22: f64,
) -> (f64, f64, f64) {
    let scale_x = (m11 * m11 + m12 * m12).sqrt();
    let scale_y = (m21 * m21 + m22 * m22).sqrt();
    let rotation_rad = m12.atan2(m11);
    let rotation_deg = rotation_rad.to_degrees();

    (scale_x, scale_y, rotation_deg)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identity_matrix() {
        let (sx, sy, rot) = decompose_2d_matrix(1.0, 0.0, 0.0, 1.0);
        assert!((sx - 1.0).abs() < 1e-6);
        assert!((sy - 1.0).abs() < 1e-6);
        assert!(rot.abs() < 1e-6);
    }

    #[test]
    fn test_scale_and_rotation() {
        // scale: 2.0, rotation: 90 deg
        // cos(90) = 0, sin(90) = 1
        // m11 = 2*0 = 0, m12 = 2*1 = 2, m21 = 2*(-1) = -2, m22 = 2*0 = 0
        let (sx, sy, rot) = decompose_2d_matrix(0.0, 2.0, -2.0, 0.0);
        assert!((sx - 2.0).abs() < 1e-6);
        assert!((sy - 2.0).abs() < 1e-6);
        assert!((rot - 90.0).abs() < 1e-6);
    }
}
