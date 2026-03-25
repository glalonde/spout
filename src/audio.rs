//! Background music player.
//!
//! Pre-renders one full loop of a tracker file to f32 PCM on a background
//! thread, then plays it in a looping cpal stream (native) or Web Audio
//! AudioBuffer (WASM, Phase 2).
//!
//! Usage:
//!   let mut player = AudioPlayer::new();   // kicks off background render
//!   // each frame:
//!   player.poll();                         // picks up stream once ready

// Tracker files embedded at compile time (~800 KB total).
const TRACKS: &[&[u8]] = &[
    include_bytes!("../assets/music/aurora.mod"),
    include_bytes!("../assets/music/yoghurt_factory.xm"),
    include_bytes!("../assets/music/aryx.s3m"),
    include_bytes!("../assets/music/brainless_2.mod"),
    include_bytes!("../assets/music/brainless_3.mod"),
    include_bytes!("../assets/music/a_so_close.xm"),
    include_bytes!("../assets/music/BUTTERFL.XM"),
    include_bytes!("../assets/music/sexy3.xm"),
    include_bytes!("../assets/music/MYDICKIN.MOD"),
    include_bytes!("../assets/music/paul.mod"),
    include_bytes!("../assets/music/radix-rainy_summerdays.mod"),
    include_bytes!("../assets/music/spacedeb.mod"),
    include_bytes!("../assets/music/z_bviinaaa.mod"),
];

fn render_track(bytes: &[u8]) -> Option<Vec<f32>> {
    let mut player = oxdz::Oxdz::new(bytes, 44100, "")
        .map_err(|e| log::error!("audio: failed to load track: {e}"))
        .ok()?;

    let mut fi = oxdz::FrameInfo::new();
    let max_time_ms = 300_000.0f32; // 5-minute cap
    let mut out = Vec::new();

    loop {
        player.frame_info(&mut fi);
        if fi.loop_count > 0 || fi.time > max_time_ms {
            break;
        }
        player.play_frame();
        for &s in player.buffer() {
            out.push(s as f32 / 32768.0);
        }
    }

    log::info!(
        "audio: rendered {:.1}s ({} samples)",
        fi.time / 1000.0,
        out.len()
    );
    Some(out)
}

// ── Native implementation ────────────────────────────────────────────────────

#[cfg(not(target_arch = "wasm32"))]
pub use native::AudioPlayer;

#[cfg(not(target_arch = "wasm32"))]
mod native {
    use super::{render_track, TRACKS};
    use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        mpsc, Arc,
    };

    pub struct AudioPlayer {
        /// Held alive to keep the cpal stream running.
        _stream: Option<cpal::Stream>,
        /// Receives rendered PCM from the background thread; stream is built
        /// on the main thread (cpal::Stream is not Send on CoreAudio/macOS).
        pending: Option<mpsc::Receiver<Vec<f32>>>,
        track_index: usize,
    }

    impl AudioPlayer {
        pub fn new() -> Self {
            let mut player = AudioPlayer {
                _stream: None,
                pending: None,
                track_index: 0,
            };
            player.start_track(0);
            player
        }

        pub fn disabled() -> Self {
            AudioPlayer {
                _stream: None,
                pending: None,
                track_index: 0,
            }
        }

        /// Call once per frame. Picks up rendered PCM and starts the cpal
        /// stream the first time it's called after rendering finishes.
        pub fn poll(&mut self) {
            if let Some(rx) = self.pending.take() {
                match rx.try_recv() {
                    Ok(samples) => {
                        if let Some(stream) = build_cpal_stream(samples) {
                            if let Err(e) = stream.play() {
                                log::error!("audio: failed to start stream: {e}");
                            } else {
                                self._stream = Some(stream);
                            }
                        }
                    }
                    Err(mpsc::TryRecvError::Empty) => self.pending = Some(rx),
                    Err(mpsc::TryRecvError::Disconnected) => {
                        log::warn!("audio: render thread exited without sending samples");
                    }
                }
            }
        }

        pub fn next_track(&mut self) {
            let next = (self.track_index + 1) % TRACKS.len();
            self.start_track(next);
        }

        fn start_track(&mut self, index: usize) {
            self._stream = None; // drop current stream immediately
            self.track_index = index;
            let (tx, rx) = mpsc::channel();

            let track_bytes: &'static [u8] = TRACKS[index];
            std::thread::spawn(move || {
                if let Some(samples) = render_track(track_bytes) {
                    let _ = tx.send(samples);
                }
            });

            self.pending = Some(rx);
        }
    }

    fn build_cpal_stream(samples: Vec<f32>) -> Option<cpal::Stream> {
        let host = cpal::default_host();
        let device = host.default_output_device().or_else(|| {
            log::error!("audio: no output device found");
            None
        })?;

        let config = cpal::StreamConfig {
            channels: 2,
            sample_rate: cpal::SampleRate(44100),
            buffer_size: cpal::BufferSize::Default,
        };

        let samples = Arc::new(samples);
        let pos = Arc::new(AtomicUsize::new(0));
        let s = samples.clone();
        let p = pos;

        device
            .build_output_stream(
                &config,
                move |out: &mut [f32], _| {
                    let len = s.len();
                    for sample in out.iter_mut() {
                        let i = p.fetch_add(1, Ordering::Relaxed) % len;
                        *sample = s[i];
                    }
                },
                |e| log::error!("audio stream error: {e}"),
                None,
            )
            .map_err(|e| log::error!("audio: failed to build stream: {e}"))
            .ok()
    }
}

// ── WASM stub (Phase 2 will implement Web Audio) ─────────────────────────────

#[cfg(target_arch = "wasm32")]
pub use wasm_stub::AudioPlayer;

#[cfg(target_arch = "wasm32")]
mod wasm_stub {
    pub struct AudioPlayer;

    impl AudioPlayer {
        pub fn new() -> Self { AudioPlayer }
        pub fn disabled() -> Self { AudioPlayer }
        pub fn poll(&mut self) {}
        pub fn next_track(&mut self) {}
    }
}
