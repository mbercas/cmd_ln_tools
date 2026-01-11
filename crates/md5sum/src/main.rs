use clap::{command, Arg, ArgAction};
use std::error::Error;
use std::fs;
use std::io::{self, Stdin};

/// Struct to hold the command line status
///
#[derive(Debug)]
struct CommandLineFlags {
    binary: bool,
    tag: bool,
}

/// Parses the command line and returns a vector of
/// files and a struct of command flags
fn parse_input_args() -> (Vec<String>, CommandLineFlags) {
    let matches = command!()
        .about("Print or check MD5 128-bit cheksums. ")
        .arg(Arg::new("FILE").action(ArgAction::Append))
        .arg(
            Arg::new("text")
                .short('t')
                .long("text")
                .action(ArgAction::SetTrue)
                .help("read in text mode (default)"),
        )
        .arg(
            Arg::new("tag")
                .long("tag")
                .action(ArgAction::SetTrue)
                .help("Create a BSD style checksum"),
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
        binary: false,
        tag: matches.get_flag("tag"),
    };

    (input_files, flags)
}

/// Retuns a formatted string with the
/// file name a * if is binary input and the hash,
/// if tags is to True generated BDS style output
fn format_output_line(file_name: &str, is_binary: bool, tag: bool, digest: &md5::Digest) -> String {
    let binary_char = if is_binary { "*" } else { " " }.to_string();
    if tag {
        format!("MD5 ({}) = {:x}", file_name, digest)
    } else {
        format!("{:x} {}{}", digest, binary_char, file_name)
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let cmd_line = parse_input_args();
    let input_files = cmd_line.0;
    let flags = cmd_line.1;

    for file_name in input_files.iter() {
        let mut processor = md5::Context::new();
        if file_name != "-" {
            match fs::read_to_string(file_name) {
                Ok(data) => {
                    processor.consume(data);
                    let output = format_output_line(
                        file_name,
                        flags.binary,
                        flags.tag,
                        &processor.finalize(),
                    );
                    println!("{output}");
                }
                Err(e) => {
                    eprintln!("Couln't open file {}: {}", file_name, e);
                }
            }
        } else {
            let stdin: Stdin = io::stdin();
            for line in stdin.lines() {
                processor.consume(&line?);
            }
            let output =
                format_output_line(file_name, flags.binary, flags.tag, &processor.finalize());
            println!("{output}");
        }
    }
    Ok(())
}

#[cfg(test)]
mod md5sum_test {
    use super::*;

    #[test]
    fn format_output_line_non_bsd() {
        let mut context = md5::Context::new();
        context.consume("123456");
        let digest = context.finalize();
        let test1 = format_output_line("file_name", false, false, &digest);
        assert_eq!(test1, format!("{:x}  {}", &digest, "file_name"));

        let test2 = format_output_line("file_name", true, false, &digest);
        assert_eq!(test2, format!("{:x} *{}", &digest, "file_name"));
    }

    #[test]
    fn format_output_line_bsd() {
        let mut context = md5::Context::new();
        context.consume("123456");
        let digest = context.finalize();
        let test1 = format_output_line("file_name", false, true, &digest);
        assert_eq!(test1, format!("MD5 ({}) = {:x}", "file_name", &digest));

        let test2 = format_output_line("file_name", true, true, &digest);
        assert_eq!(test2, format!("MD5 ({}) = {:x}", "file_name", &digest));
    }
}
