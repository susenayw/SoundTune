# SoundTune

A compact Windows app for adjusting playback and microphone levels. Select your devices, adjust their volume, or mute them independently. Changes apply to the selected Windows audio endpoints.

## Built with

- Rust
- egui and eframe for the desktop interface
- Windows Core Audio via the `windows` crate

## Run

Download the Windows executable from Releases and run it. SoundTune controls Windows audio levels; it is not a hardware driver and does not provide device-specific audio effects.

## Build

Install Rust and a Windows GNU toolchain with `windres`, then run `cargo build --release`. The executable is in `target/release`.

This is an early prerelease; feedback and issue reports are welcome.
