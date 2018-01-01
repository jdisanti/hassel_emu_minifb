extern crate clap;
extern crate hassel_emu;
extern crate minifb;

use hassel_emu::hassel::{GraphicsDevice, HasselSystemBuilder, IODevice, Key, REQUIRED_ROM_SIZE,
                         SCREEN_HEIGHT_PIXELS, SCREEN_WIDTH_PIXELS};
use hassel_emu::emulator::Emulator;

use clap::{App, Arg, SubCommand};
use minifb::{Window, WindowOptions};

use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::time::Instant;
use std::process;

use std::cell::RefCell;
use std::rc::Rc;

fn load_rom(rom_path: &str) -> Result<Vec<u8>, String> {
    println!("Loading rom named \"{}\"...", rom_path);
    let mut rom_file =
        File::open(rom_path).map_err(|_| format!("Failed to load ROM: {}", rom_path))?;

    let mut rom = Vec::new();
    rom_file
        .read_to_end(&mut rom)
        .map_err(|_| format!("Failed to read ROM: {}", rom_path))?;

    if rom.len() != REQUIRED_ROM_SIZE {
        return Err(format!(
            "ROM has unexpected size ({}); should be {} bytes.",
            rom.len(),
            REQUIRED_ROM_SIZE
        ));
    }

    Ok(rom)
}

fn main() {
    let matches = App::new("Hasseldorf Emulator (with minifb)")
        .version("0.1")
        .author("John DiSanti <johndisanti@gmail.com>")
        .about("Emulates ROMs for the homebrew Hasseldorf Computer")
        .arg(
            Arg::with_name("ROM")
                .help("Sets the ROM file to use")
                .required(true),
        )
        .arg(
            Arg::with_name("bench")
                .long("bench")
                .help("Run in benchmark mode (to performance test the emulator)"),
        )
        .get_matches();

    let rom_path = matches.value_of("ROM").unwrap();
    let rom = match load_rom(&rom_path) {
        Ok(rom) => rom,
        Err(err) => {
            println!("{}", err);
            process::exit(1);
        }
    };

    let (memory, graphics, io) = HasselSystemBuilder::new().rom(rom).build();
    let mut emulator = Emulator::new(memory);
    emulator.reset();

    if matches.is_present("bench") {
        run_mode_benchmark(emulator);
    } else {
        run_mode_default(emulator, graphics, io);
    }
}

fn run_mode_benchmark(mut emulator: Emulator) {
    let normal_time_seconds = 20;
    let normal_cycles_per_second = 6_000_000;
    let bench_cycles = normal_time_seconds * normal_cycles_per_second;

    let start_time = Instant::now();
    let mut total_cycles: usize = 0;
    while total_cycles < bench_cycles {
        total_cycles += emulator.step() as usize;
    }

    let total_time = Instant::now().duration_since(start_time);
    let total_time_f =
        total_time.as_secs() as f64 + (total_time.subsec_nanos() as f64 / 1_000_000_000f64);
    let ratio = normal_time_seconds as f64 / total_time_f;
    let mhz = (total_cycles / 1_000_000) as f64 / total_time_f as f64;
    println!(
        "Took {} seconds to execute {} cycles (about {} times real time, or {} MHz)",
        total_time_f, bench_cycles, ratio, mhz
    );
}

fn run_mode_default(
    mut emulator: Emulator,
    graphics: Rc<RefCell<GraphicsDevice>>,
    io: Rc<RefCell<IODevice>>,
) {
    let mut window = Window::new(
        "Hasseldorf Emulator",
        SCREEN_WIDTH_PIXELS,
        SCREEN_HEIGHT_PIXELS,
        WindowOptions::default(),
    ).unwrap_or_else(|e| {
        println!("Failed to create a window: {}", e);
        process::exit(1);
    });

    let mut time_last_step = Instant::now();
    let mut time_last_render = Instant::now();

    let mut total_cycles: usize = 0;
    let mut previous_keys: Vec<minifb::Key> = Vec::new();
    while window.is_open() {
        let since_last_render = Instant::now().duration_since(time_last_render);
        if since_last_render.subsec_nanos() > 13_000_000u32 {
            window
                .update_with_buffer(graphics.borrow().frame_buffer())
                .unwrap();
            time_last_render = Instant::now();
        }

        if let Some(keys_down) = window.get_keys() {
            for key in &keys_down {
                if !previous_keys.contains(&key) {
                    io.borrow_mut().key_down(convert_key(key));
                }
            }
            for key in &previous_keys {
                if !keys_down.contains(&key) {
                    io.borrow_mut().key_up(convert_key(key));
                }
            }
            previous_keys = keys_down;
        }

        let cycles = emulator.step() as u32;
        total_cycles += cycles as usize;

        // Slow down so that we're running at approximately 6 MHz
        loop {
            let since_last_step = Instant::now().duration_since(time_last_step);
            // 167 nanoseconds per cycle at 6 MHz
            if since_last_step.subsec_nanos() > cycles * 167u32 {
                time_last_step = Instant::now();
                break;
            }
        }
    }
}

