use std::{env, fs, path::PathBuf};

pub fn tmp_file_path(name: &str) -> PathBuf {
    let mut dir = env::temp_dir();
    dir.push("zparse_tests");
    let _ = fs::create_dir_all(&dir);
    dir.push(name);
    dir
}
