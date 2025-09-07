use crate::config::Config;
use crate::datefs::{DateFSError, construct_path, format_date, previous_before};
use std::collections::HashMap;
use std::fs::{File, read_to_string};
use std::io::Write;
use std::path::Path;
use time::Date;

// Stores a representation of the file in an easier-to-access format
type FileRepr = HashMap<String, String>;

pub fn write_template(config: &Config, date: Date) -> Result<(), DateFSError> {
    let base_path = Path::new(&config.root_path);
    let old_data =
        match previous_before(base_path, date)? {
            Some(old_date) => {
                let old_path = construct_path(base_path, old_date);
                Some(parse_template(old_path.as_path())?)
            },
            None => None,
        };

    let note_file = construct_path(base_path, date);
    let mut f = File::create_new(&note_file)
        .map_err(|_| DateFSError::InvalidFile(note_file.into_os_string()))?;
    let title = format!("# {}\n\n", format_date(date));
    f.write_all(title.as_bytes())
        .map_err(|e| DateFSError::OSError(e))?;

    for section in config.sections.iter() {
        let section_title = format!("## {}\n", section.title);
        f.write_all(section_title.as_bytes())
            .map_err(|e| DateFSError::OSError(e))?;

        let section_body = 
            if section.persist {
                match old_data {
                    Some(ref fr) => retrieve_section(&fr, &section.title),
                    None => "\n",
                }
            } else {
                "\n"
            };

        f.write_all(section_body.as_bytes())
            .map_err(|e| DateFSError::OSError(e))?;
    }

    return Ok(());
}

fn retrieve_section<'a, 'b>(fr: &'a FileRepr, title: &'b str) -> &'a str {
    let data = fr.get(title).map(|s| s.as_str());
    return data.unwrap_or("\n");
}

fn parse_template(path: &Path) -> Result<FileRepr, DateFSError> {
    let contents = read_to_string(path)
        .map_err(|_| DateFSError::InvalidFile(path.to_path_buf().into_os_string()))?;

    let mut sections = FileRepr::new();
    let mut cur_section = None;
    let mut cur_section_contents = String::from("");
    for line in contents.split("\n") {
        if line.starts_with("## ") {
            // store the old section
            if let Some(section) = cur_section {
                sections.insert(String::from(section), cur_section_contents);
            }
            cur_section = Some(line);
            cur_section_contents = String::from("");
        } else if cur_section != None {
            if !cur_section_contents.is_empty() {
                cur_section_contents.push_str("\n");
            }
            cur_section_contents.push_str(line);
        }
    }

    return Ok(sections);
}
