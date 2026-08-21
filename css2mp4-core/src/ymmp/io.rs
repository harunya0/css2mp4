use std::path::Path;

use crate::error::Result;
use crate::ymmp::model::YmmpProject;

/// UTF-8 BOM（YMM4 が出力するファイルに付与されている）。
pub const UTF8_BOM: &str = "\u{feff}";

impl YmmpProject {
    /// `.ymmp` ファイルを読み込む（UTF-8 BOM 付きを想定）。
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let raw = std::fs::read_to_string(path)?;
        let stripped = raw.strip_prefix(UTF8_BOM).unwrap_or(&raw);
        let project = serde_json::from_str(stripped)?;
        Ok(project)
    }

    /// `.ymmp` ファイルとして書き出す（UTF-8 BOM 付き・改行なしの圧縮形式）。
    pub fn save(&self, path: impl AsRef<Path>) -> Result<()> {
        let json = serde_json::to_string(self)?;
        let mut out = String::with_capacity(json.len() + UTF8_BOM.len());
        out.push_str(UTF8_BOM);
        out.push_str(&json);
        std::fs::write(path, out)?;
        Ok(())
    }
}
