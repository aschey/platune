## NAMP (Not Another Music Player)
[![license](https://img.shields.io/github/license/aschey/NAMP)](https://github.com/aschey/NAMP/blob/master/LICENSE)
![GitHub repo size](https://img.shields.io/github/repo-size/aschey/NAMP)

Created with 
- [Rust](https://www.rust-lang.org/)
- [Electron](https://www.electronjs.org/)
- [actix-web](https://github.com/actix/actix-web)
- [Diesel](https://github.com/diesel-rs/diesel)
- [Sqlite](https://www.sqlite.org/)
- [React](https://reactjs.org/)
- [Typescript](https://www.typescriptlang.org/)
- [Blueprint](https://github.com/palantir/blueprint)

### How It Works
On launch, NAMP will spin up a REST API written in Rust in addition to the Electron UI. The API will handle the majority of the logic,
with the Electron app being used for the presentation layer. The API will send TCP health checks to the Electron app and will automatically shut
itself down if the app is no longer running.

### Goals/Ideas
- Cross-platform desktop app
- Android app
- Support sync between Android and desktop
- Share data between different hard drive partitions and operating systems
- Possibly install the web server as a systemd/windows service instead of spinning it up on the fly, might be more robust
- Add option to install as a separate client/server system to be used as a hosted web app
- Some kind of shortcut to easily play/pause/change volume, etc. Maybe system hotkeys, taskbar icon, and/or browser extension.
- Remote control desktop player from Android app

### Features Completed
- Select folders to import into library
- Choose database path
- Map paths between hard drive partitions
- Host configured files as static files

### Features in Progress
- main song grid