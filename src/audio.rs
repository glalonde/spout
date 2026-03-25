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

/// Returns a randomly shuffled sequence of track indices covering the whole playlist.
fn shuffled_playlist() -> Vec<usize> {
    use rand::seq::SliceRandom;
    let mut indices: Vec<usize> = (0..TRACKS.len()).collect();
    indices.shuffle(&mut rand::rng());
    indices
}

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
        /// Shuffled play order; wraps around when exhausted.
        playlist: Vec<usize>,
        playlist_pos: usize,
        playing: bool,
    }

    impl AudioPlayer {
        pub fn new() -> Self {
            let playlist = super::shuffled_playlist();
            let first = playlist[0];
            let mut player = AudioPlayer {
                _stream: None,
                pending: None,
                playlist,
                playlist_pos: 0,
                playing: true,
            };
            player.start_track(first);
            player
        }

        pub fn disabled() -> Self {
            AudioPlayer {
                _stream: None,
                pending: None,
                playlist: (0..super::TRACKS.len()).collect(),
                playlist_pos: 0,
                playing: false,
            }
        }

        /// Call once per frame. Picks up rendered PCM and starts the cpal
        /// stream the first time it's called after rendering finishes.
        pub fn poll(&mut self) {
            if let Some(rx) = self.pending.take() {
                match rx.try_recv() {
                    Ok(samples) => {
                        if let Some(stream) = build_cpal_stream(samples) {
                            if self.playing {
                                if let Err(e) = stream.play() {
                                    log::error!("audio: failed to start stream: {e}");
                                }
                            }
                            self._stream = Some(stream);
                        }
                    }
                    Err(mpsc::TryRecvError::Empty) => self.pending = Some(rx),
                    Err(mpsc::TryRecvError::Disconnected) => {
                        log::warn!("audio: render thread exited without sending samples");
                    }
                }
            }
        }

        /// Toggle music on/off. Pauses or resumes the current cpal stream.
        pub fn toggle(&mut self) {
            self.playing = !self.playing;
            if let Some(stream) = &self._stream {
                if self.playing {
                    if let Err(e) = stream.play() {
                        log::error!("audio: failed to resume: {e}");
                    }
                } else if let Err(e) = stream.pause() {
                    log::error!("audio: failed to pause: {e}");
                }
            }
        }

        pub fn next_track(&mut self) {
            self.playlist_pos = (self.playlist_pos + 1) % self.playlist.len();
            let next = self.playlist[self.playlist_pos];
            self.start_track(next);
        }

        fn start_track(&mut self, index: usize) {
            self._stream = None; // drop current stream immediately
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

// ── WASM implementation (Web Audio API) ──────────────────────────────────────

#[cfg(target_arch = "wasm32")]
pub use wasm_audio::AudioPlayer;

#[cfg(target_arch = "wasm32")]
mod wasm_audio {
    use super::{render_track, TRACKS};
    use std::cell::RefCell;
    use std::rc::Rc;

    // Pending render result, tagged with a generation counter so that a stale
    // render (from a skipped track) is silently discarded.
    type Pending = Rc<RefCell<Option<(usize, Vec<f32>)>>>;

    pub struct AudioPlayer {
        pending: Pending,
        generation: usize,
        /// Shuffled play order; wraps around when exhausted.
        playlist: Vec<usize>,
        playlist_pos: usize,
        context: Option<web_sys::AudioContext>,
        source: Option<web_sys::AudioBufferSourceNode>,
        playing: bool,
    }

    impl AudioPlayer {
        pub fn new() -> Self {
            let playlist = super::shuffled_playlist();
            let first = playlist[0];
            let mut player = AudioPlayer {
                pending: Rc::new(RefCell::new(None)),
                generation: 0,
                playlist,
                playlist_pos: 0,
                context: None,
                source: None,
                playing: true,
            };
            player.start_render(first);
            player
        }

        pub fn disabled() -> Self {
            AudioPlayer {
                pending: Rc::new(RefCell::new(None)),
                generation: 0,
                playlist: (0..super::TRACKS.len()).collect(),
                playlist_pos: 0,
                context: None,
                source: None,
                playing: false,
            }
        }

        /// Call once per frame. Picks up rendered PCM, creates/resumes the
        /// AudioContext, and swaps in a new source node when a track is ready.
        pub fn poll(&mut self) {
            // Consume pending samples only if they belong to the current generation.
            let samples = {
                let mut p = self.pending.borrow_mut();
                match p.as_ref() {
                    Some((gen, _)) if *gen == self.generation => p.take().map(|(_, s)| s),
                    _ => None,
                }
            };

            if let Some(samples) = samples {
                match &self.context {
                    Some(ctx) => {
                        // AudioContext already exists — just swap in a new source.
                        if let Some(src) = make_source(ctx, &samples) {
                            if let Some(old) = self.source.replace(src) {
                                // Deref to AudioScheduledSourceNode to call the
                                // non-deprecated stop_with_when.
                                let _ = std::ops::Deref::deref(&old).stop_with_when(0.0);
                            }
                        }
                    }
                    None => {
                        // First track: create the AudioContext.
                        match web_sys::AudioContext::new() {
                            Ok(ctx) => {
                                if let Some(src) = make_source(&ctx, &samples) {
                                    self.source = Some(src);
                                }
                                self.context = Some(ctx);
                            }
                            Err(e) => log::error!("audio: AudioContext::new failed: {:?}", e),
                        }
                    }
                }
            }

            // Browsers suspend AudioContext until the first user gesture.
            // Call resume() every frame until it transitions to Running; it is a
            // no-op once running and is ignored (without error) before a gesture.
            // Only do this when music is enabled.
            if self.playing {
                if let Some(ctx) = &self.context {
                    if ctx.state() != web_sys::AudioContextState::Running {
                        let _ = ctx.resume();
                    }
                }
            }
        }

        pub fn toggle(&mut self) {
            self.playing = !self.playing;
            if let Some(ctx) = &self.context {
                if self.playing {
                    let _ = ctx.resume();
                } else {
                    let _ = ctx.suspend();
                }
            }
        }

        pub fn next_track(&mut self) {
            // Stop the current source immediately; poll() will start the next one
            // once its render completes.
            if let Some(src) = self.source.take() {
                let _ = std::ops::Deref::deref(&src).stop_with_when(0.0);
            }
            self.playlist_pos = (self.playlist_pos + 1) % self.playlist.len();
            let next = self.playlist[self.playlist_pos];
            self.start_render(next);
        }

        fn start_render(&mut self, index: usize) {
            self.generation += 1;
            let gen = self.generation;
            let pending = Rc::clone(&self.pending);
            let track_bytes: &'static [u8] = TRACKS[index];
            wasm_bindgen_futures::spawn_local(async move {
                if let Some(samples) = render_track(track_bytes) {
                    *pending.borrow_mut() = Some((gen, samples));
                }
            });
        }
    }

    /// Fill an AudioBuffer with interleaved-stereo `samples` and start looping it.
    ///
    /// `samples` is `[L0, R0, L1, R1, ...]` at 44 100 Hz, as produced by
    /// `render_track`.
    fn make_source(
        ctx: &web_sys::AudioContext,
        samples: &[f32],
    ) -> Option<web_sys::AudioBufferSourceNode> {
        let num_frames = (samples.len() / 2) as u32;

        let buffer = ctx
            .create_buffer(2, num_frames, 44100.0)
            .map_err(|e| log::error!("audio: create_buffer failed: {:?}", e))
            .ok()?;

        // Deinterleave into separate channel arrays.
        let mut left = Vec::with_capacity(num_frames as usize);
        let mut right = Vec::with_capacity(num_frames as usize);
        for frame in samples.chunks(2) {
            left.push(frame[0]);
            right.push(frame.get(1).copied().unwrap_or(0.0));
        }

        buffer
            .copy_to_channel(&left, 0)
            .map_err(|e| log::error!("audio: copy_to_channel(L) failed: {:?}", e))
            .ok()?;
        buffer
            .copy_to_channel(&right, 1)
            .map_err(|e| log::error!("audio: copy_to_channel(R) failed: {:?}", e))
            .ok()?;

        let source = ctx
            .create_buffer_source()
            .map_err(|e| log::error!("audio: create_buffer_source failed: {:?}", e))
            .ok()?;

        source.set_buffer(Some(&buffer));
        source.set_loop(true);

        let dest = ctx.destination();
        source
            .connect_with_audio_node(&dest)
            .map_err(|e| log::error!("audio: connect failed: {:?}", e))
            .ok()?;

        source
            .start()
            .map_err(|e| log::error!("audio: source.start() failed: {:?}", e))
            .ok()?;

        log::info!("audio: WASM playback started ({} frames)", num_frames);
        Some(source)
    }
}
