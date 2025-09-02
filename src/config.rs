use serde::{Serialize, Deserialize};
use std::ffi::OsString;
use std::fs::File;
use std::io::BufReader;

#[derive(Serialize, Deserialize)]
struct Config {
    root_path: String,
}

#[derive(Debug)]
enum ConfigError {
    FileNotFound,
    InvalidFormat,
}

static CONFIG_PATH: &str = "~/.config/dadi/config.yml";

fn read_config() -> Result<Config, ConfigError> {
    let config_file_path = OsString::from(CONFIG_PATH);
    let config_file = File::open(config_file_path)
        .map_err(|_| ConfigError::FileNotFound)?;
    let reader = BufReader::new(config_file);

    serde_yml::from_reader(reader)
        .map_err(|_| ConfigError::InvalidFormat)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn working_config_deserialization() {
        let config = r#"
root_path: ~/sample/path/to/directory/"#;

        serde_yml::from_str::<Config>(config).unwrap();
    }
}
