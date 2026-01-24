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
// fn parse_check_file(
//     file_name: &str,
//     flags: &CommandLineFlags,
// ) -> Result<Vec<CheckLine>, Box<dyn Error>> {
//     let mut output = vec![];
//     match fs::read_to_string(file_name) {
//         Ok(data) => {
//             for line in data.lines() {
//                 match parse_line(line) {
//                     Ok(check_line) => {
//                         output.push(check_line);
//                     }
//                     Err(e) => Err(e.into()),
//                 }
//             }
//             Ok(output)
//         }
//         Err(e) => Err(e.into()),
//     }
// }

#[derive(Debug, PartialEq)]
struct CheckLine {
    file_name: String,
    binary: bool,
    hash: String,
}

/// Parses a lines with the following pattern
/// 32 char hexadecima + space + [space|*] + str
fn parse_line(line: &str) -> Result<CheckLine, Box<dyn Error>> {
    let re = Regex::new(r"^([0-9a-fA-F]{32}) ([ |\*])([^\s\*].+)$").unwrap();
    let Some(caps) = re.captures(line) else {
        return Err(format!("Cannot parse {}", line).into());
    };
    let is_binary = if &caps[2] == "*" { true } else { false };
    let output = CheckLine {
        file_name: caps[3].to_owned(),
        binary: is_binary,
        hash: caps[1].to_owned(),
    };

    eprintln!("{:#?}", output);

    Ok(output)
}

/// Retuns a formatted string with the
/// file name a * if is binary input and the hash,
/// if tags is to True generated BDS style output
fn format_output_line(file_name: &str, flags: &CommandLineFlags, digest: &md5::Digest) -> String {
    let binary_char = if flags.binary { "*" } else { " " }.to_string();
    if flags.tag {
        format!("MD5 ({}) = {:x}", file_name, digest)
    } else {
        format!("{:x} {}{}", digest, binary_char, file_name)
    }
}

fn print_output(input_files: &[String], flags: &CommandLineFlags) -> Result<(), Box<dyn Error>> {
    for file_name in input_files.iter() {
        let mut processor = md5::Context::new();
        if file_name != "-" {
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
            let output = format_output_line(file_name, &flags, &processor.finalize());
            // If zero don't print EOL and add NUL
            if flags.zero {
                print! {"{output}\0"};
            } else {
                println!("{output}");
            }
        } else {
            let stdin: Stdin = io::stdin();
            for line in stdin.lines() {
                processor.consume(&line?);
            }
            let output = format_output_line(file_name, &flags, &processor.finalize());
            println!("{output}");
        }
    }
    Ok(())
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
        let mut context = md5::Context::new();
        let mut flags = CommandLineFlags {
            binary: false,
            tag: false,
            zero: false,
            check: false,
        };
        context.consume("123456");
        let digest = context.finalize();
        let test1 = format_output_line("file_name", &flags, &digest);
        assert_eq!(test1, format!("{:x}  {}", &digest, "file_name"));

        flags.binary = true;
        let test2 = format_output_line("file_name", &flags, &digest);
        assert_eq!(test2, format!("{:x} *{}", &digest, "file_name"));
    }

    #[test]
    fn format_output_line_bsd() {
        let mut context = md5::Context::new();
        let mut flags = CommandLineFlags {
            binary: false,
            tag: true,
            zero: false,
            check: false,
        };
        context.consume("123456");
        let digest = context.finalize();
        let test1 = format_output_line("file_name", &flags, &digest);
        assert_eq!(test1, format!("MD5 ({}) = {:x}", "file_name", &digest));

        flags.binary = true;
        let test2 = format_output_line("file_name", &flags, &digest);
        assert_eq!(test2, format!("MD5 ({}) = {:x}", "file_name", &digest));
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
