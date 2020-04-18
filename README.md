## NAMP (Not Another Music Player)
[![license](https://img.shields.io/github/license/aschey/NAMP)](https://github.com/aschey/NAMP/blob/master/LICENSE)
![GitHub repo size](https://img.shields.io/github/repo-size/aschey/NAMP)

Created with 
- Rust
- [web-view](https://github.com/Boscop/web-view)
- [actix-web](https://github.com/actix/actix-web)
- [Diesel](https://github.com/diesel-rs/diesel)
- Sqlite
- React
- Typescript
- [Blueprint](https://github.com/palantir/blueprint)

### How It Works
On launch, NAMP will spin up a web server used for hosting mp3 files as well as a REST API, and a platform-dependent browser engine sandboxed inside a desktop app.
Since it's just a web app using a REST API under the hood, the UI can also be accessed at `localhost` from any browser.

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
- Nothing! (yet)

### Features in Progress
- UI to select folders to import