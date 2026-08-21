use std::path::Path;

use crate::error::Result;
use crate::ymmp::model::YmmpProject;

/// UTF-8 BOM（YMM4 が出力するファイルに付与されている）。
pub const UTF8_BOM: &str = "\u{feff}";

/// `.ymmp` ファイルを読み込む（UTF-8 BOM 付きを想定）。
pub fn load_ymmp_from_file(path: impl AsRef<Path>) -> Result<YmmpProject> {
    let raw = std::fs::read_to_string(path)?;
    let stripped = raw.strip_prefix(UTF8_BOM).unwrap_or(&raw);
    let project = serde_json::from_str(stripped)?;
    Ok(project)
}

/// `.ymmp` ファイルとして書き出す（UTF-8 BOM 付き・改行なしの圧縮形式）。
pub fn save_ymmp_to_file(project: &YmmpProject, path: impl AsRef<Path>) -> Result<()> {
    let json = serde_json::to_string(project)?;
    let mut out = String::with_capacity(json.len() + UTF8_BOM.len());
    out.push_str(UTF8_BOM);
    out.push_str(&json);
    std::fs::write(path, out)?;
    Ok(())
}
