use log::info;
use std::time::Duration;

use rodio::source::Empty;
use rodio::source::Source;
use rodio::source::Zero;

use rodio::Sample;

pub fn make_default_sound_queue() -> SoundQueueController<f32> {
    let device = rodio::default_output_device().unwrap();
    let (controller, queue_rx) = sound_queue(true);
    rodio::play_raw(&device, queue_rx);
    controller
}

#[derive(Debug)]
enum SoundQueueCommand {
    Play,
    Pause,
    Stop,
    NextTrack,
}

pub struct SoundQueueController<S> {
    command_channel: crossbeam_channel::Sender<SoundQueueCommand>,
    sound_channel: crossbeam_channel::Sender<Box<dyn Source<Item = S> + Send>>,
    // Poll this if you want to be notified when a sound has finished. Contains the size of the
    // remaining queue.
    pub sound_finished_rx: crossbeam_channel::Receiver<usize>,
}

impl<S> SoundQueueController<S>
where
    S: Sample + Send + 'static,
{
    /// Adds a new source to the end of the queue.
    #[inline]
    pub fn append_direct<T>(&self, source: T)
    where
        T: Source<Item = S> + Send + 'static,
    {
        let _ = self.sound_channel.send(Box::new(source) as Box<_>);
    }

    #[inline]
    pub fn append<T, D>(&self, source: T)
    where
        D: rodio::Sample,
        T: Source<Item = D> + Send + 'static,
    {
        self.append_direct(source.convert_samples::<S>())
    }

    pub fn pause(&self) {
        let _ = self.command_channel.send(SoundQueueCommand::Pause);
    }

    pub fn play(&self) {
        let _ = self.command_channel.send(SoundQueueCommand::Play);
    }

    pub fn next(&self) {
        let _ = self.command_channel.send(SoundQueueCommand::NextTrack);
    }

    pub fn stop(&self) {
        let _ = self.command_channel.send(SoundQueueCommand::Stop);
    }
}

pub fn sound_queue<S>(keep_alive_if_empty: bool) -> (SoundQueueController<S>, SoundQueueSource<S>)
where
    S: Sample + Send + 'static,
{
    let (cmd_tx, cmd_rx) = crossbeam_channel::unbounded::<SoundQueueCommand>();
    let (source_tx, source_rx) = crossbeam_channel::unbounded::<Box<dyn Source<Item = S> + Send>>();
    let (sound_finished_tx, sound_finished_rx) = crossbeam_channel::unbounded::<usize>();
    let output = SoundQueueSource {
        sound_queue: Vec::new(),
        current: Box::new(Empty::<S>::new()) as Box<_>,
        keep_alive_if_empty,
        command_channel: cmd_rx,
        sound_channel: source_rx,
        sound_finished_tx,
        paused: false,
        playing_silence: false,
    };
    let input = SoundQueueController {
        command_channel: cmd_tx,
        sound_channel: source_tx,
        sound_finished_rx,
    };

    (input, output)
}

/// The input of the queue.
pub struct SoundQueueSource<S> {
    sound_queue: Vec<Box<dyn Source<Item = S> + Send>>,
    current: Box<dyn Source<Item = S> + Send>,
    keep_alive_if_empty: bool,
    command_channel: crossbeam_channel::Receiver<SoundQueueCommand>,
    sound_channel: crossbeam_channel::Receiver<Box<dyn Source<Item = S> + Send>>,
    sound_finished_tx: crossbeam_channel::Sender<usize>,
    paused: bool,
    playing_silence: bool,
}

impl<S> Source for SoundQueueSource<S>
where
    S: Sample + Send + 'static,
{
    #[inline]
    fn current_frame_len(&self) -> Option<usize> {
        // This function is non-trivial because the boundary between two sounds in the
        // queue should be a frame boundary as well.
        //
        // The current sound is free to return `None` for `current_frame_len()`, in
        // which case we *should* return the number of samples remaining the
        // current sound. This can be estimated with `size_hint()`.
        //
        // If the `size_hint` is `None` as well, we are in the worst case scenario. To
        // handle this situation we force a frame to have a maximum number of
        // samples indicate by this constant.
        const THRESHOLD: usize = 512;

        // Try the current `current_frame_len`.
        if let Some(val) = self.current.current_frame_len() {
            if val != 0 {
                return Some(val);
            }
        }

        // Try the size hint.
        if let Some(val) = self.current.size_hint().1 {
            if val < THRESHOLD && val != 0 {
                return Some(val);
            }
        }

        // Otherwise we use the constant value.
        Some(THRESHOLD)
    }

    #[inline]
    fn channels(&self) -> u16 {
        self.current.channels()
    }

    #[inline]
    fn sample_rate(&self) -> u32 {
        self.current.sample_rate()
    }

    #[inline]
    fn total_duration(&self) -> Option<Duration> {
        None
    }
}

