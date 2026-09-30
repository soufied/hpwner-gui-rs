# hpwnr GUI

A desktop GUI front end for the [`hpwnr`](https://github.com/Omegaplexx/hpwnr) command-line tool, built with Rust and [egui](https://github.com/emilk/egui). It provides forms for decrypting and encrypting Happ/V2RayTun subscription links, fetching and converting subscriptions, and configuring request options, without needing to type CLI arguments by hand.

The app shells out to the `hpwnr` executable to perform the actual decrypt, encrypt, fetch and convert operations, and only handles input, validation and result display itself.

## Prerequisites

- Linux
- Rust and Cargo (stable toolchain), see [rustup.rs](https://rustup.rs)
- The `hpwnr` executable, either on your `PATH` or at a known location on disk
- GTK3 development libraries, required by the native file picker (`rfd`), e.g. on Debian/Ubuntu: `sudo apt install libgtk-3-dev`

## Build and run

```bash
git clone https://github.com/soufied/hpwner-gui-rs
cd hpwner-gui-rs
cargo run --release
```

This builds the application and launches it. A standalone release binary is produced at `target/release/hpwnr-gui`.

## Usage

1. Open **Settings** and set the path to your `hpwnr` executable, either a bare name such as `hpwnr` if it is on your `PATH`, or a full path to the binary, then use **Browse...** if you prefer to pick it from disk.
2. Use the sidebar to switch between **Decrypt**, **Encrypt**, **Fetch** and **Convert**.
3. Fill in the required fields for the selected operation and press **Run** (or `Ctrl+Enter`).
4. Results appear in the panel at the bottom of the window, and can be copied or saved from there.

Press `Ctrl+K` at any time to open the command palette and jump to a section or action by name.
