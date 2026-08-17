use rodio::{
    ChannelCount,
    SampleRate,
    Sink,
    Source,
    mixer::Mixer,
};
use std::time::Duration;

/// Where the limiter starts easing samples back. Below this they pass through
/// untouched, so ordinary typing is bit-for-bit what the pack recorded.
const KNEE: f32 = 0.8;

pub struct AudioPlayer;

impl AudioPlayer {
    /// `gain` is the user's volume; the clips themselves are already levelled to
    /// a common loudness when the pack is loaded.
    pub fn play(mixer: &Mixer, sound: impl Source + Send + 'static, gain: f32) {
        let sink = Sink::connect_new(mixer);

        sink.append(Limited {
            inner: sound.amplify(gain),
        });

        sink.detach();
    }
}

/// A soft limiter. Volumes above 100% push the loudest key transients past what
/// the output can carry, and hard clipping there sounds like a crackle; easing
/// the peaks down instead keeps the extra loudness clean.
struct Limited<S> {
    inner: S,
}

/// The limiter's transfer curve: identity below the knee, easing asymptotically
/// towards 1.0 above it, so no gain can push a sample into hard clipping.
pub fn soften(sample: f32) -> f32 {
    let level = sample.abs();

    if level <= KNEE {
        return sample;
    }

    let headroom = 1.0 - KNEE;
    let limited = KNEE + headroom * ((level - KNEE) / headroom).tanh();

    limited.copysign(sample)
}

impl<S: Source> Iterator for Limited<S> {
    type Item = f32;

    #[inline]
    fn next(&mut self) -> Option<f32> {
        self.inner.next().map(soften)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<S: Source> Source for Limited<S> {
    fn current_span_len(&self) -> Option<usize> {
        self.inner.current_span_len()
    }

    fn channels(&self) -> ChannelCount {
        self.inner.channels()
    }

    fn sample_rate(&self) -> SampleRate {
        self.inner.sample_rate()
    }

    fn total_duration(&self) -> Option<Duration> {
        self.inner.total_duration()
    }
}
