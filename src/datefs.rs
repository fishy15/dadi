use std::fs;

use std::ffi::OsString;
use std::io::Error;
use std::path::Path;

#[derive(Debug)]
pub enum DateFSError {
    BaseMissing,
    InvalidFolder(OsString),
    OSError(Error),
}

// The previous number that occurs before the current one in the file system.
// Also checks to make sure that there are only integer file names present.
fn previous_before_int(base_path: &Path, bound: u32) -> Result<Option<u32>, DateFSError> {
    let paths = fs::read_dir(base_path).map_err(|_| DateFSError::BaseMissing)?;
    let folder_names = paths
        .map(|p| match p {
            Ok(path) => path
                .file_name()
                .into_string()
                .map_err(|_| DateFSError::InvalidFolder(path.path().into_os_string()))?
                .parse::<u32>()
                .map_err(|_| DateFSError::InvalidFolder(path.path().into_os_string())),
            Err(e) => Err(DateFSError::OSError(e)),
        })
        .collect::<Result<Vec<u32>, DateFSError>>()?;
    return Ok(folder_names.into_iter().filter(|name| *name < bound).max());
}

fn previous_year(base_path: &Path, year: u32) -> Result<Option<u32>, DateFSError> {
    return previous_before_int(base_path, year);
}

fn previous_month(base_path: &Path, year: u32, month: u32) -> Result<Option<u32>, DateFSError> {
    let mut new_base = base_path.to_path_buf();
    new_base.push(year.to_string());
    return previous_before_int(new_base.as_path(), month);
}

fn previous_date(
    base_path: &Path,
    year: u32,
    month: u32,
    day: u32,
) -> Result<Option<u32>, DateFSError> {
    let mut new_base = base_path.to_path_buf();
    new_base.push(year.to_string());
    new_base.push(month.to_string());
    return previous_before_int(new_base.as_path(), day);
}

pub fn date_exists(base_path: &Path, year: u32, month: u32, date: u32) -> bool {
    let mut path_buf = base_path.to_path_buf();
    path_buf.push(year.to_string());
    path_buf.push(month.to_string());
    path_buf.push(date.to_string());
    return path_buf.exists();
}

macro_rules! assert_ok_eq {
    ($left:expr, $right:expr) => {
        match $right {
            Ok(res) => assert_eq!($left, res),
            Err(e) => panic!("e2 return Err: {:?}", e),
        }
    };
}

#[cfg(test)]
mod test {
    use super::*;

    static BASE_PATH: &str = "./test-dirs/";

    #[test]
    fn test1() {
        let base_path = Path::new(BASE_PATH).join("test1");
        assert_ok_eq!(None, previous_year(&base_path, 2016));
        assert_ok_eq!(None, previous_year(&base_path, 2017));
        assert_ok_eq!(Some(2018), previous_year(&base_path, 2019));
        assert_ok_eq!(Some(2018), previous_year(&base_path, 2020));
        assert_ok_eq!(Some(2020), previous_year(&base_path, 2021));
        assert_ok_eq!(Some(2020), previous_year(&base_path, 2022));
        assert_ok_eq!(Some(2020), previous_year(&base_path, 2023));
        assert_ok_eq!(Some(2020), previous_year(&base_path, 2024));
        assert_ok_eq!(Some(2024), previous_year(&base_path, 2025));
        assert_ok_eq!(Some(2025), previous_year(&base_path, 2026));
    }
}
