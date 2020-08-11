![Platune](res/platune-title.png)

[![license](https://img.shields.io/github/license/aschey/platune)](https://github.com/aschey/platune/blob/master/LICENSE)
![GitHub repo size](https://img.shields.io/github/repo-size/aschey/platune)

Created with

- [Rust](https://www.rust-lang.org/)
- [Electron](https://www.electronjs.org/)
- [actix-web](https://github.com/actix/actix-web)
- [Diesel](https://github.com/diesel-rs/diesel)
- [Sqlite](https://www.sqlite.org/)
- [React](https://reactjs.org/)
- [Typescript](https://www.typescriptlang.org/)
- [Blueprint](https://github.com/palantir/blueprint)

### Overview

Platune is a cross-platform music player that supports sharing data between operating systems for users that dual boot. Platune has no builtin concept of playlists, genres, or ratings and instead employs a flexible tag system to allow users to organize their music however they see fit.

### Goals/Ideas

- Cross-platform desktop app
- Android app
- Support sync between Android and desktop
- Share data between different hard drive partitions and operating systems
- Possibly install the web server as a systemd/windows service instead of spinning it up on the fly, might be more robust
- Some kind of shortcut to easily play/pause/change volume, etc. Maybe system hotkeys, taskbar icon, and/or browser extension.
- Remote control desktop player from Android app

### Features Completed

- Select folders to import into library
- Choose database path
- Map paths between hard drive partitions
- Host configured files as static files
- Import and sync new files into database
- Play/pause/stop
- Song listview
- Album grid
- Song progress bar
- Theming
- Volume control
- Audio visualizer
- Playback with gapless audio support
- Search

### Features in Progress

- Song queueing
- Tags
