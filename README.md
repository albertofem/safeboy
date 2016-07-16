safeboy, a gameboy emulator in Rust
====

[![Build Status](https://travis-ci.com/albertofem/safeboy.svg?token=ydDBs98aEyX2bMHcZpEx&branch=master)](https://travis-ci.com/albertofem/safeboy)

![Safeboy Logo](https://raw.githubusercontent.com/albertofem/safeboy/master/logo.png?token=AAY_gE9KhnJZpdV37GtHCtK0iQXJ_WJPks5Wt1c1wA%3D%3D)

safeboy is a gameboy (8bit, black and white) emulator made in Rust. It's based on RBoy implementation (https://github.com/mvdnes/rboy).

# Learning purposes

This emulator is made with learning purposes. With that I mind:

* I documented as much as I can all complicated hardware instructions and other parts of the core (MMU, GPU, etc.)
* I try to have as many as test cases as possible
* Variable names are not abbreviated like it's usual in emulators, so it's more understandable

# Supported features

* Only GameBoy, no GameBoy color support
* MBC1
* Timer

# TODO

* Rest of MBCs
* Sound
* Serial Port

# License

MIT