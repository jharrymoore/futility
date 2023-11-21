use std::path::PathBuf;

pub fn file_watcher(file: PathBuf) -> Option<Vec<String>> {
    let contents = std::fs::read_to_string(file);
    match contents {
        Ok(contents) => {
            let lines: Vec<String> = contents.lines().map(|s| s.to_string()).collect();
            Some(lines)
        }
        Err(_) => None,
    }
}

