use std::fs;

use std::ffi::{OsStr, OsString};
use std::io::Error;
use std::path::{Path, PathBuf};
use time::Date;

#[derive(Debug)]
pub enum DateFSError {
    BaseMissing,
    EmptyDir,
    InvalidDate(i32, u8, u8),
    InvalidFile(OsString),
    OSError(Error),
}

// The previous date that occurs before the current one in the file system.
// Also does checks to make sure that all files are valid.
// Returns None if there was no previous date
pub fn previous_before(base_path: &Path, bound: Date) -> Result<Option<Date>, DateFSError> {
    let paths = fs::read_dir(base_path).map_err(|_| DateFSError::BaseMissing)?;
    let file_dates = paths
        .map(|p|
            match p {
                Ok(path) => Ok(extract_date(path.path().as_path())?),
                Err(e) => Err(DateFSError::OSError(e)),
            })
            .collect::<Result<Vec<Date>, DateFSError>>()?;

    let max_date = file_dates.into_iter().filter(|d| *d < bound).max();
    return Ok(max_date);
}

fn extract_date(path: &Path) -> Result<Date, DateFSError> {
    let err = DateFSError::InvalidFile(path.as_os_str().into());

    if path.extension() == Some(&OsStr::new("md")) {
        if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
            let format = time::format_description::well_known::Iso8601::DATE;
            let date = Date::parse(stem, &format).map_err(|_| err)?;
            return Ok(date);
        }
    }

    return Err(err);
}

pub fn construct_path(base_path: &Path, date: Date) -> PathBuf {
    let format = time::format_description::well_known::Iso8601::DATE;
    let file_name = format!("{}.md", date.format(&format).unwrap());
    return base_path.join(file_name);
}

#[cfg(test)]
mod test {
    use super::*;
    use time::Month;

    macro_rules! assert_ok_eq {
        ($left:expr, $right:expr) => {
            match $right {
                Ok(res) => assert_eq!($left, res),
                Err(e) => panic!("e2 return Err: {:?}", e),
            }
        };
    }

    fn date(year: i32, month: u8, date: u8) -> Date {
        let month_enum = Month::January.nth_next(month - 1);
        return Date::from_calendar_date(year, month_enum, date).unwrap();
    }

    static BASE_PATH: &str = "./test-dirs/";

    #[test]
    fn test_previous() {
        let base_path = Path::new(BASE_PATH).join("test_previous");

        assert_ok_eq!(None, previous_before(&base_path, date(2017, 1, 1)));
        assert_ok_eq!(None, previous_before(&base_path, date(2018, 1, 1)));
        assert_ok_eq!(None, previous_before(&base_path, date(2018, 3, 28)));
        assert_ok_eq!(None, previous_before(&base_path, date(2018, 3, 29)));
        assert_ok_eq!(Some(date(2018, 3, 29)), previous_before(&base_path, date(2018, 3, 30)));
        assert_ok_eq!(Some(date(2018, 3, 29)), previous_before(&base_path, date(2018, 4, 1)));
        assert_ok_eq!(Some(date(2018, 3, 29)), previous_before(&base_path, date(2018, 4, 28)));
        assert_ok_eq!(Some(date(2018, 3, 29)), previous_before(&base_path, date(2018, 4, 29)));
        assert_ok_eq!(Some(date(2018, 4, 29)), previous_before(&base_path, date(2018, 4, 30)));
        assert_ok_eq!(Some(date(2018, 4, 30)), previous_before(&base_path, date(2018, 5, 1)));
        assert_ok_eq!(Some(date(2018, 4, 30)), previous_before(&base_path, date(2018, 5, 2)));
        assert_ok_eq!(Some(date(2018, 4, 30)), previous_before(&base_path, date(2018, 6, 2)));
        assert_ok_eq!(Some(date(2018, 4, 30)), previous_before(&base_path, date(2018, 12, 31)));
        assert_ok_eq!(Some(date(2018, 4, 30)), previous_before(&base_path, date(2019, 1, 1)));
        assert_ok_eq!(Some(date(2019, 1, 1)),  previous_before(&base_path, date(2019, 1, 2)));
        assert_ok_eq!(Some(date(2019, 1, 1)),  previous_before(&base_path, date(2019, 1, 3)));
        assert_ok_eq!(Some(date(2019, 1, 1)),  previous_before(&base_path, date(3019, 1, 3)));
    }

    #[test]
    fn test_exists() {
        let base_path = Path::new(BASE_PATH).join("test_previous");

        fn date_exists(base_path: &Path, date: Date) -> bool {
            return construct_path(base_path, date).exists();
        }

        assert!(!date_exists(&base_path, date(2018, 3, 28)));
        assert!(date_exists(&base_path, date(2018, 3, 29)));
        assert!(!date_exists(&base_path, date(2018, 3, 30)));

        assert!(!date_exists(&base_path, date(2018, 4, 28)));
        assert!(date_exists(&base_path, date(2018, 4, 29)));
        assert!(date_exists(&base_path, date(2018, 4, 30)));
        assert!(!date_exists(&base_path, date(2018, 5, 1)));

        assert!(!date_exists(&base_path, date(2018, 12, 31)));
        assert!(date_exists(&base_path, date(2019, 1, 1)));
        assert!(!date_exists(&base_path, date(2019, 1, 2)));
    }

    #[test]
    fn test_missing_md() {
        let base_path = Path::new(BASE_PATH).join("test_missing_md");
        assert!(previous_before(&base_path, date(2018, 3, 20)).is_err());
        assert!(previous_before(&base_path, date(2019, 3, 20)).is_err());
    }

    #[test]
    fn test_invalid_format() {
        let base_path = Path::new(BASE_PATH).join("test_invalid_format");
        println!("{:?}", previous_before(&base_path, date(2038, 3, 20)));
        assert!(previous_before(&base_path, date(2038, 3, 20)).is_err());
    }
}
