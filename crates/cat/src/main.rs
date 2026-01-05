use std::env;
use std::fs;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {

    let args: Vec<String> = env::args().collect();

    let mut input_files : Vec<String> = vec![];

    // Store the input files into a vector
    if args.len() > 1 {
        for i in 1..args.len() {
            input_files.push(args[i].clone());
        }
    }

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

    for data in contents.iter() {
        println!("{data}");
    }

    Ok(())
}
