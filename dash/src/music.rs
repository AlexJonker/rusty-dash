use slint::ComponentHandle;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;

use lofty::prelude::*;
use lofty::probe::Probe;

use playback_rs::{Player, Song};

use crate::AppWindow;

fn collect_paths(root: &str) -> Vec<PathBuf> {
    let mut paths = Vec::new();

    for artist_entry in std::fs::read_dir(root).unwrap() {
        let artist_path = artist_entry.unwrap().path();

        for album_entry in std::fs::read_dir(&artist_path).unwrap() {
            let album_path = album_entry.unwrap().path();

            for song_entry in std::fs::read_dir(&album_path).unwrap() {
                paths.push(song_entry.unwrap().path());
            }
        }
    }

    paths
}

fn format_time(secs: u64) -> String {
    format!("{}:{:02}", secs / 60, secs % 60)
}

pub struct MusicController;

impl MusicController {
    pub fn new(ui: &AppWindow) -> Self {
        let ui_handle = ui.as_weak();
        ui.on_play_pause(move || {
            let ui = ui_handle.unwrap();
            ui.set_is_playing(!ui.get_is_playing());
        });

        let ui_handle = ui.as_weak();
        ui.on_toggle_loop(move || {
            let ui = ui_handle.unwrap();
            ui.set_is_loop_enabled(!ui.get_is_loop_enabled());
        });

        let ui_handle = ui.as_weak();
        ui.on_toggle_shuffle(move || {
            let ui = ui_handle.unwrap();
            ui.set_is_shuffle_enabled(!ui.get_is_shuffle_enabled());
        });

        let ui_handle = ui.as_weak();
        ui.on_next_track(move || {
            let _ui = ui_handle.unwrap();
        });

        let ui_handle = ui.as_weak();
        ui.on_previous_track(move || {
            let _ui = ui_handle.unwrap();
        });

        ui.on_set_progress(move |progress| {
            let clamped = progress.clamp(0.0, 1.0);
            println!("Set progress to {}", clamped);
        });

        Self
    }

    pub fn play_music(ui: &AppWindow) {
        let ui_weak = ui.as_weak();

        thread::spawn(move || {
            let player = Player::new(None).unwrap();
            let paths = collect_paths("/storage/music");

            for path in &paths {
                while player.has_next_song() {
                    thread::sleep(Duration::from_millis(100));
                }

                let song = Song::from_file(path, None).unwrap();
                player.play_song_next(&song, None).unwrap();

                while player.has_next_song() {
                    thread::sleep(Duration::from_millis(100));
                }

                let tagged_file = Probe::open(path)
                    .expect("ERROR: Bad path")
                    .read()
                    .expect("ERROR: Failed to read file");

                let duration = tagged_file.properties().duration().as_secs();
                let tag = match tagged_file.primary_tag() {
                    Some(primary_tag) => primary_tag,
                    None => tagged_file.first_tag().expect("ERROR: No tags found!"),
                };

                let title = tag.title().as_deref().unwrap_or("None").to_string();
                let artist = tag.artist().as_deref().unwrap_or("None").to_string();
                let album = tag.album().as_deref().unwrap_or("None").to_string();

                let ui_weak_inner = ui_weak.clone();
                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(ui) = ui_weak_inner.upgrade() {
                        ui.set_track_title(title.into());
                        ui.set_artist_name(artist.into());
                        ui.set_album_name(album.into());
                        ui.set_total_time(format_time(duration).into());
                    }
                });

                loop {
                    if let Some((elapsed, _total)) = player.get_playback_position() {
                        let secs = elapsed.as_secs();
                        let current = format_time(secs);
                        let progress = if duration > 0 {
                            secs as f32 / duration as f32
                        } else {
                            0.0
                        };

                        let ui_weak_inner = ui_weak.clone();
                        let _ = slint::invoke_from_event_loop(move || {
                            if let Some(ui) = ui_weak_inner.upgrade() {
                                ui.set_current_time(current.into());
                                ui.set_progress(progress);
                            }
                        });
                    }

                    if !player.has_current_song() {
                        break;
                    }

                    thread::sleep(Duration::from_secs(1));
                }
            }
        });
    }
}
