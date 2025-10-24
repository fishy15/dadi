mod config;
mod datefs;
mod template;

use chrono::{Datelike, Local};
use clap::{Parser, Subcommand};
use config::{Config, read_config};
use datefs::construct_path;
use std::time::Duration;
use template::write_template;
use time::{Date, Month};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<CliSubcommand>,
}

#[derive(Subcommand)]
enum CliSubcommand {
    Today,
    Collate {
        #[arg(default_value_t = 7)]
        days: u32,
    },
}

fn get_today(config: &Config) -> Date {
    let now = Local::now();
    let hour_shift = Duration::from_secs((config.reset_hours_after_midnight * 60 * 60).into());
    let date = (now - hour_shift).date_naive();

    let year = date.year();
    let month = Month::January.nth_next(date.month0() as u8);
    let day = date.day() as u8;

    return Date::from_calendar_date(year, month, day).unwrap();
}

fn open_today_entry(config: &Config) {
    let date = get_today(config);
    write_template(&config, date).unwrap();

    let today_file = construct_path(std::path::Path::new(&config.root_path), date);
    let editor = std::env::var("EDITOR").expect("$EDITOR is not set");

    std::process::Command::new(editor)
        .args([today_file])
        .spawn()
        .expect("failed to start editor")
        .wait()
        .expect("editor returned with non-zero status");
}

fn collate_results(_config: &Config, _days: u32) {}

fn main() {
    let config = read_config().unwrap();
    let cli = Cli::parse();
    match cli.command {
        None | Some(CliSubcommand::Today) => open_today_entry(&config),
        Some(CliSubcommand::Collate { days }) => collate_results(&config, days),
    }
}
