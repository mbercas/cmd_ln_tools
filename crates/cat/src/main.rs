use std::env;
use std::fs;
use std::error::Error;
use clap::{command, Arg, ArgAction};
use std::io::{self, Write};

/// Takes a vector of strings, where each string may have one or
/// more EOL characters and separaes the lines to return a vector
/// of single line strigns.
fn unwrap_lines(data:Vec<String>) -> Vec<String> {
    
    let mut output_data = vec![];

    for lines in data.iter() {
        output_data.extend(lines.split('\n').map(String::from).collect::<Vec<String>>());
    }

    output_data
}

fn append_line_number(data:Vec<String>) -> Vec<String> {
    let input_data = unwrap_lines(data);
    
    // Calculate the right alignment of the number column
    let nc = 1 + (input_data.len() % 10);

    input_data.iter()
        .enumerate()
        .map(|(index, line)| format!("{:>nc$} {}", index+1, line))
        .collect()
}

fn remove_consecutive_empty_lines(data:Vec<String>) -> Vec<String> {
    let input_data = unwrap_lines(data);
    let mut empty_line_counter = 0;
    
    input_data.into_iter()
        .filter(|line| {
            if line.is_empty() {
                empty_line_counter += 1;
            } else {
                empty_line_counter = 0;
            }
            empty_line_counter < 2
        }
        ).collect()
    
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


fn main() -> Result<(), Box<dyn Error>> {

    let matches = command!()
        .about("Concatenate FILE(s) to standard output")
        .arg(Arg::new("FILE").action(ArgAction::Append))
        .arg(Arg::new("numbers")
            .short('n')
            .long("numbers")
            .action(ArgAction::SetTrue)
            .help("Prepends line numbers to the output"))
        .arg(Arg::new("squeeze-blank")
            .short('s')
            .long("squeeze-blank")
            .action(ArgAction::SetTrue)
            .help("Remove consecutive empty lines"))
        .get_matches();
  
    let input_files = matches
        .get_many::<String>("FILE")
        .unwrap_or_default()
        .map(|v| v.as_str())
        .collect::<Vec<_>>();

    // Iterate over the valid input files and load the contents into memory
    let mut contents : Vec<String> = vec![];
    let mut errors = vec![];

    for fname in input_files.iter() {

        match fs::read_to_string(fname) {
            Ok(data) => contents.push(data),
            Err(e) => {
                eprintln!("Error reading file {fname}: {e}");
                errors.push((fname, e));
            },
        }
    }
    if matches.get_flag("squeeze-blank") {
        contents = remove_consecutive_empty_lines(contents);
    }
    if matches.get_flag("numbers") {
        contents =  append_line_number(contents);
    }
    
    print_output(contents);
    
    Ok(())
}



#[cfg(test)]
mod cat_tests {
    use super::*;

    fn generate_test_vector(n_lines: usize, x_factor: usize) -> Vec<String> {
        let mut lines = vec![];
        match x_factor {
            1 => {
                for i in 1..=n_lines {
                    lines.push(format!("Line {}", i).to_owned());
                }
                },
            2 => {        
                for i in 1..=n_lines {       
                    lines.push(format!("Line {}\nLine {}", x_factor*i-1, x_factor*i).to_owned());
                }
                },
            _ => panic!("Use a valid x_factor"),
        }
            
        
        lines
    }

    #[test]
    fn append_line_number_check_returned_size() {
        let mut orig_lines = vec![];
        const N : usize = 100;
        let mut xfactor = 1;
        orig_lines = generate_test_vector(N, xfactor);

        let mod_lines = append_line_number(orig_lines);
        assert_eq!(N*xfactor, mod_lines.len());

        xfactor = 2;
        orig_lines = generate_test_vector(N, xfactor);

        let mod_lines = append_line_number(orig_lines);
        assert_eq!(N*xfactor, mod_lines.len());
   }
    
    #[test]
    fn append_line_number_check_indexing() {
        let mut orig_lines = vec![];
        const N : usize = 100;
        let xfactor = 1;
        orig_lines = generate_test_vector(N, xfactor);

        let mod_lines = append_line_number(orig_lines);
        for (i, line) in mod_lines.iter().enumerate() {
            let mut tokens = line.split_whitespace();
            assert_eq!(i+1, tokens.next().unwrap().parse::<usize>().unwrap())
        }
    }

    #[test]
    fn remove_consecutive_empty_lines_empty_input() {
        let orig_lines = vec![];
        let mod_lines = remove_consecutive_empty_lines(orig_lines);
        assert_eq!(0, mod_lines.len());

        let mut orig_lines = vec![];
        for _ in 0..100 {
            orig_lines.push(String::from(""));
        }

        let mod_lines = remove_consecutive_empty_lines(orig_lines);
        assert_eq!(1, mod_lines.len()); // one line of the repeated chunk remains
        
    }
    
    #[test]
    fn remove_consecutive_empty_lines_check_count() {
        const N : usize = 100;
        let xfactor = 1;
        let mut orig_lines = generate_test_vector(N, xfactor);

        for _ in 0..100 {
            orig_lines.push(String::from(""));
        }

        assert_eq!(N*xfactor+100, orig_lines.len());
        let mod_lines = remove_consecutive_empty_lines(orig_lines);
        assert_eq!(N*xfactor+1, mod_lines.len());
    }
}
