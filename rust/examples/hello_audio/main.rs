use log::{error, info};
use sdl2::mixer::{InitFlag, AUDIO_S16LSB, DEFAULT_CHANNELS};
/// Demonstrates the simultaneous mixing of music and sound effects.
use std::path::Path;

gflags::define! {
    --log_filter: &str = "info"
}
gflags::define! {
    --music_file: &str = "bviinaaa.mod"
}
gflags::define! {
    -h, --help = false
}

fn main() -> Result<(), String> {
    gflags::parse();
    if HELP.flag {
        gflags::print_help_and_exit(0);
    }
    scrub_log::init_with_filter_string(LOG_FILTER.flag).unwrap();
    const MUSIC_PATH: &'static str = "../archaeology/spout/spoutSDL/src/music/";
    let dest_path = std::path::Path::new(MUSIC_PATH).join(MUSIC_FILE.flag);
    if demo(&dest_path).is_err() {
        error!("Failed to get file, use flag --music_file with one of these names");
        walkdir::WalkDir::new(MUSIC_PATH)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| !e.file_type().is_dir())
            .filter_map(Some)
            .for_each(|e| info!("Music: {:?}", e.path().file_name().unwrap()));
    }
    Ok(())
}

fn demo(music_file: &Path) -> Result<(), String> {
    println!("linked version: {}", sdl2::mixer::get_linked_version());

    let sdl = sdl2::init()?;
    let _audio = sdl.audio()?;
    let mut timer = sdl.timer()?;

    let frequency = 44_100;
    let format = AUDIO_S16LSB; // signed 16 bit samples, in little-endian byte order
    let channels = DEFAULT_CHANNELS; // Stereo
    let chunk_size = 1_024;
    sdl2::mixer::open_audio(frequency, format, channels, chunk_size)?;
    let _mixer_context =
        sdl2::mixer::init(InitFlag::MP3 | InitFlag::FLAC | InitFlag::MOD | InitFlag::OGG)?;

    // Number of mixing channels available for sound effect `Chunk`s to play
    // simultaneously.
    sdl2::mixer::allocate_channels(4);

    {
        let n = sdl2::mixer::get_chunk_decoders_number();
        println!("available chunk(sample) decoders: {}", n);
        for i in 0..n {
            println!("  decoder {} => {}", i, sdl2::mixer::get_chunk_decoder(i));
        }
    }

    {
        let n = sdl2::mixer::get_music_decoders_number();
        println!("available music decoders: {}", n);
        for i in 0..n {
            println!("  decoder {} => {}", i, sdl2::mixer::get_music_decoder(i));
        }
    }

    println!("query spec => {:?}", sdl2::mixer::query_spec());

    let music = sdl2::mixer::Music::from_file(music_file)?;

    fn hook_finished() {
        println!("play ends! from rust cb");
    }

    sdl2::mixer::Music::hook_finished(hook_finished);

    println!("music => {:?}", music);
    println!("music type => {:?}", music.get_type());
    println!("music volume => {:?}", sdl2::mixer::Music::get_volume());
    println!("play => {:?}", music.play(1));
    while sdl2::mixer::Music::is_playing() {
        timer.delay(10);
    }
    sdl2::mixer::Music::halt();
    Ok(())
}
