//! Background music player.
//!
//! Pre-renders one full loop of a tracker file to f32 PCM on a background
//! thread, then plays it in a looping cpal stream (native) or Web Audio
//! AudioBuffer (WASM).
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
    let mut indices: Vec<usize> = (0..TRACKS.len()).collect();
    // Seed from wall-clock time so the order differs between launches.
    // fastrand::Rng::new() can be deterministic on some platforms (notably WASM).
    let seed = web_time::SystemTime::now()
        .duration_since(web_time::SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64;
    fastrand::Rng::with_seed(seed).shuffle(&mut indices);
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

// ── Platform backend trait ───────────────────────────────────────────────────

/// Platform-specific audio output. Implementations handle the actual playback
/// machinery; playlist management lives in the shared `AudioPlayer`.
trait Backend {
    fn new_backend() -> Self;
    fn start_track(&mut self, index: usize);
    fn stop_current(&mut self);
    /// Called once per frame. `playing` reflects the shared playing state.
    fn poll(&mut self, playing: bool);
    fn set_playing(&mut self, playing: bool);
    /// Returns true when the current track has finished playing.
    fn is_finished(&self) -> bool;
}

// ── Shared AudioPlayer ──────────────────────────────────────────────────────

pub struct AudioPlayer {
    backend: PlatformBackend,
    playlist: Vec<usize>,
    playlist_pos: usize,
    playing: bool,
}

impl AudioPlayer {
    pub fn new() -> Self {
        let playlist = shuffled_playlist();
        let first = playlist[0];
        let mut player = AudioPlayer {
            backend: PlatformBackend::new_backend(),
            playlist,
            playlist_pos: 0,
            playing: true,
        };
        player.backend.start_track(first);
        player
    }

    pub fn disabled() -> Self {
        AudioPlayer {
            backend: PlatformBackend::new_backend(),
            playlist: shuffled_playlist(),
            playlist_pos: 0,
            playing: false,
        }
    }

    pub fn poll(&mut self) {
        self.backend.poll(self.playing);

        // Auto-advance when the current track finishes.
        if self.playing && self.backend.is_finished() {
            self.next_track();
        }
    }

    pub fn toggle(&mut self) {
        self.playing = !self.playing;
        self.backend.set_playing(self.playing);
    }

    pub fn next_track(&mut self) {
        self.playing = true;
        self.backend.stop_current();
        self.playlist_pos = (self.playlist_pos + 1) % self.playlist.len();
        let next = self.playlist[self.playlist_pos];
        self.backend.start_track(next);
    }
}

// ── Native backend (cpal) ───────────────────────────────────────────────────

#[cfg(not(target_arch = "wasm32"))]
type PlatformBackend = native::NativeBackend;

#[cfg(not(target_arch = "wasm32"))]
mod native {
    use super::{render_track, Backend, TRACKS};
    use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
    use std::sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        mpsc, Arc,
    };

    pub struct NativeBackend {
        _stream: Option<cpal::Stream>,
        /// Set by the cpal callback when playback reaches the end of the buffer.
        finished: Arc<AtomicBool>,
        pending: Option<mpsc::Receiver<Vec<f32>>>,
        /// Old stream kept alive until the next frame so its Drop doesn't
        /// block the frame where we start the new track.
        _retiring: Option<cpal::Stream>,
    }

    impl Backend for NativeBackend {
        fn new_backend() -> Self {
            NativeBackend {
                _stream: None,
                finished: Arc::new(AtomicBool::new(false)),
                pending: None,
                _retiring: None,
            }
        }

        fn start_track(&mut self, index: usize) {
            self._stream = None; // drop current stream immediately
            self.finished.store(false, Ordering::Relaxed);
            let (tx, rx) = mpsc::channel();

            let track_bytes: &'static [u8] = TRACKS[index];
            std::thread::spawn(move || {
                if let Some(samples) = render_track(track_bytes) {
                    let _ = tx.send(samples);
                }
            });

            self.pending = Some(rx);
        }

        fn stop_current(&mut self) {
            // Move to _retiring instead of dropping immediately — cpal stream
            // Drop can block waiting for the audio callback to finish.
            self._retiring = self._stream.take();
            self.finished.store(false, Ordering::Relaxed);
        }

        fn poll(&mut self, playing: bool) {
            // Drop any retired stream from the previous frame.
            self._retiring = None;

            if let Some(rx) = self.pending.take() {
                match rx.try_recv() {
                    Ok(samples) => {
                        if let Some(stream) = build_cpal_stream(samples, Arc::clone(&self.finished))
                        {
                            if playing {
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

        fn set_playing(&mut self, playing: bool) {
            if let Some(stream) = &self._stream {
                if playing {
                    if let Err(e) = stream.play() {
                        log::error!("audio: failed to resume: {e}");
                    }
                } else if let Err(e) = stream.pause() {
                    log::error!("audio: failed to pause: {e}");
                }
            }
        }

        fn is_finished(&self) -> bool {
            self._stream.is_some()
                && self.finished.load(Ordering::Relaxed)
                && self.pending.is_none()
        }
    }

    fn build_cpal_stream(samples: Vec<f32>, finished: Arc<AtomicBool>) -> Option<cpal::Stream> {
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
                        let i = p.fetch_add(1, Ordering::Relaxed);
                        if i < len {
                            *sample = s[i];
                        } else {
                            *sample = 0.0;
                            finished.store(true, Ordering::Relaxed);
                        }
                    }
                },
                |e| log::error!("audio stream error: {e}"),
                None,
            )
            .map_err(|e| log::error!("audio: failed to build stream: {e}"))
            .ok()
    }
}

// ── WASM backend (Web Audio API) ────────────────────────────────────────────

#[cfg(target_arch = "wasm32")]
type PlatformBackend = wasm_audio::WasmBackend;

#[cfg(target_arch = "wasm32")]
mod wasm_audio {
    use super::{render_track, Backend, TRACKS};
    use std::cell::{Cell, RefCell};
    use std::rc::Rc;
    use wasm_bindgen::JsCast;

    // Pending render result, tagged with a generation counter so that a stale
    // render (from a skipped track) is silently discarded.
    type Pending = Rc<RefCell<Option<(usize, Vec<f32>)>>>;

    pub struct WasmBackend {
        pending: Pending,
        generation: usize,
        context: Option<web_sys::AudioContext>,
        source: Option<web_sys::AudioBufferSourceNode>,
        /// Set by the `onended` callback when the current track finishes.
        finished: Rc<Cell<bool>>,
    }

    impl Backend for WasmBackend {
        fn new_backend() -> Self {
            WasmBackend {
                pending: Rc::new(RefCell::new(None)),
                generation: 0,
                context: None,
                source: None,
                finished: Rc::new(Cell::new(false)),
            }
        }

        fn start_track(&mut self, index: usize) {
            self.generation += 1;
            self.finished.set(false);
            let gen = self.generation;
            let pending = Rc::clone(&self.pending);
            let track_bytes: &'static [u8] = TRACKS[index];
            wasm_bindgen_futures::spawn_local(async move {
                if let Some(samples) = render_track(track_bytes) {
                    *pending.borrow_mut() = Some((gen, samples));
                }
            });
        }

        fn stop_current(&mut self) {
            if let Some(src) = self.source.take() {
                let _ = std::ops::Deref::deref(&src).stop_with_when(0.0);
            }
            self.finished.set(false);
        }

        fn poll(&mut self, playing: bool) {
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
                        if let Some(src) = make_source(ctx, &samples, Rc::clone(&self.finished)) {
                            if let Some(old) = self.source.replace(src) {
                                let _ = std::ops::Deref::deref(&old).stop_with_when(0.0);
                            }
                        }
                    }
                    None => match web_sys::AudioContext::new() {
                        Ok(ctx) => {
                            if let Some(src) =
                                make_source(&ctx, &samples, Rc::clone(&self.finished))
                            {
                                self.source = Some(src);
                            }
                            self.context = Some(ctx);
                        }
                        Err(e) => log::error!("audio: AudioContext::new failed: {:?}", e),
                    },
                }
            }

            // Browsers suspend AudioContext until the first user gesture.
            if playing {
                if let Some(ctx) = &self.context {
                    if ctx.state() != web_sys::AudioContextState::Running {
                        let _ = ctx.resume();
                    }
                }
            }
        }

        fn set_playing(&mut self, playing: bool) {
            if let Some(ctx) = &self.context {
                if playing {
                    let _ = ctx.resume();
                } else {
                    let _ = ctx.suspend();
                }
            }
        }

        fn is_finished(&self) -> bool {
            self.source.is_some() && self.finished.get() && self.pending.borrow().is_none()
        }
    }

    /// Fill an AudioBuffer with interleaved-stereo `samples` and start playing it.
    ///
    /// `samples` is `[L0, R0, L1, R1, ...]` at 44 100 Hz, as produced by
    /// `render_track`. Sets `finished` to `true` when playback ends.
    fn make_source(
        ctx: &web_sys::AudioContext,
        samples: &[f32],
        finished: Rc<Cell<bool>>,
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
        source.set_loop(false);

        // Signal when playback ends so poll() can auto-advance.
        let onended = wasm_bindgen::closure::Closure::wrap(Box::new(move || {
            finished.set(true);
        }) as Box<dyn FnMut()>);
        std::ops::Deref::deref(&source).set_onended(Some(onended.as_ref().unchecked_ref()));
        onended.forget();

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
