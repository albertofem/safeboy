safeboy, a gameboy emulator in Rust
====

[![Build Status](https://travis-ci.org/albertofem/safeboy.svg?branch=master)](https://travis-ci.org/albertofem/safeboy)

![Safeboy Logo](https://raw.githubusercontent.com/albertofem/safeboy/master/logo.png)

safeboy is a GameBoy (8bit, black and white) emulator made in Rust. It's based mainly on RBoy implementation (https://github.com/mvdnes/rboy).

# Learning purposes

This emulator is made for learning purposes. With that I mind:

* I documented as much as I can all complicated hardware instructions and other parts of the core (MMU, GPU, etc.)
* I try to have as many as test cases as possible (WIP!)
* Variable names are not abbreviated like it's usual in emulators, so it's more understandable

# Supported features

* MBC1
* Timer

# TODO

* Rest of MBCs
* Sound
* Serial Port
* Accuracy

# License

MIT

# Some links and interesting stuff

| Link | Description |
| --- | --- |
| [Pandocs](http://problemkaputt.de/pandocs.htm)| Clasic GameBoy and GameBoy Color reference |
| [awesome-gbdev](https://github.com/avivace/awesome-gbdev) | Good compilation of resources about GameBoy development |
| [mooneye-gb](https://github.com/Gekkio/mooneye-gb) | GameBoy research project with many accuracy notes |
| [Simplicity Betrayed](http://queue.acm.org/detail.cfm?id=1755886) | Good article on emulation |
| [GameBoy Programming Info](http://fms.komkon.org/GameBoy/Tech/Software.html) | Some programming info on the GameBoy |
| [Virtual GameBoy](http://fms.komkon.org/VGB/) | Multiplatform GameBoy emulator |
| [Gambatte](https://github.com/sinamas/gambatte) | Cycle accurate GameBoy emulator with sources |
| [GBDev Wiki](http://gbdev.gg8.se/wiki/articles/Main_Page) | Wiki for GB developers with many links and info |
| [GameBoy ASMSCHOOL](http://gameboy.mongenel.com/asmschool.html) | Good series of articles on GameBoy assembly |
| [Emulating the GameBoy](http://www.codeslinger.co.uk/pages/projects/gameboy.html) | Collection of articles on GameBoy emulation with tons of detail and implementation snippets |
| [Z-80 articles](http://www.righto.com/search/label/Z-80) | Collection of low level details about the Z80 |
| [Z-80 Stack](http://www.smspower.org/Development/Stack) | Good article on the Z80 stack and assembler details |
| [GameBoy Playmobile](http://www.rickard.gunee.com/projects/playmobile/html/index.html) | Building a wireless GameBoy, good series of articles |
