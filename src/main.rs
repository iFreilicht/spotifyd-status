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

fn main() {
    let mut last_valid_output: Vec<u8> = vec![0];
    let mut buffer: String = String::new();
    let mut scroll_amount: usize = 0;
    loop {
        // Check whether spotify is even running first
        let running = Command::new("pgrep")
            .arg("spotifyd")
            .output()
            .expect("Failed to execute pgrep!")
            .status
            .success();

        // String slice that scrolls through the buffer
        let mut sliced: &str = "";

        if !running {
            // Clear buffers so we detect change properly when spotifyd comes back
            buffer.clear();
            last_valid_output.clear();
        } else {
            // Get spotifyd data from playerctl
            let playerctl = Command::new("playerctl")
                .args(&["--player=spotifyd", "metadata", "--format", FORMAT])
                .output()
                .expect("Failed to execute playerctl!");

            // Continue only if execution was successful. Otherwise, the previously received
            // output is reused. This is useful as playerctl will randomly fail with spotifyd:
            // https://github.com/Spotifyd/spotifyd/issues/557
            if playerctl.status.code() == Some(0) && last_valid_output != playerctl.stdout {
                // Read changed output to buffer and format it
                last_valid_output = playerctl.stdout;
                buffer = String::from_utf8_lossy(&last_valid_output).into();
                buffer.pop(); // Remove trailing newline
                buffer.push_str(DIVIDER);

                // We need two copies back-to-back so the string slice seemlingly wraps around
                buffer.push_str(&buffer.clone());

                // Reset slice position for new song
                scroll_amount = 0
            }

            // This may be zero, even if the buffer is 1 charactor long!
            let half_length = buffer.len() / 2;

            // Don't try to calculate anything else, it's not going to be possible
            if half_length == 0 {
                sliced = &buffer;
            } else {
                // Move slice by one
                scroll_amount = (scroll_amount + 1) % half_length;
                // TODO: This is not Unicode safe yet!
                sliced = &buffer[scroll_amount..scroll_amount + min(MAX_WIDTH, half_length)];
            }
        }

        println!("{}", sliced);
        thread::sleep(DELAY);
    }
}
