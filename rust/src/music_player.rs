use lazy_static::lazy_static;
use log::{error, info, trace};

// Singleton music player...
lazy_static! {
    pub static ref MUSIC_PLAYER: std::sync::Mutex<MusicThread> =
        std::sync::Mutex::new(MusicThread::init());
}

gflags::define! {
    --log_filter: &str = "info"
}
gflags::define! {
    --library_dir: &str = "../archaeology/spout/spoutSDL/src/music/"
}

fn notify_track_finished() {
    MUSIC_PLAYER.lock().unwrap().notify_track_finished();
}

enum MusicPlayerCommand {
    Play,
    Pause,
    NextTrack,
    TrackFinished,
}

struct MusicPlayer<'a> {
    sdl: sdl2::Sdl,
    library: Vec<Box<sdl2::mixer::Music<'a>>>,
    current_track: usize,
    command_channel: std::sync::mpsc::Receiver<MusicPlayerCommand>,
}

impl<'a> MusicPlayer<'a> {
    pub fn init(
        music_dir: &std::path::Path,
        command_channel: std::sync::mpsc::Receiver<MusicPlayerCommand>,
    ) -> Result<Self, String> {
        let mut player = MusicPlayer::<'a> {
            sdl: sdl2::init()?,
            library: vec![],
            current_track: 0,
            command_channel,
        };
        player.sdl.audio()?;
        let frequency = 44_100;
        let format = sdl2::mixer::AUDIO_S16LSB; // signed 16 bit samples, in little-endian byte order
        let channels = sdl2::mixer::DEFAULT_CHANNELS; // Stereo
        let chunk_size = 1_024;
        sdl2::mixer::open_audio(frequency, format, channels, chunk_size)?;
        sdl2::mixer::Music::hook_finished(notify_track_finished);

        // Open all the library files.
        let files: Vec<Box<sdl2::mixer::Music<'a>>> = walkdir::WalkDir::new(music_dir)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| !e.file_type().is_dir())
            .filter_map(Some)
            .map(|e| e.path().to_path_buf())
            .map(|e| -> Result<Box<sdl2::mixer::Music<'a>>, String> {
                Ok(Box::new(sdl2::mixer::Music::from_file(e)?))
            })
            .filter_map(Result::ok)
            .collect();

        if files.len() <= 0 {
            return Err(String::from("No songs to play"));
        }
        // player.library = files;
        player.library = files;

        Ok(player)
    }

    pub fn run(&mut self) {
        loop {
            // Keep looping here so we can check if the song is still playing and move on if
            // it isn't.
            match self.command_channel.recv() {
                Ok(command) => match command {
                    MusicPlayerCommand::Play => self.handle_play(),
                    MusicPlayerCommand::Pause => self.handle_pause(),
                    MusicPlayerCommand::NextTrack => self.handle_next_track(),
                    MusicPlayerCommand::TrackFinished => self.handle_track_finished(),
                },
                Err(err) => error!("Error: {}", err),
            }
        }
    }

    fn handle_play(&mut self) {
        trace!("Starting playback");
        if sdl2::mixer::Music::is_paused() {
            // If we were already playing something.
            sdl2::mixer::Music::resume();
        } else {
            if let Err(err) = self.library[self.current_track].play(1) {
                error!("Error playing file: {}", err);
            }
        }
    }

    fn handle_pause(&mut self) {
        trace!("Pausing playback");
        sdl2::mixer::Music::pause();
    }

    fn handle_next_track(&mut self) {
        trace!("Next track");
        let num_songs = self.library.len();
        self.current_track = (self.current_track + 1) % num_songs;
        self.handle_play();
    }

    fn handle_track_finished(&mut self) {
        trace!("Track finished");
        self.handle_next_track();
    }
}

pub struct MusicThread {
    command_channel: std::sync::mpsc::Sender<MusicPlayerCommand>,
    thread: std::thread::JoinHandle<()>,
}
impl MusicThread {
    // TODO pass through failed player init result
    pub fn init() -> Self {
        let (tx, rx) = std::sync::mpsc::channel::<MusicPlayerCommand>();
        MusicThread {
            command_channel: tx,
            thread: std::thread::spawn(move || {
                let mut player =
                    MusicPlayer::init(std::path::Path::new(LIBRARY_DIR.flag), rx).unwrap();
                player.run();
            }),
        }
    }
    pub fn play(&self) {
        self.command_channel.send(MusicPlayerCommand::Play);
    }
    pub fn pause(&self) {
        self.command_channel.send(MusicPlayerCommand::Pause);
    }
    pub fn next_track(&self) {
        self.command_channel.send(MusicPlayerCommand::NextTrack);
    }
    pub fn notify_track_finished(&self) {
        self.command_channel.send(MusicPlayerCommand::TrackFinished);
    }
}
