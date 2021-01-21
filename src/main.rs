//! A simple scrolling spotifyd status tool
//! Reads metadata using playerctl and buffers it for more robust operation
//! Will not fail if spotifyd is not running, no song is playing,
//! or if playerctl fails to retrieve metadata, but handle that gracefully.

use std::process::Command;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use unicode_segmentation::UnicodeSegmentation;

// Format specification to pass to playerctl. To see the potential options, run
// $ playerctl --player=spotifyd metadata
// and check the playerctl man pages, searching for "Format Strings"
const FORMAT: &str = " {{ artist }}  {{ album }}  {{ title }} - ";

// How long to wait to scroll one letter further
const SCROLL_DELAY: Duration = Duration::from_millis(300);

// How long to wait before polling playerctl for spotifyd's metadata again
// 4 seconds is conservative but very reliable. To test if we can go further down,
// run `spotifyd --no-daemon` in a separate terminal and check for errors like this:
//     Couldn't fetch metadata from spotify: Err(RateLimited(Some(<num>)))
const POLL_DELAY: Duration = Duration::from_secs(4);

// Maximum number of characters in the output
const MAX_WIDTH: usize = 20;

/// Run playerctl to get info about currently playing track
fn playerctl_output() -> String {
    // Check status of spotifyd
    let status = Command::new("playerctl")
        .args(&["--player=spotifyd", "status"])
        .output()
        .expect("Failed to execute playerctl!");

    // Spotifyd is not running
    if status.status.code() != Some(0)
        // Or no client is connected
        || String::from_utf8_lossy(&status.stdout).starts_with("Stopped")
    {
        // Nothing to display. Empty string will make the status bar disappear
        return String::new();
    }

    let metadata = Command::new("playerctl")
        .args(&["--player=spotifyd", "metadata", "--format", FORMAT])
        .output()
        .expect("Failed to execute playerctl!");

    // If no metadata could be fetched, stdout will just be an empty string
    String::from_utf8_lossy(&metadata.stdout)
        .trim_end_matches(|c: char| c == '\n')
        .into()
}

/// Scroll the buffer over unicode graphemes
fn scroll_by(buffer: &str, scroll_amount: usize) -> String {
    buffer
        .graphemes(true)
        .cycle()
        .skip(scroll_amount)
        .take(MAX_WIDTH)
        .collect()
}

fn advance_scroll_amount(buffer: &str, scroll_amount: usize) -> usize {
    // We need to count graphemes so the scrolling resets at the correct position
    let num_graphemes = buffer.graphemes(true).count();

    if num_graphemes <= MAX_WIDTH {
        0
    } else {
        (scroll_amount + 1) % num_graphemes
    }
}

fn main() {
    let mut buffer: String = String::new();
    let mut scroll_amount: usize = 0;
    let (tx, rx) = mpsc::channel();

    // Update the track metadata in a separate thread with more delay between each update
    // to mitigate rate limiting spotify imposes on API calls
    thread::spawn(move || loop {
        tx.send(playerctl_output()).unwrap();
        thread::sleep(POLL_DELAY);
    });

    // Scroll through the buffer in an endless loop
    loop {
        // Process output only if playerctl ran successfully. Otherwise, the previously received
        // output is reused. This is useful as playerctl will randomly fail with spotifyd:
        // https://github.com/Spotifyd/spotifyd/issues/557
        if let Ok(output) = rx.try_recv() {
            if output != buffer {
                // Read changed output to buffer
                buffer = output;

                // Reset slice position for new track
                scroll_amount = 0;
            }
        }

        // Polybar will re-draw a custom script on every new line it outputs
        println!("{}", scroll_by(&buffer, scroll_amount));

        if scroll_amount == 0 {
            // If this is a new track, wait for a while before starting to scroll
            // This makes the track easier to read initially
            thread::sleep(SCROLL_DELAY * 10);
        } else {
            thread::sleep(SCROLL_DELAY);
        }
        scroll_amount = advance_scroll_amount(&buffer, scroll_amount);
    }
}
