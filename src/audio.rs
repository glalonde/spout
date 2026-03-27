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

/// Synchronous render for native (background thread).
#[cfg(not(target_arch = "wasm32"))]
fn render_track(bytes: &[u8]) -> Option<Vec<f32>> {
    let mut player = oxdz::Oxdz::new(bytes, 44100, "")
        .map_err(|e| log::error!("audio: failed to load track: {e}"))
        .ok()?;

    let mut fi = oxdz::FrameInfo::new();
    let max_time_ms = 300_000.0f32;
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

/// Async render for WASM — yields to the browser event loop every ~200 tracker
/// frames so the game keeps running while the track renders in the background.
#[cfg(target_arch = "wasm32")]
async fn render_track_async(bytes: &[u8]) -> Option<Vec<f32>> {
    let mut player = oxdz::Oxdz::new(bytes, 44100, "")
        .map_err(|e| log::error!("audio: failed to load track: {e}"))
        .ok()?;

    let mut fi = oxdz::FrameInfo::new();
    let max_time_ms = 300_000.0f32;
    let mut out = Vec::new();
    let mut frame_count = 0u32;

    loop {
        player.frame_info(&mut fi);
        if fi.loop_count > 0 || fi.time > max_time_ms {
            break;
        }
        player.play_frame();
        for &s in player.buffer() {
            out.push(s as f32 / 32768.0);
        }
        frame_count += 1;

        // Yield every ~20 tracker frames to keep the game responsive.
        // Each tracker frame produces ~882 samples at 44100 Hz / 50 Hz tick
        // rate, so 20 frames ≈ 17,640 samples ≈ a few ms of CPU work.
        if frame_count % 20 == 0 {
            yield_to_browser().await;
        }
    }

    log::info!(
        "audio: rendered {:.1}s ({} samples)",
        fi.time / 1000.0,
        out.len()
    );
    Some(out)
}

/// Yield to the browser event loop via setTimeout(0).
#[cfg(target_arch = "wasm32")]
async fn yield_to_browser() {
    let promise = js_sys::Promise::new(&mut |resolve, _| {
        // safe: window always exists in a browser context
        let _ = web_sys::window()
            .expect("no window")
            .set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, 0);
    });
    let _ = wasm_bindgen_futures::JsFuture::from(promise).await;
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
        mpsc, Arc, RwLock,
    };

    /// Shared sample buffer that the cpal callback reads from. Swapping the
    /// inner `Vec` is enough to change tracks without rebuilding the stream.
    struct SharedBuffer {
        samples: RwLock<Arc<Vec<f32>>>,
        pos: AtomicUsize,
        finished: AtomicBool,
    }

    impl SharedBuffer {
        fn new() -> Arc<Self> {
            Arc::new(SharedBuffer {
                // Start with a single silent stereo frame so the stream callback
                // always has something to read (avoids division-by-zero on len).
                samples: RwLock::new(Arc::new(vec![0.0, 0.0])),
                pos: AtomicUsize::new(0),
                finished: AtomicBool::new(false),
            })
        }

        fn swap(&self, new_samples: Vec<f32>) {
            *self.samples.write().expect("audio lock poisoned") = Arc::new(new_samples);
            self.pos.store(0, Ordering::Relaxed);
            self.finished.store(false, Ordering::Relaxed);
        }

        fn silence(&self) {
            self.swap(vec![0.0, 0.0]);
        }

        fn read_into(&self, out: &mut [f32]) {
            let buf = self.samples.read().expect("audio lock poisoned").clone();
            let len = buf.len();
            for sample in out.iter_mut() {
                let i = self.pos.fetch_add(1, Ordering::Relaxed);
                if i < len {
                    *sample = buf[i];
                } else {
                    *sample = 0.0;
                    self.finished.store(true, Ordering::Relaxed);
                }
            }
        }
    }

    pub struct NativeBackend {
        _stream: Option<cpal::Stream>,
        buffer: Arc<SharedBuffer>,
        pending: Option<mpsc::Receiver<Vec<f32>>>,
    }

    impl Backend for NativeBackend {
        fn new_backend() -> Self {
            let buffer = SharedBuffer::new();
            let stream = build_cpal_stream(Arc::clone(&buffer));
            NativeBackend {
                _stream: stream,
                buffer,
                pending: None,
            }
        }

        fn start_track(&mut self, index: usize) {
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
            // Swap in silence; the stream keeps running.
            self.buffer.silence();
        }

        fn poll(&mut self, playing: bool) {
            if let Some(rx) = self.pending.take() {
                match rx.try_recv() {
                    Ok(samples) => {
                        self.buffer.swap(samples);
                        if playing {
                            if let Some(stream) = &self._stream {
                                if let Err(e) = stream.play() {
                                    log::error!("audio: failed to start stream: {e}");
                                }
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
                && self.buffer.finished.load(Ordering::Relaxed)
                && self.pending.is_none()
        }
    }

    fn build_cpal_stream(buffer: Arc<SharedBuffer>) -> Option<cpal::Stream> {
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

        device
            .build_output_stream(
                &config,
                move |out: &mut [f32], _| {
                    buffer.read_into(out);
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
    use super::{render_track_async, Backend, TRACKS};
    use std::cell::{Cell, RefCell};
    use std::rc::Rc;
    use wasm_bindgen::JsCast;

    // Pending source node, tagged with a generation counter so that a stale
    // render (from a skipped track) is silently discarded.
    type PendingSource = Rc<RefCell<Option<(usize, web_sys::AudioBufferSourceNode)>>>;

    pub struct WasmBackend {
        pending: PendingSource,
        generation: usize,
        /// Generation counter for the `finished` flag so stale `onended`
        /// callbacks (from stopped sources) don't trigger auto-advance.
        finished_gen: Rc<Cell<usize>>,
        context: Rc<RefCell<Option<web_sys::AudioContext>>>,
        source: Option<web_sys::AudioBufferSourceNode>,
        /// Set by the `onended` callback when the current track finishes.
        /// Stores the generation it was set for, or 0 if not finished.
        finished: Rc<Cell<usize>>,
    }

    impl Backend for WasmBackend {
        fn new_backend() -> Self {
            WasmBackend {
                pending: Rc::new(RefCell::new(None)),
                generation: 0,
                finished_gen: Rc::new(Cell::new(0)),
                context: Rc::new(RefCell::new(None)),
                source: None,
                finished: Rc::new(Cell::new(0)),
            }
        }

        fn start_track(&mut self, index: usize) {
            self.generation += 1;
            self.finished_gen.set(self.generation);
            self.finished.set(0);
            let gen = self.generation;
            let pending = Rc::clone(&self.pending);
            let finished = Rc::clone(&self.finished);
            let finished_gen = Rc::clone(&self.finished_gen);
            let context = Rc::clone(&self.context);
            let track_bytes: &'static [u8] = TRACKS[index];

            // Render + build AudioBuffer + create source all happen async,
            // yielding to the browser event loop periodically so the game
            // doesn't freeze.
            wasm_bindgen_futures::spawn_local(async move {
                let Some(samples) = render_track_async(track_bytes).await else {
                    return;
                };

                // Bail if a newer track was requested while we were rendering.
                if finished_gen.get() != gen {
                    return;
                }

                // Ensure we have an AudioContext.
                {
                    let mut ctx_ref = context.borrow_mut();
                    if ctx_ref.is_none() {
                        match web_sys::AudioContext::new() {
                            Ok(ctx) => *ctx_ref = Some(ctx),
                            Err(e) => {
                                log::error!("audio: AudioContext::new failed: {:?}", e);
                                return;
                            }
                        }
                    }
                }

                let ctx_borrow = context.borrow();
                let ctx = ctx_borrow.as_ref().expect("context just created");

                if let Some(src) = make_source_async(ctx, &samples, gen, finished).await {
                    *pending.borrow_mut() = Some((gen, src));
                }
            });
        }

        fn stop_current(&mut self) {
            if let Some(src) = self.source.take() {
                // Clear onended before stopping so the callback doesn't fire.
                std::ops::Deref::deref(&src).set_onended(None);
                let _ = std::ops::Deref::deref(&src).stop_with_when(0.0);
            }
            self.finished.set(0);
        }

        fn poll(&mut self, playing: bool) {
            // Pick up a ready source node if it belongs to the current generation.
            let pending_entry = {
                let mut p = self.pending.borrow_mut();
                p.take()
            };

            if let Some((gen, src)) = pending_entry {
                if gen == self.generation {
                    // Current generation — start playing.
                    if let Some(old) = self.source.replace(src) {
                        std::ops::Deref::deref(&old).set_onended(None);
                        let _ = std::ops::Deref::deref(&old).stop_with_when(0.0);
                    }
                    // Start playback now that we've adopted the source.
                    let _ = self.source.as_ref().expect("just set").start();
                    log::info!("audio: WASM playback started (gen {})", gen);
                } else {
                    // Stale generation — discard without playing.
                    log::info!("audio: discarding stale source (gen {} != {})", gen, self.generation);
                }
            }

            // Browsers suspend AudioContext until the first user gesture.
            if playing {
                if let Some(ctx) = self.context.borrow().as_ref() {
                    if ctx.state() != web_sys::AudioContextState::Running {
                        let _ = ctx.resume();
                    }
                }
            }
        }

        fn set_playing(&mut self, playing: bool) {
            if let Some(ctx) = self.context.borrow().as_ref() {
                if playing {
                    let _ = ctx.resume();
                } else {
                    let _ = ctx.suspend();
                }
            }
        }

        fn is_finished(&self) -> bool {
            // Only consider finished if the flag was set by the current generation's
            // onended callback (not a stale one from a stopped source).
            self.source.is_some()
                && self.finished.get() == self.finished_gen.get()
                && self.pending.borrow().is_none()
        }
    }

    /// Build an AudioBuffer from interleaved stereo samples. Does NOT start
    /// playback — that's deferred to `poll()` so orphaned sources can't play.
    async fn make_source_async(
        ctx: &web_sys::AudioContext,
        samples: &[f32],
        gen: usize,
        finished: Rc<Cell<usize>>,
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

        // Yield before the expensive copy_to_channel calls.
        super::yield_to_browser().await;

        buffer
            .copy_to_channel(&left, 0)
            .map_err(|e| log::error!("audio: copy_to_channel(L) failed: {:?}", e))
            .ok()?;

        super::yield_to_browser().await;

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
        // Uses the generation counter to distinguish "ended naturally" from
        // "was stopped by stop_current()".
        let onended = wasm_bindgen::closure::Closure::wrap(Box::new(move || {
            finished.set(gen);
        }) as Box<dyn FnMut()>);
        std::ops::Deref::deref(&source).set_onended(Some(onended.as_ref().unchecked_ref()));
        onended.forget();

        let dest = ctx.destination();
        source
            .connect_with_audio_node(&dest)
            .map_err(|e| log::error!("audio: connect failed: {:?}", e))
            .ok()?;

        // Note: source.start() is NOT called here. poll() starts it when
        // it picks up the source, preventing orphaned playback.
        Some(source)
    }
}
