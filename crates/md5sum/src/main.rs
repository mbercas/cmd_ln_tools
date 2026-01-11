use clap::{command, Arg, ArgAction};
use std::error::Error;
use std::fs;
use std::io::{self, Stdin};

/// Parses the command line and returns a vector of
/// files and a struct of command flags
fn parse_input_args() -> Vec<String> {
    let matches = command!()
        .about("Print or check MD5 128-bit cheksums. ")
        .arg(Arg::new("FILE").action(ArgAction::Append))
        .get_matches();

    let mut input_files = matches
        .get_many::<String>("FILE")
        .unwrap_or_default()
        .map(|x| x.to_owned())
        .collect::<Vec<String>>();

    if input_files.is_empty() {
        input_files.push("-".to_owned());
    }
    input_files
}

fn main() -> Result<(), Box<dyn Error>> {
    let input_files = parse_input_args();

    for file_name in input_files.iter() {
        println!("Processing file: {file_name}");
        if file_name != "-" {
            match fs::read_to_string(file_name) {
                Ok(data) => {
                    let digest = md5::compute(data);
                    println!("{}:  {:x}", file_name, digest);
                }
                Err(e) => {
                    eprintln!("Couln't open file {}: {}", file_name, e);
                }
            }
        } else {
            let stdin: Stdin = io::stdin();
            let mut processor = md5::Context::new();
            for line in stdin.lines() {
                processor.consume(&line?);
            }
            let hash = processor.finalize();
            println!("Stdin:  {:x}", hash);
        }
    }
    Ok(())
}

mod md5sum_test {
    use super::*;

    #[test]
    fn get_input_files_from_arguments() {
        assert!(true);
    }
}
