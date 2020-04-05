## NAMP (Not Another Music Player)
Created with 
- Rust
- [web-view](https://github.com/Boscop/web-view)
- [actix-web](https://github.com/actix/actix-web)
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
- Possibly install the web server as a systemd/windows service instead of spinning it up on the fly, might be more robust
- Add option to install as a separate client/server system to be used as a hosted web app