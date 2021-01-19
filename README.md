# spotifyd-status

A simple scrolling status widget for spotifyd and polybar.
Written in rust, WIP.

It is tolerant to faults by both playerctl and spotifyd, so issues with 
[spotifyd not responding to playerctl queries](https://github.com/Spotifyd/spotifyd/issues/557)
will not disrupt the user experience.

# Installation

Clone this repo, make sure you have `cargo` installed and build in release mode:

    cargo build --release

Put the following in your `polybar/config` file (replacing `<path>/<to>` appropriately):

    [module/spotifyd]
    type = custom/script
    interval = 0
    tail = true
    format = <label>
    exec = /home/<path>/<to>/spotifyd-status

And make sure to add it to your bar's modules:

    [bar/primary]
    modules-right =  tray spotify pulseaudio date battery powermenu

After that, re-launch polybar:

    ~/.config/polybar/launch.sh

And you're good to go!
Because this script is stateful and runs forever, you need to re-launch polybar
everytime you re-compile.

# TODO

- [ ] Allow UTF-8 strings to be used for formatting
- [ ] Enable setting parameters other than by recompiling
- [ ] Clear output if spotifyd didn't respond for a long time