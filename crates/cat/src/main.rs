use clap::{Arg, ArgAction, command};
use std::env;
use std::error::Error;
use std::fs;
use std::io::{self, Stdin, Write};

/// Takes a strins that may have  one or
/// more EOL characters and separaes the lines to return a vector
/// of single line strigns.
fn unwrap_lines(data: String) -> Vec<String> {
    data.split('\n').map(String::from).collect::<Vec<String>>()
}

/// Appends a line number at the beggining of every line,
/// if the ignore_blanks flag is set, does not add a number to
/// empty lines
fn append_line_number(
    data: Vec<String>,
    ignore_blanks: bool,
    starting_number: usize,
) -> (Vec<String>, usize) {
    // Calculate the right alignment of the number column
    let nc = data.len().to_string().len();

    let mut line_number = starting_number;
    let output = data
        .iter()
        .map(|line| {
            let mut s = String::new();
            if (!ignore_blanks) || (!line.is_empty()) {
                line_number += 1;
                s = format!("{:>nc$} {}", line_number, line);
            } 
            s
        })
        .collect();

    (output, line_number)
}

fn remove_consecutive_empty_lines(
    data: Vec<String>,
    prev_emptylines: usize,
) -> (Vec<String>, usize) {
    let mut empty_line_counter = prev_emptylines;

    let output = data
        .into_iter()
        .filter(|line| {
            if line.is_empty() {
                empty_line_counter += 1;
            } else {
                empty_line_counter = 0;
            }
            empty_line_counter < 2
        })
        .collect();

    (output, empty_line_counter)
}

fn print_output(data: Vec<String>) {
    let stdout = io::stdout();
    let mut handle = io::BufWriter::new(stdout);
    for lines in data.iter() {
        // println!("{data}");
        // TODO: handle the Err
        let _ = writeln!(handle, "{lines}");
    }
}

fn generate_output(
    data: Vec<String>,
    squeeze_blank: bool,
    number_noblank: bool,
    numbers: bool,
    empty_line_counter: usize,
    last_line_number: usize,
) -> (Vec<String>, usize, usize) {
    // Split lines with mulitple end of line into separate vector entries
    let mut output = data;
    let mut empty_line_counter = empty_line_counter;
    let mut last_line_number = last_line_number;
    if squeeze_blank {
        (output, empty_line_counter) = remove_consecutive_empty_lines(output, empty_line_counter);
    }
    if number_noblank {
        (output, last_line_number) = append_line_number(output, true, last_line_number);
    } else if numbers {
        (output, last_line_number) = append_line_number(output, false, last_line_number);
    }

    (output, empty_line_counter, last_line_number)
}

