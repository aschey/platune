![Platune](res/platune-title.png)

![license](https://img.shields.io/badge/License-MIT%20or%20Apache%202-green.svg)
[![CI](https://github.com/aschey/platune/actions/workflows/test.yml/badge.svg)](https://github.com/aschey/platune/actions/workflows/test.yml)
![codecov](https://codecov.io/gh/aschey/platune/branch/main/graph/badge.svg?token=NWS6Q3W4FP)
![GitHub repo size](https://img.shields.io/github/repo-size/aschey/platune)
![Lines of Code](https://aschey.tech/tokei/github/aschey/platune)

**NOTE: This project is still in its early stages and many of these features do
not work yet**

## Overview

Platune is a cross-platform music player that uses a client-server architecture
over gRPC. It is both a complete solution and a set of modular components to
allow users to create their own custom players. At its core, Platune is a set of
[protobuf definitions](https://github.com/aschey/platune/tree/main/platuned/proto)
that create the contract. Any gRPC client or server that implements these
protobufs will be compatible with the rest of the ecosystem.

Platune is split up into two independent modules - an audio player and a library
manager. These two modules can be ran together as a single server or
independently as two separate servers. This allows for a variety of setups such
as using the Platune management server with a different audio player or setting
up your own music server where a local Platune audio server can play music
streamed from a remote Platune management server.

### Structure

- [libplatune](https://github.com/aschey/platune/tree/main/libplatune) - Set of
  libraries containing business logic. Can be used to create custom servers.
  - [management](https://github.com/aschey/platune/tree/main/libplatune/management) -
    Library for managing an audio database
  - [player](https://github.com/aschey/platune/tree/main/libplatune/player) -
    Library for audio playback
- [platuned](https://github.com/aschey/platune/tree/main/platuned)
  - [client](https://github.com/aschey/platune/tree/main/platuned/client) -
    Generated client stubs for multiple languages
  - [server](https://github.com/aschey/platune/tree/main/platuned/server) - gRPC
    server frontend for libplatune
- [platunectl](https://github.com/aschey/platune/tree/main/platuned/server/src/bin/platunectl.rs) -
  Management interface for the platuned system service
- [platune-cli](https://github.com/aschey/platune/tree/main/platune-cli) -
  Hybrid CLI/TUI designed for quick and easy usage
- platune-gui (not yet implemented) - Feature-rich graphical client
- [platune-tray](https://github.com/aschey/platune/tree/main/platune-tray) -
  Simple tray-based client
- platune-mobile (not yet implemented) - mobile app

## Feature Overview

- Advanced searching capabilities
- Flexible tag system to allow for complex data organization
- Cross-platform support
- Support multi-boot setups via configurable drive mappings
- Sync data between mobile and desktop
- Mobile app functions as both a standalone player and a remote control
- Custom audio visualizations
- Gapless audio playback
- Automatic filesystem syncing
- Push-based architecture - all running clients should be notified and kept in
  sync with the current state of the server
- Playback of local files and streaming over HTTP

## Project Goals

- Be simple to use with minimal configuration. Don't try to support every
  possible use case.
- Follow the [Unix Philosophy](https://en.wikipedia.org/wiki/Unix_philosophy) -
  the core code should be simple and extendable, non-essential functionality
  should be implemented separately.
- Strive for compatibility with established tools and protocols where possible.
- Focus on file-based media playback, not third-party streaming services
  (integration with streaming services may be added via plugins in the future).
- The GUI and CLI will support all essential features. Extra functionality may
  only be available on the GUI.

### Comparison with MPD

[MPD](https://www.musicpd.org/) is a similar music player app that has been
around for a while. MPD is mainly focused on playing music while Platune aims to
be a complete solution for managing your music library. We also aim to be
accessible for less tech-savvy users, support most features without the need for
complex customizations, and require less configuration. Platune maintains a set
of official clients that take advantage of all available functionality while MPD
mainly relies on third-party clients. However, MPD is a much more stable and
robust product that supports a variety of complex audio setups which will
probably never be supported by Platune.
