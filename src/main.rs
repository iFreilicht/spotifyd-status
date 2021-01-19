//! A simple scrolling spotifyd status tool
//! Reads metadata using playerctl and buffers it for more robust operation
//! Will not fail if spotifyd is not running, no song is playing,
//! or if playerctl fails to retrieve metadata, but handle that gracefully.

use std::cmp::min;
use std::process::Command;
use std::thread;
use std::time::Duration;

// Divider between scrolling instances of spotifyd output
const DIVIDER: &str = " - ";

// Format specification to pass to playerctl. To see the potential options, run
// $ playerctl --player=spotifyd metadata
// and check the playerctl man pages, searching for "Format Strings"
const FORMAT: &str = "{{ artist }}: {{ album }}: {{ title }}";

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

    // We can't process any further if we didn't get valid output
    // This can happen if the track is paused or spotifyd didn't respond to the request
    if output.status.code() != Some(0) {
        return None;
    }

    Some(String::from_utf8_lossy(&output.stdout).into())
}

/// Format the buffer to contain the divider and a second copy of itself to ease scrolling
fn format_buffer(playerctl_output: &str) -> String {
    let mut buffer: String = playerctl_output.into();
    buffer.pop(); // Remove trailing newline
    buffer.push_str(DIVIDER);

    // We need two copies back-to-back so the string slice seemlingly wraps around
    buffer.push_str(&buffer.clone());
    buffer
}

fn scroll_by(buffer: &str, scroll_amount: usize) -> &str {
    // This may be zero, even if the buffer is 1 charactor long!
    let half_length = buffer.len() / 2;

    // Don't try to calculate anything else, it's not going to be possible
    if half_length == 0 {
        buffer
    } else {
        // TODO: This is not Unicode safe yet!
        &buffer[scroll_amount..scroll_amount + min(MAX_WIDTH, half_length)]
    }
}

fn advance_scroll_amount(buffer: &str, scroll_amount: usize) -> usize {
    if buffer.len() <= MAX_WIDTH {
        return 0;
    }

    // Move slice by one
    (scroll_amount + 1) % (buffer.len() / 2)
}

fn main() {
    let mut last_valid_output: String = String::new();
    let mut buffer: String = String::new();
    let mut scroll_amount: usize = 0;
    loop {
        // String slice that scrolls through the buffer
        let mut sliced: &str = "";

        // Check whether spotify is even running first
        if !is_spotifyd_running() {
            // Clear buffers so we detect change properly when spotifyd comes back
            buffer.clear();
            last_valid_output.clear();
        } else {
            // Process output only if playerctl ran successfully. Otherwise, the previously received
            // output is reused. This is useful as playerctl will randomly fail with spotifyd:
            // https://github.com/Spotifyd/spotifyd/issues/557
            if let Some(output) = playerctl_output() {
                if output != last_valid_output {
                    // Read changed output to buffer and format it
                    last_valid_output = output;
                    buffer = format_buffer(&last_valid_output);

                    // Reset slice position for new track
                    scroll_amount = 0;
                }
            }

            sliced = scroll_by(&buffer, scroll_amount);
        }

        // Polybar will re-draw a custom script on every new line it outputs
        println!("{}", sliced);

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
