# android_apps_egui

This is a utility program I'm working on for two reasons. The main purpose is to organize the apps on my phone and 
figure out which ones to delete, without the distraction of my phone itself.

The other reason is just to get some practice with Rust, and with building gui apps in general.

# About the program
The program is built in rust using [`egui`](https://docs.rs/egui/latest/egui/), a rust gui library, and
[`eframe`](https://docs.rs/eframe/0.16.0/eframe/), its associated framework. It also includes a web scraper that runs
asynchronously and fetches images for the listed apps from the Google Play store.

# Usage
The primary way to export a list of apps that this program can read is using the Android app
[List My Apps](https://play.google.com/store/apps/details?id=de.onyxbits.listmyapps).