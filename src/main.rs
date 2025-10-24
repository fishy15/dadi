mod config;
mod datefs;
mod template;

use chrono::{Datelike, Local};
use clap::{Parser, Subcommand};
use config::{Config, read_config};
use datefs::construct_path;
use std::io::Write;
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

fn collate_results(config: &Config, days: u32) {
    let today = get_today(config);
    let mut current_date = today;
    let mut date_info = vec![];
    for _ in 0..days {
        current_date = current_date.previous_day().unwrap();
        let day_contents = template::parse_day(&config, current_date).unwrap();
        if let Some(fr) = day_contents {
            date_info.push((current_date, fr));
        }
    }

    let data: String = date_info
        .into_iter()
        .rev()
        .map(|(date, fr)| {
            let date_header = format!("# {}\n\n", datefs::format_date(date));
            let contents: String = config
                .sections
                .iter()
                .map(|s| {
                    if s.collate {
                        let header = format!("## {}\n", s.title);
                        let empty = String::from("");
                        let info = fr.get(&s.title).unwrap_or(&empty);
                        return header + &info;
                    } else {
                        return String::from("");
                    }
                })
                .collect();
            return date_header + &contents;
        })
        .collect();

    let editor = std::env::var("EDITOR").expect("$EDITOR is not set");
    if editor == "nvim" {
        let mut process = std::process::Command::new(editor)
            .args(["-R", "+set filetype=markdown", "-"])
            .stdin(std::process::Stdio::piped())
            .spawn()
            .expect("failed to start editor");

        let mut editor_stdin = process
            .stdin
            .as_ref()
            .expect("failed to open editor's stdin");
        editor_stdin
            .write_all(data.as_bytes())
            .expect("failed to pipe data to editor");

        process
            .wait()
            .expect("editor returned with non-zero status");
    } else {
        println!("{} is not supported yet", editor);
    }
}

fn main() {
    let config = read_config().unwrap();
    let cli = Cli::parse();
    match cli.command {
        None | Some(CliSubcommand::Today) => open_today_entry(&config),
        Some(CliSubcommand::Collate { days }) => collate_results(&config, days),
    }
}
