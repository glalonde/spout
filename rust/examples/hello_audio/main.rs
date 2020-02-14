use memmap::Mmap;
use sdl2::audio::{AudioCallback, AudioSpecDesired};
use std::env;
use std::error::Error;
use std::fs::File;
use std::io::{stdout, Write};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Duration;

fn main() {
    match run() {
        Ok(_) => {}
        Err(e) => eprintln!("error: {}", e),
    }
}

struct ModPlayer<'a> {
    oxdz: oxdz::Oxdz<'a>,
    data: Arc<Mutex<oxdz::FrameInfo>>,
}

impl<'a> AudioCallback for ModPlayer<'a> {
    type Channel = i16;

    fn callback(&mut self, mut out: &mut [i16]) {
        {
            let mut fi = self.data.lock().unwrap();
            self.oxdz.frame_info(&mut fi);
        }
        self.oxdz.fill_buffer(&mut out, 0);
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let filename = "../archaeology/spout/spoutSDL/src/music/brainless_2.mod";
    let file = File::open(filename)?;

    let oxdz = {
        let mmap = unsafe { Mmap::map(&file).expect("failed to map the file") };
        oxdz::Oxdz::new(&mmap[..], 44100, "")?
    };

    // Display basic module information
    let mut mi = oxdz::ModuleInfo::new();
    oxdz.module_info(&mut mi);
    println!("Title : {}", mi.title);
    println!("Format: {}", mi.description);

    // From Rust-SDL2 SquareWave example
    let sdl_context = sdl2::init().unwrap();
    let audio_subsystem = sdl_context.audio().unwrap();

    let desired_spec = AudioSpecDesired {
        freq: Some(44100),
        channels: Some(2), // stereo
        samples: None,     // default buffer size
    };

    let data = Arc::new(Mutex::new(oxdz::FrameInfo::new()));

    let device = audio_subsystem
        .open_playback(None, &desired_spec, |spec| {
            // Show obtained AudioSpec
            println!("{:?}", spec);

            // initialize the audio callback
            ModPlayer {
                oxdz,
                data: data.clone(),
            }
        })
        .unwrap();

    // Start playback
    device.resume();

    loop {
        {
            let fi = data.lock().unwrap();
            print!("pos:{:3} - row:{:3} \r", fi.pos, fi.row);
        }
        stdout().flush().unwrap();
        std::thread::sleep(Duration::from_millis(50));
    }
}
