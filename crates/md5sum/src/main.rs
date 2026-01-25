use clap::{command, Arg, ArgAction};
use regex::Regex;
use std::error::Error;
use std::fs;
use std::io::{self, BufReader, Stdin};

/// Struct to hold the command line status
///
#[derive(Debug)]
struct CommandLineFlags {
    binary: bool,
    tag: bool,
    zero: bool,
    check: bool,
}

/// Parses the command line and returns a vector of
/// files and a struct of command flags
fn parse_input_args() -> (Vec<String>, CommandLineFlags) {
    let matches = command!()
        .about("Print or check MD5 128-bit cheksums. ")
        .arg(Arg::new("FILE").action(ArgAction::Append))
        .arg(
            Arg::new("binary")
                .short('b')
                .long("binary")
                .action(ArgAction::SetTrue)
                .help("Read in binary mode"),
        )
        .arg(
            Arg::new("check")
                .short('c')
                .long("check")
                .action(ArgAction::SetTrue)
                .help("Read checksums from the file and check them"),
        )
        .arg(
            Arg::new("text")
                .short('t')
                .long("text")
                .action(ArgAction::SetTrue)
                .help("Read in text mode (default)"),
        )
        .arg(
            Arg::new("tag")
                .long("tag")
                .action(ArgAction::SetTrue)
                .help("Create a BSD style checksum"),
        )
        .arg(
            Arg::new("zero")
                .short('z')
                .long("zero")
                .action(ArgAction::SetTrue)
                .help("End each output line with NUL, no newline, and disable file name scaping"),
        )
        .get_matches();

    let mut input_files = matches
        .get_many::<String>("FILE")
        .unwrap_or_default()
        .map(|x| x.to_owned())
        .collect::<Vec<String>>();

    if input_files.is_empty() {
        input_files.push("-".to_owned());
    }

    let flags = CommandLineFlags {
        binary: matches.get_flag("binary"),
        tag: matches.get_flag("tag"),
        zero: matches.get_flag("zero"),
        check: matches.get_flag("check"),
    };

    (input_files, flags)
}

/// Opens the file passed as argument and parses the contents
/// Returns a vector of file names ans expected hashes or error
/// if the format is incorrect or cannot read the input file.
fn parse_check_file(file_name: &str) -> Result<Vec<Md5Record>, Box<dyn Error>> {
    let mut output = vec![];
    match fs::read_to_string(file_name) {
        Ok(data) => {
            for (idx, line) in data.lines().enumerate() {
                match parse_line(line) {
                    Ok(check_line) => {
                        output.push(check_line);
                    }
                    Err(e) => {
                        eprintln!("{}::{} Error parsing line.", file_name, idx + 1)
                    }
                }
            }
            if output.is_empty() {
                Ok(output)
            } else {
                Err(format!("Couldn't parse enties in file {}", file_name).into())
            }
        }
        Err(e) => Err(e.into()),
    }
}

#[derive(Debug, PartialEq)]
struct Md5Record {
    file_name: String,
    binary: bool,
    hash: String,
}

/// Parses a lines with the following pattern
/// 32 char hexadecimal + space + [space|*] + str
fn parse_line(line: &str) -> Result<Md5Record, Box<dyn Error>> {
    let re = Regex::new(r"^([0-9a-fA-F]{32}) ([ |\*])([^\s\*].+)$").unwrap();
    let Some(caps) = re.captures(line) else {
        return Err(format!("Cannot parse {}", line).into());
    };
    let is_binary = if &caps[2] == "*" { true } else { false };
    let output = Md5Record {
        file_name: caps[3].to_owned(),
        binary: is_binary,
        hash: caps[1].to_owned(),
    };

    eprintln!("{:#?}", output);

    Ok(output)
}

// fn check(file_name: &str) -> Result<(), Box<dyn Error>> {
//     match parse_check_file(file_name) {
//         Ok(input_lines) => {
//             for line in input_lines {

//             }
//         }
//     }
//     Ok(())
// }

fn get_md5_record(file_name: &str, flags: &CommandLineFlags) -> Result<Md5Record, Box<dyn Error>> {
    let mut processor = md5::Context::new();
    if !flags.binary {
        match fs::read_to_string(file_name) {
            Ok(data) => {
                processor.consume(data);
            }
            Err(e) => {
                eprintln!("Couldn't open file {}: {}", file_name, e);
            }
        }
    } else {
        match fs::read(file_name) {
            Ok(bytes) => {
                processor.consume(bytes);
            }
            Err(e) => {
                eprintln!("Couldn't open file {} in binary mode: {}", file_name, e);
            }
        }
    }
    Ok(Md5Record {
        file_name: file_name.to_owned(),
        binary: flags.binary,
        hash: format!("{:x}", processor.finalize()),
    })
}