impl<S> Iterator for SoundQueueSource<S>
where
    S: Sample + Send + 'static,
{
    type Item = S;

    #[inline]
    fn next(&mut self) -> Option<S> {
        loop {
            // Read command channel.
            self.read_command_channel();

            // Read input channel.
            self.read_sound_channel();

            if self.paused {
                return Some(S::zero_value());
            }

            // Basic situation that will happen most of the time.
            if let Some(sample) = self.current.next() {
                println!("here1");
                return Some(sample);
            }

            // Since `self.current` has finished, we need to pick the next sound.
            // In order to avoid inlining this expensive operation, the code is in another
            // function.
            if self.go_next().is_err() {
                println!("here2");
                return None;
            }
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.current.size_hint().0, None)
    }
}

impl<S> SoundQueueSource<S>
where
    S: Sample + Send + 'static,
{
    fn read_command_channel(&mut self) {
        // Read one command per sample.
        match self.command_channel.try_recv() {
            Ok(command) => self.handle_command(command),
            Err(_) => (),
        }
    }

    fn handle_command(&mut self, command: SoundQueueCommand) {
        println!("Got command! {:?}", command);

        match command {
            SoundQueueCommand::Play => {
                self.paused = false;
            }
            SoundQueueCommand::Pause => {
                self.paused = true;
            }
            SoundQueueCommand::NextTrack => {
                let _ = self.go_next();
            }
            SoundQueueCommand::Stop => {
                self.sound_queue.clear();
                let _ = self.go_next();
            }
        };
    }

    fn read_sound_channel(&mut self) {
        match self.sound_channel.try_recv() {
            Ok(source) => self.sound_queue.push(source),
            Err(_) => (),
        }
    }

    // Called when `current` is empty and we must jump to the next element.
    // Returns `Ok` if the sound should continue playing, or an error if it should
    // stop.
    //
    // This method is separate so that it is not inlined.
    fn go_next(&mut self) -> Result<(), ()> {
        if !self.playing_silence {
            let _ = self.sound_finished_tx.send(self.sound_queue.len());
        }
        let next = {
            if self.sound_queue.len() == 0 {
                if self.keep_alive_if_empty {
                    // Play a short silence in order to avoid spinlocking.
                    let silence = Zero::<S>::new(1, 44100); // TODO: meh
                    self.playing_silence = true;
                    Box::new(silence.take_duration(Duration::from_millis(10))) as Box<_>
                } else {
                    return Err(());
                }
            } else {
                self.playing_silence = false;
                self.sound_queue.remove(0)
            }
        };

        self.current = next;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use rodio::buffer::SamplesBuffer;
    use rodio::source::Source;

    #[test]
    #[ignore] // See upstream rodio::queue if they have fixed this test.
    fn basic() {
        let (tx, mut rx) = super::sound_queue(false);

        tx.append_direct(SamplesBuffer::new(1, 48000, vec![10i16, -10, 10, -10]));
        tx.append_direct(SamplesBuffer::new(2, 96000, vec![5i16, 5, 5, 5]));

        assert_eq!(rx.channels(), 1);
        assert_eq!(rx.sample_rate(), 48000);
        assert_eq!(rx.next(), Some(10));
        assert_eq!(rx.next(), Some(-10));
        assert_eq!(rx.next(), Some(10));
        assert_eq!(rx.next(), Some(-10));
        // assert_eq!(rx.channels(), 2);
        // assert_eq!(rx.sample_rate(), 96000);
        assert_eq!(rx.next(), Some(5));
        assert_eq!(rx.next(), Some(5));
        assert_eq!(rx.next(), Some(5));
        assert_eq!(rx.next(), Some(5));
        assert_eq!(rx.next(), None);
    }

    #[test]
    fn immediate_end() {
        let (_, mut rx) = super::sound_queue::<i16>(false);
        assert_eq!(rx.next(), None);
    }

    #[test]
    fn keep_alive() {
        let (tx, mut rx) = super::sound_queue(true);
        tx.append_direct(SamplesBuffer::new(1, 48000, vec![10i16, -10, 10, -10]));

        assert_eq!(rx.next(), Some(10));
        assert_eq!(rx.next(), Some(-10));
        assert_eq!(rx.next(), Some(10));
        assert_eq!(rx.next(), Some(-10));

        for _ in 0..1000 {
            assert_eq!(rx.next(), Some(0));
        }
    }
}
