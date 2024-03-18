use std::{
    env,
    fs::{self, ReadDir},
    path::{Path, PathBuf},
    process::Command,
    str::FromStr,
};

use tokio::task::JoinSet;

#[tokio::main]
async fn main() {
    let root_path = PathBuf::from_str(&env::args().nth(1).expect("No path specified"))
        .expect("Not a valid path");
    let root_folder =
        fs::read_dir(root_path.clone()).expect(&format!("Could not read folder {:?}", root_path));

    println!(
        "Average bitrate: {} kb/s",
        get_average_bitrate(root_folder).await
    )
}

async fn get_average_bitrate(folder: ReadDir) -> u32 {
    let mut set: JoinSet<u32> = JoinSet::new();
    for entry in folder.filter_map(|entry| entry.ok()) {
        set.spawn(async move { return grep(ffmpeg(&entry.path())).parse().unwrap() });
    }

    let mut sum = 0;
    let mut count = 0;

    while !set.is_empty() {
        sum += set.join_next().await.unwrap().unwrap();
        count += 1;
    }

    sum / count
}

fn ffmpeg(path: &Path) -> String {
    String::from_utf8(
        Command::new("ffmpeg")
            .arg("-i")
            .arg(path)
            .output()
            .unwrap()
            .stderr,
    )
    .expect("ffmpeg did not return valid UTF-8")
}

fn grep(input: String) -> String {
    input
        .lines()
        .filter(|line| line.contains("bitrate: "))
        .map(|line| line.split("bitrate: ").nth(1).unwrap().replace(" kb/s", ""))
        .collect::<Vec<_>>()
        .join("")
}
