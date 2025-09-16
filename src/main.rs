mod config;
mod datefs;
mod template;

use chrono::{Datelike, Local};
use config::read_config;
use datefs::construct_path;
use std::env;
use std::path::Path;
use std::process::Command;
use std::time::Duration;
use template::write_template;
use time::{Date, Month};

fn main() {
    let config = read_config().unwrap();

    let now = Local::now();
    let hour_shift =  Duration::from_secs((config.reset_hours_after_midnight * 60 * 60).into());
    let date = (now - hour_shift).date_naive();

    let year = date.year();
    let month = Month::January.nth_next(date.month0() as u8);
    let day = date.day() as u8;

    let date = Date::from_calendar_date(year, month, day).unwrap();

    write_template(&config, date).unwrap();

    let today_file = construct_path(Path::new(&config.root_path), date);
    let editor = env::var("EDITOR").expect("$EDITOR is not set");

    Command::new(editor)
        .args([today_file])
        .spawn()
        .expect("failed to start editor")
        .wait()
        .expect("editor returned with non-zero status");
}
