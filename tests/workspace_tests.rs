use std::{fs, time::{SystemTime, UNIX_EPOCH}};
use vyron_ide::workspace;

fn temp_dir() -> std::path::PathBuf {
    let n = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let p = std::env::temp_dir().join(format!("vyron-test-{n}"));
    fs::create_dir_all(&p).unwrap(); p
}

#[test]
fn create_and_read_file() {
    let root = temp_dir();
    let file = root.join("sample.py");
    workspace::create_file(&file).unwrap();
    workspace::write(&file, "print('ok')").unwrap();
    assert_eq!(workspace::read(&file).unwrap(), "print('ok')");
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn import_file_avoids_overwrite() {
    let root = temp_dir();
    let src = root.join("source.txt");
    fs::write(&src, "source").unwrap();
    let dest_dir = root.join("workspace");
    fs::create_dir(&dest_dir).unwrap();
    fs::write(dest_dir.join("source.txt"), "existing").unwrap();
    let imported = workspace::import_file(&src, &dest_dir).unwrap();
    assert_eq!(imported.file_name().unwrap(), "source-2.txt");
    fs::remove_dir_all(root).unwrap();
}