fn main() -> Result<(), Box<dyn Error>> {
    let matches = command!()
        .about("Concatenate FILE(s) to standard output")
        .arg(Arg::new("FILE").action(ArgAction::Append))
        .arg(
            Arg::new("numbers")
                .short('n')
                .long("numbers")
                .action(ArgAction::SetTrue)
                .help("Prepends line numbers to the output"),
        )
        .arg(
            Arg::new("squeeze-blank")
                .short('s')
                .long("squeeze-blank")
                .action(ArgAction::SetTrue)
                .help("Remove consecutive empty lines"),
        )
        .arg(
            Arg::new("number-noblank")
                .short('b')
                .long("number-noblank")
                .action(ArgAction::SetTrue)
                .help("number nonempty output lines, overrides -n"),
        )
        .get_matches();

    let input_files = matches
        .get_many::<String>("FILE")
        .unwrap_or_default()
        .map(|v| v.as_str())
        .collect::<Vec<_>>();

    // Iterate over the valid input files and load the contents into memory
    let mut contents: Vec<String> = vec![];
    let mut errors = vec![];

    // A counter of how many empty lines at the end of the prev. file

    let mut empty_line_counter = 0;
    let mut last_line_number = 0;
    for fname in input_files.iter() {
        if fname != &"-" {
            match fs::read_to_string(fname) {
                Ok(data) => {
                    let output = generate_output(
                        unwrap_lines(data),
                        matches.get_flag("squeeze-blank"),
                        matches.get_flag("number-noblank"),
                        matches.get_flag("numbers"),
                        empty_line_counter,
                        last_line_number,
                    );
                    contents = output.0;
                    empty_line_counter = output.1;
                    last_line_number = output.2;
                    print_output(contents);
                }
                Err(e) => {
                    eprintln!("Error reading file {fname}: {e}");
                    errors.push((fname, e));
                }
            }
        } else {
            let stdin: Stdin = io::stdin();
            for line in stdin.lines() {
                let output = generate_output(
                    unwrap_lines(line.unwrap()),
                    matches.get_flag("squeeze-blank"),
                    matches.get_flag("number-noblank"),
                    matches.get_flag("numbers"),
                    empty_line_counter,
                    last_line_number,
                );
                contents = output.0;
                empty_line_counter = output.1;
                last_line_number = output.2;
                print_output(contents);
            }
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        // Err(Box::new(Err(errors[0].1.into())))
        Err("Errors reported to stderr".into())
    }
}

#[cfg(test)]
mod cat_tests {
    use super::*;

    fn generate_test_string(n_lines: usize) -> String {
        let mut output: String = "".to_owned();
        match n_lines {
            0 => output,
            1 => String::from("Line 1"),
            2.. => {
                for i in 1..n_lines {
                    output.push_str(&format!("Line {}\n", i));
                }
                output.push_str(&format!("Line {}", n_lines - 1));
                output
            }
        }
    }

    fn generate_test_vector(n_lines: usize) -> Vec<String> {
        let mut lines = vec![];
        for i in 1..=n_lines {
            lines.push(format!("Line {}", i).to_owned());
        }
        lines
    }

    #[test]
    fn unwrap_lines_() {
        let unwrapped = unwrap_lines("".to_owned());
        assert_eq!(1, unwrapped.len());

        let n: usize = 3;
        let orig_lines = generate_test_string(n);
        let unwrapped = unwrap_lines(orig_lines);
        assert_eq!(n, unwrapped.len());

        let n = 100;
        let orig_lines = generate_test_string(n);
        let unwrapped = unwrap_lines(orig_lines);
        assert_eq!(n, unwrapped.len());
    }

    #[test]
    fn append_line_number_check_returned_size() {
        let mut orig_lines = vec![];
        const N: usize = 100;
        orig_lines = generate_test_vector(N);

        let mod_lines = append_line_number(orig_lines, false, 0_usize);
        assert_eq!(N, mod_lines.0.len());
        assert_eq!(N, mod_lines.1);
    }

    #[test]
    fn append_line_number_check_indexing() {
        let mut orig_lines = vec![];
        const N: usize = 100;
        orig_lines = generate_test_vector(N);

        let mod_lines = append_line_number(orig_lines, false, 0_usize);
        for (i, line) in mod_lines.0.iter().enumerate() {
            let mut tokens = line.split_whitespace();
            assert_eq!(i + 1, tokens.next().unwrap().parse::<usize>().unwrap())
        }
    }

    #[test]
    fn remove_consecutive_empty_lines_empty_input() {
        let orig_lines = vec![];
        let mod_lines = remove_consecutive_empty_lines(orig_lines, 0_usize);
        assert_eq!(0, mod_lines.0.len());
        assert_eq!(0, mod_lines.1);

        const N: usize = 100;

        let mut orig_lines = vec![];
        for _ in 0..N {
            orig_lines.push(String::from(""));
        }

        let mod_lines = remove_consecutive_empty_lines(orig_lines, 0_usize);
        assert_eq!(1, mod_lines.0.len()); // one line of the repeated chunk remains
        assert_eq!(N, mod_lines.1);
    }

    #[test]
    fn remove_consecutive_empty_lines_check_count() {
        const N: usize = 100;
        let mut orig_lines = generate_test_vector(N);

        for _ in 0..100 {
            orig_lines.push(String::from(""));
        }

        assert_eq!(N + 100, orig_lines.len());
        let mod_lines = remove_consecutive_empty_lines(orig_lines, 0_usize);
        assert_eq!(N + 1, mod_lines.0.len());
        assert_eq!(N, mod_lines.1);

        // calling the function again should not remove more lines, but should
        // report the emoty line at the end

        let mod_lines = remove_consecutive_empty_lines(mod_lines.0, 0_usize);
        assert_eq!(N + 1, mod_lines.0.len());
        assert_eq!(1, mod_lines.1);
    }
}
