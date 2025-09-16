use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub root_path: String,
    pub sections: Vec<SectionConfig>,

    #[serde(default)]
    pub reset_hours_after_midnight: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SectionConfig {
    pub title: String,

    #[serde(default)]
    pub persist: bool,
}

#[derive(Debug)]
pub enum ConfigError {
    FileNotFound,
    InvalidFormat,
}

static CONFIG_PATH: &str = "~/.config/dadi/config.yml";

pub fn read_config() -> Result<Config, ConfigError> {
    let expanded_path = shellexpand::tilde(CONFIG_PATH);
    let config_file_path = Path::new(expanded_path.as_ref());
    let config_file = File::open(config_file_path).map_err(|_| ConfigError::FileNotFound)?;
    let reader = BufReader::new(config_file);

    let mut config: Config =
        serde_yml::from_reader(reader).map_err(|_| ConfigError::InvalidFormat)?;
    config.root_path = shellexpand::tilde(&config.root_path).into_owned();
    return Ok(config);
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn working_config_deserialization() {
        let config = r#"
root_path: ~/sample/path/to/directory/
sections:
  - title: section 1
  - title: section 2
    persist: true
  - title: section 3
    persist: false"#;

        let config = serde_yml::from_str::<Config>(config).unwrap();
        assert_eq!("~/sample/path/to/directory/", config.root_path);

        assert_eq!("section 1", config.sections[0].title);
        assert_eq!(false, config.sections[0].persist);

        assert_eq!("section 2", config.sections[1].title);
        assert_eq!(true, config.sections[1].persist);

        assert_eq!("section 3", config.sections[2].title);
        assert_eq!(false, config.sections[2].persist);

        assert_eq!(0, config.reset_hours_after_midnight);
    }
}
