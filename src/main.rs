use std::{env, fs, process};
use std::collections::HashMap;
use plist::{Value};

fn process_args(args: &[String]) -> Option<&String> {
  if args.len() == 1 {
    return Some(&args[0]);
  }
  None
}

fn fill_mkv_files(output: &mut Vec<String>, dir_path: &String) {
  let entries = fs::read_dir(dir_path).unwrap();
  for entry in entries {
    let filepath = entry.unwrap().path();
    if filepath.extension().unwrap() == "mkv" {
      output.push(filepath.to_string_lossy().parse().unwrap());
    }
  }
  output.sort()
}

fn fill_recent_played_media(output: &mut HashMap<String, f64>) {
  let env_home = env::var("HOME").unwrap();
  let vlc_preferences_path = format!("{}/Library/Preferences/org.videolan.vlc.plist", env_home);
  let vlc_preferences = Value::from_file(vlc_preferences_path).unwrap();
  let recent_played_media_dict = vlc_preferences
    .as_dictionary().unwrap()
    .get("recentlyPlayedMedia").unwrap()
    .as_dictionary().unwrap();
  for (key, value) in recent_played_media_dict {
    let filepath = key.trim_start_matches("file://").parse().unwrap();
    output.insert(filepath, value.as_signed_integer().unwrap() as f64);
  }
}

fn fill_mkv_files_playtime(output: &mut HashMap<String, f64>, mkv_files: Vec<String>) {
  for mkv_file in mkv_files {
    let container = matroska::open(&mkv_file).unwrap();
    let duration = container.info.duration.unwrap().as_secs();
    output.insert(mkv_file, duration as f64);
  }
}

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

  let mut mkv_files: Vec<String> = Vec::new();
  fill_mkv_files(&mut mkv_files, dir_path);

  let mut mkv_files_playtime: HashMap<String, f64> = HashMap::new();
  fill_mkv_files_playtime(&mut mkv_files_playtime, mkv_files);

  let mut recent_played_media: HashMap<String, f64> = HashMap::new();
  fill_recent_played_media(&mut recent_played_media);

  println!("break here");

}
