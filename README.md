![Platune](res/platune-title.png)

[![license](https://img.shields.io/github/license/aschey/platune)](https://github.com/aschey/platune/blob/master/LICENSE)
[![CI](https://github.com/aschey/Platune/actions/workflows/test.yml/badge.svg)](https://github.com/aschey/Platune/actions/workflows/test.yml)
![GitHub repo size](https://img.shields.io/github/repo-size/aschey/platune)

**NOTE: This project is still in its early stages and most of these features do not work yet**

## Overview

Platune is a cross-platform music player that uses a client-server architecture over gRPC. It is both a complete solution and a set of modular components to allow users to create their own custom players. At its core, Platune is a set of [protobuf definitions](https://github.com/aschey/Platune/tree/master/platuned/proto) that create the contract. Any gRPC client or server that implements these protobufs will be compatible with the rest of the ecosystem.

### Structure

- [libplatune](https://github.com/aschey/Platune/tree/master/libplatune) -
  Set of libraries containing business logic. Can be used to create custom servers.
  - [management](https://github.com/aschey/Platune/tree/master/libplatune/management) -
    Library for managing an audio database
  - [player](https://github.com/aschey/Platune/tree/master/libplatune/player) -
    Library for audio playback
- [platuned](https://github.com/aschey/Platune/tree/master/platuned)
  - [client](https://github.com/aschey/Platune/tree/master/platuned/client) -
    Generated client stubs for multiple languages
  - [server](https://github.com/aschey/Platune/tree/master/platuned/server) -
    gRPC server frontend for libplatune
- [platune-cli](https://github.com/aschey/Platune/tree/master/platune-cli) -
  Simple command line client designed for quick and easy usage
- [platune-tui](https://github.com/aschey/Platune/tree/master/platune-cli) -
  Terminal client for those who hate using mice
- platune-gui (not yet implemented) -
  Feature-rich graphical client
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
- Push-based architecture - all running clients should be notified and kept in sync with the current state of the server

## Project Goals

- Be simple to use with minimal configuration. Don't try to support every possible use case.
- Follow the [Unix Philosophy](https://en.wikipedia.org/wiki/Unix_philosophy) - the core code should be simple and extendable, non-essential functionality should be implemented separately.
- Strive for compatiblity with established tools and protocols where possible.
- Focus on local media playback, not streaming services (integration with streaming services may be added via plugins in the future).
- The GUI, TUI, and CLI will support all essential features. Extra functionality may only be available on the GUI.

### Comparison with MPD

[MPD](https://www.musicpd.org/) is a similar music player app that has been around for a while. Compared to MPD, Platune aims to require less configuration to use and to have a simpler process for creating custom clients. Platune also maintains a set of official clients that take advantage of all available functionality while MPD mainly relies on third-party clients. However, MPD is a much more stable and robust product that supports a variety of complex setups which will probably never be supported by Platune.
