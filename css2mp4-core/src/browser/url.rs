use std::path::Path;

/// ローカルファイルパスを `file://` スキームの絶対 URL に変換する。
///
/// Windows の UNC プレフィックス (`\\?\`) を除去し、ブラウザが解釈可能な
/// `file:///C:/path/to/file.html` 形式に正規化します。
pub fn to_file_url(path: &Path) -> String {
    let canonical = dunce::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    url::Url::from_file_path(&canonical)
        .map(|u| u.to_string())
        .unwrap_or_else(|_| format!("file:///{}", canonical.display()).replace('\\', "/"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_to_file_url_format() {
        let url = to_file_url(&PathBuf::from("Cargo.toml"));
        assert!(url.starts_with("file:///"));
        assert!(!url.contains('\\'));
        assert!(!url.contains("?"));
    }
}
