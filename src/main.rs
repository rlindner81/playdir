use std::env;
use std::fs;
use std::process;

fn process_args(args: &[String]) -> Result<&String, ()> {
    if args.len() == 1 {
        return Ok(&args[0])
    }
    return Err(());
}

/*
fn process_dir(dir_path: String) {
    match fs::read_dir(dir_path) {
        Ok(entries) => {
            println!("Contents of directory '{}':", dir_path);
            for entry in entries {
                if let Ok(entry) = entry {
                    println!("{}", entry.file_name().to_string_lossy());
                }
            }
        }
        Err(err) => {
            eprintln!("Error reading directory '{}': {}", dir_path, err);
            process::exit(1);
        }
    }
}
 */

fn main() {
    let args: Vec<String>  = env::args().collect();
    let (program_name, rest_of_args) = args.split_at(1);
    let dir_path = process_args(rest_of_args);
    if let Err(err) = dir_path {
        eprintln!("Usage: {} <dir>", program_name[0]);
        process::exit(1);
    }

    /*
    let entries = process_dir(dir_path);
    if let Err(err) = entries {
        eprintln!("Error reading directory '{}': {}", dir_path, err);
        process::exit(1);
    }
    */
}