/// Retuns a formatted string with the
/// file name a * if is binary input and the hash,
/// if tags is to True generated BDS style output
fn format_output_line(md5_record: &Md5Record, tag: bool) -> String {
    let binary_char = if md5_record.binary { "*" } else { " " }.to_string();
    if tag {
        format!("MD5 ({}) = {}", md5_record.file_name, md5_record.hash)
    } else {
        format!(
            "{} {}{}",
            md5_record.hash, binary_char, md5_record.file_name
        )
    }
}

fn print_output(input_files: &[String], flags: &CommandLineFlags) -> Result<(), Box<dyn Error>> {
    let mut error_counter = 0;
    let mut output = String::new();
    for file_name in input_files.iter() {
        if file_name != "-" {
            match get_md5_record(&file_name, flags) {
                Ok(md5_record) => {
                    output = format_output_line(&md5_record, flags.tag);
                }
                Err(e) => {
                    eprintln!("Couldn't open file {}: {}", file_name, e);
                    error_counter += 1;
                }
            }
        } else {
            let stdin: Stdin = io::stdin();
            let mut processor = md5::Context::new();
            for line in stdin.lines() {
                processor.consume(&line?);
            }
            let md5_record = Md5Record {
                file_name: file_name.to_owned(),
                binary: flags.tag,
                hash: format!("{:x}", processor.finalize()),
            };
            output = format_output_line(&md5_record, flags.tag);
        }
        // If zero don't print EOL and add NUL
        if flags.zero {
            print! {"{output}\0"};
        } else {
            println!("{output}");
        }
    }

    if error_counter == 0 {
        Ok(())
    } else {
        Err(format!("{} errors detected", error_counter).into())
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let cmd_line = parse_input_args();
    let input_files = cmd_line.0;
    let flags = cmd_line.1;

    if !flags.check {
        let out = print_output(&input_files, &flags);
        out
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod md5sum_test {

    use super::*;

    #[test]
    fn format_output_line_non_bsd() {
        let file_name = "filename";
        let hash = "4e7bb796c99cf98ae40b32b644119c74";
        let mut md5_record = Md5Record {
            file_name: file_name.to_owned(),
            binary: false,
            hash: hash.to_owned(),
        };
        let test1 = format_output_line(&md5_record, false);
        assert_eq!(test1, format!("{}  {}", hash, file_name));

        md5_record.binary = true;
        let test2 = format_output_line(&md5_record, false);
        assert_eq!(test2, format!("{} *{}", hash, file_name));
    }

    #[test]
    fn format_output_line_bsd() {
        let file_name = "filename";
        let hash = "4e7bb796c99cf98ae40b32b644119c74";
        let mut md5_record = Md5Record {
            file_name: file_name.to_owned(),
            binary: false,
            hash: hash.to_owned(),
        };
        let test1 = format_output_line(&md5_record, true);
        assert_eq!(test1, format!("MD5 ({}) = {}", file_name, hash));

        md5_record.binary = true;
        let test2 = format_output_line(&md5_record, true);
        assert_eq!(test2, format!("MD5 ({}) = {}", file_name, hash));
    }

    #[test]
    fn parse_line_correct_line() {
        let line_1 = "4e7bb796c99cf98ae40b32b644119c74  src/main.rs";
        let line_2 = "4e7bb796c99cf98ae40b32b644119c74 *src/main.rs";

        let output_1 = parse_line(line_1).unwrap();
        let output_2 = parse_line(line_2).unwrap();

        assert_eq!("4e7bb796c99cf98ae40b32b644119c74", output_1.hash);
        assert_eq!("4e7bb796c99cf98ae40b32b644119c74", output_2.hash);

        assert!(!output_1.binary);
        assert!(output_2.binary);

        assert_eq!("src/main.rs", output_1.file_name);
        assert_eq!("src/main.rs", output_2.file_name);
    }

    #[test]
    fn parse_line_return_error() {
        let line_1 = "4e9cf98ae40b32b644119c74  src/main.rs"; // hash is too short
        let line_2 = "4e7bb796c99cf98ae40b32b644119c74 &src/main.rs"; // wrong binary symbol
        let line_3 = "4e7bb796c99cf98ae40b32b644119c74   src/main.rs"; // too many spaces
        let line_4 = "4e7bb796c99cf98ae40b32b644119c74  *src/main.rs"; // two spaces and *
        let line_5 = "4e7bb796c99cf98ae40b32b644119c74   "; // missing file name

        assert!(parse_line(line_1).is_err());
        assert!(parse_line(line_2).is_err());
        assert!(parse_line(line_3).is_err());
        assert!(parse_line(line_4).is_err());
        assert!(parse_line(line_5).is_err());
    }
}
