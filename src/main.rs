use std::time::Duration;
use std::process::Command;
use std::error::Error;
use reqwest;
use serde::Deserialize;
use stopwatch::Stopwatch;
use std::thread;
use tokio;
use std::collections::BTreeMap;
use std::io::{self, Write};

fn clear_screen() {
    // \x1B[2J clears the screen, \x1B[H moves the cursor to the top-left
    print!("\x1B[2J\x1B[H");
    io::stdout().flush().unwrap();
}
#[derive(Deserialize, Debug)]
#[serde(rename_all="camelCase")]
struct LyricResponse {
    name:String,
    artist_name:String,
    synced_lyrics:Option<String>
}

fn check_position() -> f32 {
    let playing_command = Command::new("playerctl").arg("position").output().
        expect("Failed");
    let playing_command = String::from_utf8_lossy(&playing_command.stdout);
    let duration_played:f32 = playing_command.trim().parse().unwrap();
    return duration_played;
}

fn check_playing() -> bool {
    let playing_command = Command::new("playerctl").arg("status").output().
        expect("Failed");
    let playing_command = String::from_utf8_lossy(&playing_command.stdout);
    if playing_command=="Playing" {
        return true;
    }
    return false;
}

fn print_lyrics(lyrics:String) {
    let mut synced_lines = BTreeMap::new();
    let lines:Vec<&str> = lyrics.split("\n").collect();
    for line in &lines {
        let split_line:Vec<&str> = line.split("] ").collect();
        let minutes: i32 = split_line[0][1..3].parse().unwrap();
        let seconds: i32 = split_line[0][4..6].parse().unwrap();
        let time = minutes*60 + seconds;
        synced_lines.insert(time, split_line[1]);
    }
    let mut i = 0;
    let mut first = true;
    let duration = check_position();
    // CHECK IF SONG IS PAUSED
    //while !check_playing() {
    //    clear_screen();
    //    println!("Paused");
    //    thread::sleep(Duration::from_secs(1));
    //}
    // ITERATE THROUGH LYRICS TILL YOU FIND THE LYRIC WHICH IS BIGGER IN TIME THAN duration
    clear_screen();
    println!("***");
    while i < synced_lines.len()-1 {
        if let Some((time, lyr)) = synced_lines.iter().nth(i) {
            if let Some((time2, lyr2)) = synced_lines.iter().nth(i+1) {
                if (*time2 as f32) >= duration  {
                    if first {
                        thread::sleep(Duration::from_secs_f32((*time2 as f32) - duration));
                        first = false;
                    } else {
                        thread::sleep(Duration::from_secs_f32((time2 - time) as f32));
                    }
                    clear_screen();
                    println!("{lyr2}");

                }
            }
        }
        i+=1;
    }
}

fn main()-> Result<(), Box<dyn Error>> {
    let output = Command::new("playerctl").arg("metadata").output()
        .expect("Process failed");
    let output = String::from_utf8_lossy(&output.stdout);
    let lines:Vec<_> = output.split("\n").collect();
    if lines.len() <2 { //PLAYERCTL DOESN'T FIND ANYTHING PLAYING / DOESN'T EXIST
        println!("Playerctl didn't find any song playing");
        return Ok(());
    }
    let title = lines[1];
    let title = title.split("               ")
                     .collect::<Vec<_>>()[1];
    let artist = lines[3].split("              ")
                     .collect::<Vec<_>>()[1];
    //MAKING REQUEST TO lrclib.net with title of the song
    let client = reqwest::blocking::Client::new();
    let rootapi = "https://lrclib.net/api";
    let title_slog = title.replace(" ", "+");
    let artist_slog = artist.replace(" ", "+");
    let url = format!("{rootapi}/search?track_name={title_slog}&artist_name={artist_slog}");
    println!("Getting lyrics for {}...\nurl is {url}", title);
    let response = client.get(url)
                .header("User-Agent", "Lyricist/1.0").send()?;
    if response.status().is_success() {
        let lyrics: Vec<LyricResponse> = response.json()?;
        if lyrics.is_empty() {
            println!("This song wasn't found on lrclib.net databases");
            return Ok(()); 
        }
        if let Some(synced) = &lyrics[0].synced_lyrics {
            print_lyrics(synced.to_string());
        } else {
            println!("Time Synced lyrics for this song don't exist yet");
            //TODO SWITCH TO STATIC LYRICS
        }
    } else {
        println!("The response failed");
    }
    Ok(())
}
