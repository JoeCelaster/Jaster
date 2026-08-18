use std::{
    sync::{
        Arc, RwLock,
        atomic::{AtomicBool, Ordering},
        mpsc::{self, SyncSender},
    },
    thread,
    time::Duration,
};

use rodio::{
    OutputStream, OutputStreamBuilder, Source,
    cpal::{
        self,
        traits::{DeviceTrait, HostTrait},
    },
    mixer::Mixer,
};

use crate::audio::player::AudioPlayer;

type Error = Box<dyn std::error::Error>;

/// How often the daemon re-asks the OS which output device is the default one.
/// Cheap enough to do twice a second, and fast enough that plugging headphones
/// in feels immediate.
const WATCH: Duration = Duration::from_millis(500);

/// The speakers, as the rest of the daemon sees them.
///
/// The output device is not a decision made once at startup: people plug in
/// headphones, unplug them, and pick a different device in Windows' volume
/// flyout, and a stream opened on the old endpoint keeps playing into it — or
/// into nothing at all. So the stream lives on its own thread that keeps
/// watching what the default device is, and this handle points at whichever
/// mixer is current.
#[derive(Clone)]
pub struct AudioEngine {
    /// `None` only in the moment between closing one device and opening the
    /// next; a keystroke then is silent rather than fatal.
    output: Arc<RwLock<Option<Mixer>>>,
}

impl AudioEngine {
    /// Opens the default output device and keeps following it. Returns the
    /// engine and the name of the device it landed on.
    pub fn start() -> Result<(Self, String), Error> {
        let output: Arc<RwLock<Option<Mixer>>> = Arc::new(RwLock::new(None));

        let shared = Arc::clone(&output);
        let (ready, opened) = mpsc::sync_channel(1);

        // The stream itself can never leave this thread: on Windows the WASAPI
        // stream is neither `Send` nor safe to drop from elsewhere.
        thread::spawn(move || follow_default_device(shared, ready));

        match opened.recv() {
            Ok(Ok(device)) => Ok((Self { output }, device)),
            Ok(Err(err)) => Err(err.into()),
            Err(_) => Err("The audio thread stopped before it opened a device".into()),
        }
    }

    /// Mix a clip into the current output. Called once per keystroke, so it
    /// takes a read lock and nothing more.
    pub fn play(&self, sound: impl Source + Send + 'static, gain: f32) {
        let Ok(output) = self.output.read() else {
            return;
        };

        if let Some(mixer) = output.as_ref() {
            AudioPlayer::play(mixer, sound, gain);
        }
    }
}

/// Owns the output stream for as long as the daemon runs, reopening it whenever
/// the default device changes or the current one falls over.
fn follow_default_device(
    output: Arc<RwLock<Option<Mixer>>>,
    ready: SyncSender<Result<String, String>>,
) {
    // Set from cpal's error callback, on cpal's thread, when the device stops
    // accepting samples — an unplugged endpoint is the usual reason.
    let broken = Arc::new(AtomicBool::new(false));

    let mut stream: Option<OutputStream> = None;
    let mut ready = Some(ready);
    let mut playing_on = String::new();

    // What the OS last told us the default was. Compared against itself rather
    // than against the device we hold, because when the default device cannot
    // be opened we fall back to another one and must not then reopen forever.
    let mut default = None;
    let mut complained = false;

    loop {
        let latest = default_name();
        let switched = latest.is_some() && latest != default;

        default = latest;

        let failed = broken.swap(false, Ordering::Relaxed);

        if stream.is_none() || failed || switched {
            // Drop the old stream before opening the next: while we hold it,
            // the device it belongs to is still in use.
            *output.write().unwrap_or_else(|err| err.into_inner()) = None;
            stream = None;

            match open(&broken) {
                Ok((opened, device)) => {
                    *output.write().unwrap_or_else(|err| err.into_inner()) =
                        Some(opened.mixer().clone());

                    stream = Some(opened);
                    complained = false;

                    match ready.take() {
                        Some(ready) => {
                            let _ = ready.send(Ok(device.clone()));
                        }
                        None if device != playing_on => {
                            println!("🔊 Output moved to {device}");
                        }
                        None => {}
                    }

                    playing_on = device;
                }
                Err(err) => {
                    // Failing the very first open is fatal — there is nothing
                    // for the daemon to do without an output device.
                    if let Some(ready) = ready.take() {
                        let _ = ready.send(Err(err.to_string()));
                        return;
                    }

                    if !complained {
                        eprintln!("⚠ No audio output device: {err}");
                        complained = true;
                    }

                    playing_on.clear();
                }
            }
        }

        thread::sleep(WATCH);
    }
}

fn default_name() -> Option<String> {
    cpal::default_host()
        .default_output_device()
        .and_then(|device| device.name().ok())
}

/// Open the default device, falling back to any other device that will take us
/// — the same order [`rodio::OutputStreamBuilder::open_default_stream`] uses,
/// but with an error callback attached so we hear about a device going away.
fn open(broken: &Arc<AtomicBool>) -> Result<(OutputStream, String), Error> {
    let host = cpal::default_host();

    let default = host.default_output_device();
    let fallbacks = host.output_devices().ok().into_iter().flatten();

    let mut last: Option<Error> = None;

    for device in default.into_iter().chain(fallbacks) {
        let name = device.name().unwrap_or_else(|_| "unnamed device".into());

        let flag = Arc::clone(broken);

        let report = move |err: cpal::StreamError| {
            eprintln!("⚠ Audio stream error: {err}");
            flag.store(true, Ordering::Relaxed);
        };

        let opened = OutputStreamBuilder::from_device(device)
            .map_err(Error::from)
            .and_then(|builder| {
                builder
                    .with_error_callback(report)
                    .open_stream_or_fallback()
                    .map_err(Error::from)
            });

        match opened {
            Ok(mut stream) => {
                // We close and reopen streams on purpose whenever the default
                // device changes; rodio's parting note about playback stopping
                // would just be noise in the daemon log.
                stream.log_on_drop(false);

                return Ok((stream, name));
            }
            Err(err) => last = Some(err),
        }
    }

    Err(last.unwrap_or_else(|| "No audio output device".into()))
}
