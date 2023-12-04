extern crate safeboy;
extern crate clap;

use clap::Parser;
use safeboy::frontend::gameboy::Gameboy;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    rom: String,
}

fn main() {
    let args = Args::parse();

    let rom_file = args.rom;

    println!("Welcome to Safeboy! We are preparing your rom to emulate...");
    println!("Loading rom file: {}", rom_file);

    let mut gameboy = Gameboy::new(rom_file.as_str());

    gameboy.run();
}