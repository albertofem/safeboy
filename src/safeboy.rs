extern crate safeboy;
extern crate clap;

use clap::{Arg, App};
use safeboy::cartridge::cartridge::Cartridge;

fn main() {
    let matches = App::new("myapp")
        .version("1.0")
        .author("Alberto Fernández <albertofem@gmail.com>")
        .about("A GameBoy cycle accurate emulator")
        .arg(Arg::with_name("ROM")
                .help("Rom file to emulate")
                .required(true)
                .index(1)
            )
        .get_matches();

    let rom_file = matches.value_of("ROM").unwrap();

    println!("Welcome to Safeboy! We are preparing your rom to emulate...");
    println!("Loading rom file: {}", rom_file);

    let mut cartridge = Cartridge::new();

    cartridge.read(&rom_file)
}