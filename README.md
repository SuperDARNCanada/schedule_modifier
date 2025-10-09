# Schedule Modifier for Borealis
This repo provides a TUI application for modifying Borealis schedule files.

## Installation
Download the schedule_modifier binary from the [latest release](https://github.com/SuperDARNCanada/schedule_modifier/releases/download/v0.1.0/schedule_modifier-x86_64-unknown-linux-musl.tar.xz), unzip, and place the binary in a convenient location (perhaps with your local copy of the Borealis schedule files?)
Note that you must also have a copy of the [borealis experiments repository](https://github.com/SuperDARNCanada/borealis_experiments) in order to load in the available experiments.

## Usage
```
Usage: schedule_modifier [SCHEDULE_DIR] [EXPERIMENTS_DIR]

Arguments:
  [SCHEDULE_DIR]     Directory containing schedule files (overrides `LOCAL_SCHEDULE_DIR` from environment)
  [EXPERIMENTS_DIR]  Path to borealis experiments directory (defaults to `$BOREALISPATH/src/borealis_experiments`)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

Running the application will open a screen with the current schedule loaded in. 
You may add new schedule lines by pressing `a`, or remove lines by pressing `r`. 
When you are done editing, press `q` from the main screen.
This will take you to a screen showing a summary of the changes made, at which point you can press `y` to confirm the 
new changes and write to file, `q` to cancel the changes and quit, or `b` to go back to editing.
The keybindings are shown at the bottom of the screen at all times, for ease of use.
