use kira::{
    manager::{backend::DefaultBackend, AudioManager, AudioManagerSettings},
    sound::{
        static_sound::{StaticSoundData, StaticSoundHandle, StaticSoundSettings},
        PlaybackState,
    },
    tween::Tween,
};

#[derive(rust_embed::RustEmbed)]
#[folder = "assets/music/output"]
#[include = "a_so_close.ogg"]
struct OggFiles;

pub struct Music {
    manager: AudioManager,
    sound_data: StaticSoundData,
    sound_handle: Option<StaticSoundHandle>,
}

impl Music {
    pub fn new(path: &str) -> Result<Option<Music>, Box<dyn std::error::Error>> {
        let cursor = std::io::Cursor::new(OggFiles::get(path).unwrap().data);
        let manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())?;
        let sound_data = StaticSoundData::from_cursor(cursor, StaticSoundSettings::default())?;

        return Result::Ok(Some(Music {
            manager: manager,
            sound_data: sound_data,
            sound_handle: None,
        }));
    }

    pub fn start_song(&mut self) {
        self.sound_handle = self.manager.play(self.sound_data.clone()).ok();
    }

    pub fn toggle_song(&mut self) {
        if let Some(h) = &mut self.sound_handle {
            match h.state() {
                PlaybackState::Paused => {
                    let _ = h.resume(Tween::default());
                }
                PlaybackState::Playing => {
                    let _ = h.pause(Tween::default());
                }
                PlaybackState::Stopped => {
                    self.start_song();
                }
                _ => {}
            }
        } else {
            self.start_song();
        }
    }
}

pub async fn get_music() -> String {
    #[cfg(not(target_arch = "wasm32"))]
    {}
    #[cfg(target_arch = "wasm32")]
    {
        log::info!("Looking for answers 1");
        let resp = reqwest::get("http://google.com").await.unwrap().text().await.unwrap();
        log::info!("Looking for answers 2");
        return resp;
    }
}
