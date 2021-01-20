//! A simple scrolling spotifyd status tool
//! Reads metadata using playerctl and buffers it for more robust operation
//! Will not fail if spotifyd is not running, no song is playing,
//! or if playerctl fails to retrieve metadata, but handle that gracefully.

use std::process::Command;
use std::thread;
use std::time::Duration;
use unicode_segmentation::UnicodeSegmentation;

// Format specification to pass to playerctl. To see the potential options, run
// $ playerctl --player=spotifyd metadata
// and check the playerctl man pages, searching for "Format Strings"
const FORMAT: &str = " {{ artist }}  {{ album }}  {{ title }} - ";

// How long to sleep between each iteration
const DELAY: Duration = Duration::from_millis(300);

// Maximum number of characters in the output
const MAX_WIDTH: usize = 20;

fn is_spotifyd_running() -> bool {
    Command::new("pgrep")
        .arg("spotifyd")
        .output()
        .expect("Failed to execute pgrep!")
        .status
        .success()
}

/// Run playerctl to get info about currently playing track
fn playerctl_output() -> Option<String> {
    let output = Command::new("playerctl")
        .args(&["--player=spotifyd", "metadata", "--format", FORMAT])
        .output()
        .expect("Failed to execute playerctl!");

    if output.status.code() != Some(0) {
        // We can't process any further if we didn't get valid output
        // This can happen if the track is paused or spotifyd didn't respond to the request
        None
    } else {
        Some(
            String::from_utf8_lossy(&output.stdout)
                .trim_end_matches(|c: char| c == '\n')
                .into(),
        )
    }
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
    loop {
        // Check whether spotify is even running first
        if !is_spotifyd_running() {
            // Clear buffer so we detect change properly when spotifyd comes back
            buffer.clear();
        } else {
            // Process output only if playerctl ran successfully. Otherwise, the previously received
            // output is reused. This is useful as playerctl will randomly fail with spotifyd:
            // https://github.com/Spotifyd/spotifyd/issues/557
            if let Some(output) = playerctl_output() {
                if output != buffer {
                    // Read changed output to buffer
                    buffer = output;

                    // Reset slice position for new track
                    scroll_amount = 0;
                }
            }
        }

        // Polybar will re-draw a custom script on every new line it outputs
        println!("{}", scroll_by(&buffer, scroll_amount));

        if scroll_amount == 0 {
            // If this is a new track, wait for a while before starting to scroll
            // This makes the track easier to read initially
            thread::sleep(DELAY * 10);
        } else {
            thread::sleep(DELAY);
        }
        scroll_amount = advance_scroll_amount(&buffer, scroll_amount);
    }
}
