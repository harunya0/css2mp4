use std::path::Path;

/// ローカルファイルパスを `file://` スキームの絶対 URL に変換する。
pub fn to_file_url(path: &Path) -> String {
    let abs = path
        .canonicalize()
        .unwrap_or_else(|_| path.to_path_buf());
    format!("file://{}", abs.display())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_to_file_url_format() {
        let url = to_file_url(&PathBuf::from("test.html"));
        assert!(url.starts_with("file://"));
    }
}
