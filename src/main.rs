use plist::Value;
use std::collections::BTreeMap;
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter, Write};
use std::path::Path;
use std::process::Command;
use std::{env, fs, process};

const VLC_PATH: &str = "vlc";
const STATE_JSON_FILE: &str = "state.json";
const UNKNOWN_DURATION: f64 = -1.0;
const PLAYTIME_THRESHOLD: f64 = 0.85;

type VideoTimes = BTreeMap<String, f64>;

fn read_video_times_from_file<P: AsRef<Path>>(path: P) -> Result<VideoTimes, Box<dyn Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let result = serde_json::from_reader(reader)?;
    Ok(result)
}

fn write_video_times_to_file<P: AsRef<Path>>(
    path: P,
    data: &VideoTimes,
) -> Result<(), Box<dyn Error>> {
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);
    serde_json::to_writer_pretty(&mut writer, data)?;
    writer.flush()?;
    Ok(())
}

fn process_args(args: &[String]) -> Option<&String> {
    if args.len() == 1 {
        return Some(&args[0]);
    }
    None
}

fn read_duration_from_video_file<P: AsRef<Path>>(path: P) -> Result<f64, Box<dyn Error>> {
    let container = matroska::open(&path)?;
    let duration = match container.info.duration {
        Some(duration) => duration.as_secs() as f64,
        _ => UNKNOWN_DURATION,
    };
    Ok(duration)
}

fn fill_video_times_from_dir(
    video_times: &mut VideoTimes,
    dir_path: &String,
) -> Result<(), Box<dyn Error>> {
    let mut video_files: Vec<String> = Vec::new();
    let entries = fs::read_dir(dir_path)?;
    for entry in entries {
        let filepath = entry?.path();
        if filepath.extension() == Some("mkv".as_ref()) {
            video_files.push(filepath.to_string_lossy().parse()?);
        }
    }
    video_files.sort();
    for video_file in video_files {
        if !video_times.contains_key(&video_file) {
            video_times.insert(video_file, UNKNOWN_DURATION);
        }
    }
    Ok(())
}

fn fill_video_times_from_vlc(
    video_times: &mut VideoTimes,
    dir_path: &String,
) -> Result<(), Box<dyn Error>> {
    let env_home = env::var("HOME")?;
    let vlc_preferences_path = format!("{}/Library/Preferences/org.videolan.vlc.plist", env_home);
    let vlc_preferences = Value::from_file(vlc_preferences_path)?;
    if let Some(recent_played_media_dict) = vlc_preferences.as_dictionary().and_then(|pref_dict| {
        pref_dict
            .get("recentlyPlayedMedia")
            .and_then(|recent_media_value| recent_media_value.as_dictionary())
    }) {
        for (key, value) in recent_played_media_dict {
            let video_file: String = key.trim_start_matches("file://").parse()?;
            if !video_file.starts_with(dir_path) {
                continue;
            }
            if video_times.contains_key(&video_file) {
                let playtime = match value.as_signed_integer() {
                    Some(value) => value as f64,
                    _ => UNKNOWN_DURATION,
                };
                if let Some(target) = video_times.get_mut(&video_file) {
                    if playtime > *target {
                        *target = playtime;
                    }
                };
            }
        }
    };
    Ok(())
}

fn determine_next_video_file(video_times: &VideoTimes, dir_path: &String) -> Option<String> {
    for (key, playtime) in video_times {
        if !key.starts_with(dir_path) {
            continue;
        }
        if *playtime == UNKNOWN_DURATION {
            return Some(key.clone());
        }
        if let Ok(duration) = read_duration_from_video_file(key) {
            let progress = *playtime / duration;
            println!("{:?} has {:?}", key, progress);
            if progress <= PLAYTIME_THRESHOLD {
                return Some(key.clone());
            }
        }
    }
    None
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

    let mut video_times = match read_video_times_from_file(STATE_JSON_FILE) {
        Ok(result) => result,
        _ => BTreeMap::new(),
    };

    fill_video_times_from_dir(&mut video_times, dir_path).unwrap();

    fill_video_times_from_vlc(&mut video_times, dir_path).unwrap();

    write_video_times_to_file(STATE_JSON_FILE, &video_times).unwrap();

    let video_file = match determine_next_video_file(&video_times, dir_path) {
        Some(x) => x,
        None => {
            println!("all videos watched");
            process::exit(0);
        }
    };

    Command::new(VLC_PATH)
        .arg("--no-fullscreen")
        .arg("--playlist-autostart")
        .arg("--start-paused")
        .arg(&video_file)
        .status()
        .unwrap();
}
