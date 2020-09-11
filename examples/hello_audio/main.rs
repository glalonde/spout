use log::{error, info};

gflags::define! {
    --log_filter: &str = "info"
}
gflags::define! {
    --library_dir: &str = "assets/music/output"
}

#[derive(Debug)]
enum MusicPlayerCommand {
    Play,
    Pause,
    NextTrack,
}

struct MusicPlayer {
    output_stream: rodio::OutputStream,
    sound_queue: spout::sound_queue::SoundQueueController<f32>,
    library: Vec<std::path::PathBuf>,
    current_track: usize,
    command_rx: crossbeam_channel::Receiver<MusicPlayerCommand>,
}

impl MusicPlayer {
    pub fn init(
        music_dir: &std::path::Path,
        command_rx: crossbeam_channel::Receiver<MusicPlayerCommand>,
    ) -> Result<Self, String> {
        let (sound_queue, output_stream) = spout::sound_queue::make_default_sound_queue();
        let mut player = MusicPlayer {
            output_stream,
            sound_queue,
            library: vec![],
            current_track: 0,
            command_rx,
        };

        // Open all the library files.
        let files: Vec<std::path::PathBuf> = walkdir::WalkDir::new(music_dir)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| !e.file_type().is_dir())
            .filter(|e| e.path().extension().unwrap() == "ogg")
            .filter(|e| std::fs::File::open(e.path()).is_ok())
            .filter_map(Some)
            .map(|e| e.path().to_path_buf())
            .collect();

        if files.len() <= 0 {
            return Err(String::from("No songs to play"));
        }
        player.library = files;
        player.handle_sound_finished(0);

        Ok(player)
    }

    pub fn run(&mut self) {
        loop {
            // Keep looping here so we can check if the song is still playing and move on if
            // it isn't.
            crossbeam_channel::select! {
                recv(self.command_rx) -> msg => match msg {Ok(command) => self.handle_command(command), Err(_) =>()},
                recv(self.sound_queue.sound_finished_rx) -> msg => match msg {Ok(sound) => self.handle_sound_finished(sound), Err(_) =>()},
            }
        }
    }

    fn try_append_next_track(&mut self) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let file = std::fs::File::open(&self.library[self.current_track])?;
        let decoder = rodio::Decoder::new(std::io::BufReader::new(file))?;
        self.sound_queue.append(decoder);
        info!("Added {:?} to the queue", self.library[self.current_track]);
        Ok(())
    }

    fn handle_sound_finished(&mut self, remaining_sounds: usize) {
        if remaining_sounds < 2 {
            let num_songs = self.library.len();
            if num_songs > 0 {
                self.current_track = (self.current_track + 1) % num_songs;
                let _ = self.try_append_next_track();
            }
        }
    }

    fn handle_command(&mut self, command: MusicPlayerCommand) {
        match command {
            MusicPlayerCommand::Play => self.handle_play(),
            MusicPlayerCommand::Pause => self.handle_pause(),
            MusicPlayerCommand::NextTrack => self.handle_next_track(),
        }
    }

    fn handle_play(&mut self) {
        self.sound_queue.play();
    }

    fn handle_pause(&mut self) {
        self.sound_queue.pause();
    }

    fn handle_next_track(&mut self) {
        self.sound_queue.next();
    }
}

fn main() {
    gflags::parse();
    scrub_log::init_with_filter_string(LOG_FILTER.flag).unwrap();
    let (tx, rx) = crossbeam_channel::unbounded::<MusicPlayerCommand>();
    std::thread::spawn(move || {
        let mut player = MusicPlayer::init(std::path::Path::new(LIBRARY_DIR.flag), rx).unwrap();
        player.run();
    });
    let try_send_command = move |command_string: &str| -> () {
        let maybe_command = match command_string {
            "play\n" => Some(MusicPlayerCommand::Play),
            "pause\n" => Some(MusicPlayerCommand::Pause),
            "next\n" => Some(MusicPlayerCommand::NextTrack),
            _ => None,
        };
        if let Some(command) = maybe_command {
            info!("Sending command {:?}", command);
            let _ = tx.send(command);
        } else {
            error!("Couldn't interpret command {:?}", command_string);
        }
    };
    loop {
        let mut input = String::new();
        match std::io::stdin().read_line(&mut input) {
            Ok(_) => try_send_command(&input[..]),
            Err(error) => error!("error: {}", error),
        }
        info!("Read: {}", input);
    }
}
