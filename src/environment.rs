use std::{
    collections::HashMap,
    fs::File,
    io::{self, BufRead},
    path::PathBuf,
};

pub fn parse_environment_file(path: PathBuf) -> io::Result<HashMap<String, String>> {
    let file = File::open(path)?;
    let reader = io::BufReader::new(file);

    let mut map = HashMap::new();

    for line in reader.lines() {
        let line = line?;
        let line = line.trim();
        if !line.is_empty() && !line.starts_with('#') {
            if let Some((key, value)) = line.split_once('=') {
                map.insert(key.trim().to_string(), value.trim().to_string());
            }
        }
    }

    Ok(map)
}
