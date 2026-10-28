use egui::Color32;
use std::{path::PathBuf, time::SystemTime};

#[derive(Clone)]
pub struct EditorTab {
    pub path: PathBuf,
    pub text: String,
    pub saved_text: String,
    pub last_modified: Option<SystemTime>,
}
impl EditorTab {
    pub fn new(path: PathBuf, text: String) -> Self {
        let saved_text = text.clone();
        let last_modified = std::fs::metadata(&path).and_then(|m| m.modified()).ok();
        Self { path, text, saved_text, last_modified }
    }
    pub fn dirty(&self) -> bool { self.text != self.saved_text }
    pub fn title(&self) -> String {
        let name = self.path.file_name().unwrap_or_default().to_string_lossy();
        if self.dirty() { format!("● {name}") } else { name.to_string() }
    }
}

pub fn language_for(path: &std::path::Path) -> &'static str {
    match path.extension().and_then(|s| s.to_str()).unwrap_or("").to_ascii_lowercase().as_str() {
        "rs" => "rs", "py" => "py", "js" | "mjs" | "cjs" => "js", "ts" | "tsx" => "ts",
        "html" | "htm" => "html", "css" => "css", "json" => "json", "md" => "md",
        "c" | "h" => "c", "cpp" | "cc" | "hpp" => "cpp", "java" => "java", "go" => "go",
        "sql" => "sql", "sh" | "bash" => "sh", _ => "txt"
    }
}
pub fn outline(text: &str, ext: &str) -> Vec<(usize, String)> {
    let mut out = Vec::new();
    for (idx, line) in text.lines().enumerate() {
        let t = line.trim();
        let found = match ext {
            "rs" => ["fn ", "pub fn ", "struct ", "enum ", "trait ", "mod ", "impl "].iter().any(|p| t.starts_with(p)),
            "py" => t.starts_with("def ") || t.starts_with("async def ") || t.starts_with("class "),
            "js" | "ts" | "tsx" => t.starts_with("function ") || t.starts_with("class ") || t.starts_with("const ") && t.contains("=>"),
            "go" => t.starts_with("func ") || t.starts_with("type "),
            "c" | "h" | "cpp" | "hpp" => t.starts_with("class ") || t.starts_with("struct ") || t.starts_with("#define "),
            "java" => t.contains(" class ") || t.starts_with("class ") || t.contains(" void ") || t.starts_with("public "),
            _ => false
        };
        if found { out.push((idx + 1, t.chars().take(90).collect())); }
    }
    out
}
pub fn line_count(text: &str) -> usize { text.lines().count().max(1) }
pub fn line_number_color() -> Color32 { Color32::from_rgb(111, 121, 135) }
