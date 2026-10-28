use anyhow::{Context, Result};
use std::{fs, path::{Path, PathBuf}};
use walkdir::WalkDir;

#[derive(Clone, Debug)]
pub struct FileEntry { pub path: PathBuf, pub is_dir: bool, pub depth: usize }

pub fn scan(root: &Path) -> Vec<FileEntry> {
    WalkDir::new(root).follow_links(false).into_iter().filter_map(|entry| entry.ok())
        .filter(|e| e.path() == root || !e.file_name().to_string_lossy().starts_with('.'))
        .filter(|e| e.path() != root)
        .map(|e| FileEntry { path: e.path().to_path_buf(), is_dir: e.file_type().is_dir(),
            depth: e.depth().saturating_sub(1) }).collect()
}
pub fn read(path: &Path) -> Result<String> {
    fs::read_to_string(path).with_context(|| format!("Could not read {}", path.display()))
}
pub fn write(path: &Path, text: &str) -> Result<()> {
    fs::write(path, text).with_context(|| format!("Could not save {}", path.display()))
}
pub fn create_file(path: &Path) -> Result<()> {
    fs::OpenOptions::new().write(true).create_new(true).open(path)
        .with_context(|| format!("Could not create {}", path.display()))?; Ok(())
}
pub fn create_dir(path: &Path) -> Result<()> {
    fs::create_dir(path).with_context(|| format!("Could not create folder {}", path.display()))
}
pub fn import_file(source: &Path, root: &Path) -> Result<PathBuf> {
    let name = source.file_name().context("Selected file has no name")?;
    let mut dest = root.join(name);
    if dest.exists() {
        let stem = source.file_stem().unwrap_or_default().to_string_lossy();
        let ext = source.extension().map(|e| format!(".{}", e.to_string_lossy())).unwrap_or_default();
        let mut n = 2;
        loop {
            dest = root.join(format!("{stem}-{n}{ext}"));
            if !dest.exists() { break; }
            n += 1;
        }
    }
    fs::copy(source, &dest).with_context(|| format!("Could not import {}", source.display()))?;
    Ok(dest)
}
pub fn rename(path: &Path, new_name: &str) -> Result<PathBuf> {
    if new_name.trim().is_empty() || new_name.contains('/') || new_name.contains('\\') {
        anyhow::bail!("Enter a valid file or folder name");
    }
    let dest = path.parent().context("Cannot rename workspace root")?.join(new_name);
    fs::rename(path, &dest).with_context(|| format!("Could not rename {}", path.display()))?;
    Ok(dest)
}
pub fn delete(path: &Path) -> Result<()> {
    if path.is_dir() { fs::remove_dir_all(path)?; } else { fs::remove_file(path)?; }
    Ok(())
}
