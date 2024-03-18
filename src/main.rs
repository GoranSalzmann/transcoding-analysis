use async_recursion::async_recursion;
use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
    str::FromStr,
    sync::Arc,
};
use tokio::{sync::Mutex, task::JoinSet};
use trie::{translate, Trie};

mod trie;

#[tokio::main]
async fn main() {
    let root_path = PathBuf::from_str(&env::args().nth(1).expect("No path specified"))
        .expect("Not a valid path")
        .canonicalize()
        .unwrap();
    let bitrates = Arc::new(Mutex::new(Trie::new()));

    visit_folder(root_path.clone(), Arc::clone(&bitrates)).await;
    print_results(root_path, bitrates).await;
}

async fn print_results(root_path: PathBuf, bitrates: Arc<Mutex<Trie<(u32, u32)>>>) {
    let lock = bitrates.lock().await;
    let trie = lock.get(translate(root_path.to_string_lossy().to_string(), '\\'));
    if trie.is_none() {
        println!("No video files found.");
    } else {
        print!(
            "{}: ",
            root_path.file_name().unwrap().to_string_lossy().to_string()
        );
        trie.unwrap().pretty_print(
            |o| {
                o.map_or("Untracked".to_string(), |(sum, count)| {
                    format!("{} kb/s", sum.checked_div(*count).unwrap_or(0))
                })
            },
            0,
        );
    }
}

#[async_recursion]
async fn visit_folder(path: PathBuf, bitrates: Arc<Mutex<Trie<(u32, u32)>>>) -> (u32, u32) {
    let mut sum = 0;
    let mut count = 0;

    let folder = fs::read_dir(path.clone()).expect(&format!("Could not read folder {:?}", path));

    let mut set: JoinSet<Option<u32>> = JoinSet::new();
    for entry in folder.filter_map(|entry| entry.ok()) {
        if entry.metadata().unwrap().is_file() {
            set.spawn(async move { return grep(ffmpeg(&entry.path())).parse().ok() });
        } else {
            let (sub_sum, sub_count) = visit_folder(entry.path(), Arc::clone(&bitrates)).await;
            sum += sub_sum;
            count += sub_count;
        }
    }

    while let Some(bitrate) = set.join_next().await.map(|b| b.unwrap()) {
        if let Some(bitrate) = bitrate {
            sum += bitrate;
            count += 1;
        }
    }

    if count != 0 {
        bitrates
            .lock()
            .await
            .set(translate(path.to_string_lossy(), '\\'), (sum, count));
    }

    (sum, count)
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