fn convert_key(key: &minifb::Key) -> Key {
    match *key {
        minifb::Key::Key0 => Key::Key0,
        minifb::Key::Key1 => Key::Key1,
        minifb::Key::Key2 => Key::Key2,
        minifb::Key::Key3 => Key::Key3,
        minifb::Key::Key4 => Key::Key4,
        minifb::Key::Key5 => Key::Key5,
        minifb::Key::Key6 => Key::Key6,
        minifb::Key::Key7 => Key::Key7,
        minifb::Key::Key8 => Key::Key8,
        minifb::Key::Key9 => Key::Key9,

        minifb::Key::A => Key::A,
        minifb::Key::B => Key::B,
        minifb::Key::C => Key::C,
        minifb::Key::D => Key::D,
        minifb::Key::E => Key::E,
        minifb::Key::F => Key::F,
        minifb::Key::G => Key::G,
        minifb::Key::H => Key::H,
        minifb::Key::I => Key::I,
        minifb::Key::J => Key::J,
        minifb::Key::K => Key::K,
        minifb::Key::L => Key::L,
        minifb::Key::M => Key::M,
        minifb::Key::N => Key::N,
        minifb::Key::O => Key::O,
        minifb::Key::P => Key::P,
        minifb::Key::Q => Key::Q,
        minifb::Key::R => Key::R,
        minifb::Key::S => Key::S,
        minifb::Key::T => Key::T,
        minifb::Key::U => Key::U,
        minifb::Key::V => Key::V,
        minifb::Key::W => Key::W,
        minifb::Key::X => Key::X,
        minifb::Key::Y => Key::Y,
        minifb::Key::Z => Key::Z,

        minifb::Key::Space => Key::Space,
        minifb::Key::Tab => Key::Tab,

        minifb::Key::Backslash => Key::Backslash,
        minifb::Key::Comma => Key::Comma,
        minifb::Key::Equal => Key::Equal,
        minifb::Key::LeftBracket => Key::LeftBracket,
        minifb::Key::Minus => Key::Minus,
        minifb::Key::Period => Key::Period,
        minifb::Key::RightBracket => Key::RightBracket,
        minifb::Key::Semicolon => Key::Semicolon,

        minifb::Key::Slash => Key::Slash,
        minifb::Key::Enter => Key::Enter,

        minifb::Key::Backspace => Key::Backspace,
        minifb::Key::Delete => Key::Delete,
        minifb::Key::End => Key::End,

        minifb::Key::F1 => Key::F1,
        minifb::Key::F2 => Key::F2,
        minifb::Key::F3 => Key::F3,
        minifb::Key::F4 => Key::F4,
        minifb::Key::F5 => Key::F5,
        minifb::Key::F6 => Key::F6,
        minifb::Key::F7 => Key::F7,
        minifb::Key::F8 => Key::F8,
        minifb::Key::F9 => Key::F9,
        minifb::Key::F10 => Key::F10,
        minifb::Key::F11 => Key::F11,
        minifb::Key::F12 => Key::F12,
        minifb::Key::F13 => Key::F13,
        minifb::Key::F14 => Key::F14,
        minifb::Key::F15 => Key::F15,

        minifb::Key::Down => Key::Down,
        minifb::Key::Left => Key::Left,
        minifb::Key::Right => Key::Right,
        minifb::Key::Up => Key::Up,
        minifb::Key::Apostrophe => Key::Apostrophe,
        minifb::Key::Backquote => Key::Backquote,

        minifb::Key::Escape => Key::Escape,

        minifb::Key::Home => Key::Home,
        minifb::Key::Insert => Key::Insert,
        minifb::Key::Menu => Key::Menu,

        minifb::Key::PageDown => Key::PageDown,
        minifb::Key::PageUp => Key::PageUp,

        minifb::Key::Pause => Key::Pause,
        minifb::Key::NumLock => Key::NumLock,
        minifb::Key::CapsLock => Key::CapsLock,
        minifb::Key::ScrollLock => Key::ScrollLock,
        minifb::Key::LeftShift => Key::LeftShift,
        minifb::Key::RightShift => Key::RightShift,
        minifb::Key::LeftCtrl => Key::LeftCtrl,
        minifb::Key::RightCtrl => Key::RightCtrl,

        minifb::Key::NumPad0 => Key::NumPad0,
        minifb::Key::NumPad1 => Key::NumPad1,
        minifb::Key::NumPad2 => Key::NumPad2,
        minifb::Key::NumPad3 => Key::NumPad3,
        minifb::Key::NumPad4 => Key::NumPad4,
        minifb::Key::NumPad5 => Key::NumPad5,
        minifb::Key::NumPad6 => Key::NumPad6,
        minifb::Key::NumPad7 => Key::NumPad7,
        minifb::Key::NumPad8 => Key::NumPad8,
        minifb::Key::NumPad9 => Key::NumPad9,
        minifb::Key::NumPadDot => Key::NumPadDot,
        minifb::Key::NumPadSlash => Key::NumPadSlash,
        minifb::Key::NumPadAsterisk => Key::NumPadAsterisk,
        minifb::Key::NumPadMinus => Key::NumPadMinus,
        minifb::Key::NumPadPlus => Key::NumPadPlus,
        minifb::Key::NumPadEnter => Key::NumPadEnter,

        minifb::Key::LeftAlt => Key::LeftAlt,
        minifb::Key::RightAlt => Key::RightAlt,

        minifb::Key::LeftSuper => Key::LeftSuper,
        minifb::Key::RightSuper => Key::RightSuper,

        minifb::Key::Unknown => Key::Unknown,
        minifb::Key::Count => Key::Unknown,
    }
}
