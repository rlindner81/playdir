use std::{env, fs, process, io};

fn process_args(args: &[String]) -> Option<&String> {
  if args.len() == 1 {
    return Some(&args[0]);
  }
  None
}

fn process_dir(dir_path: &String) -> Result<fs::ReadDir, io::Error> {
  match fs::read_dir(dir_path) {
    Ok(entries) => Ok(entries),
    Err(err) => Err(err)
  }
}

// fn read_plist_entries() -> Result<(), ()> {
//   println!("lala");
//   Ok(())
// }

fn main() {
  let args: Vec<String> = env::args().collect();
  let (program_name, rest_of_args) = args.split_at(1);
  let dir_path = match process_args(rest_of_args) {
    Some(x) => x,
    None => {
      eprintln!("Usage: {} <dir>", program_name[0]);
      process::exit(1);
    }
  };

  let entries = match process_dir(dir_path) {
    Ok(entries) => entries,
    Err(err) => {
      eprintln!("Error reading dir '{}': {}", dir_path, err);
      process::exit(1);
    }
  };

  for entry in entries {
    match entry {
      Ok(entry) => {
        println!("{:?}", entry.path());
      }
      Err(_) => ()
    }
  }

}
