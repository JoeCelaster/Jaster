use evdev::KeyCode;
use rodio::{Decoder, Source, buffer::SamplesBuffer};
use std::{
    collections::HashMap,
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
};

use crate::audio::theme::{Define, SoundPack};

/// Every pack is scaled to this RMS so switching packs does not change how loud
/// typing is. Roughly -34 dBFS: the packs' own peaks are what stop it going any
/// higher, and `jaster volume` scales from here.
const TARGET_RMS: f32 = 0.02;

/// Leave a little headroom so normalization never pushes a pack into clipping.
const PEAK_CEILING: f32 = 0.95;

pub struct SoundCache {
    pub sounds: HashMap<KeyCode, SamplesBuffer>,
    pub generic: SamplesBuffer,
}

struct Pcm {
    channels: u16,
    sample_rate: u32,
    samples: Vec<f32>,
}

/// Which decoded audio a key uses. Packs point several keys at one file or one
/// sheet offset, and those keys should share a single decode.
#[derive(Clone, PartialEq, Eq, Hash)]
enum ClipId {
    File(String),
    Slice(u64, u64),
}

fn decode(path: &Path) -> Result<Pcm, Box<dyn std::error::Error>> {
    let decoder = Decoder::try_from(BufReader::new(File::open(path)?))?;

    let channels = decoder.channels();
    let sample_rate = decoder.sample_rate();
    let samples: Vec<f32> = decoder.collect();

    if samples.is_empty() {
        return Err(format!("{} decoded to no audio", path.display()).into());
    }

    Ok(Pcm {
        channels,
        sample_rate,
        samples,
    })
}

/// Slicing a sound sheet mid-waveform clicks, so ease each clip in and out.
fn smooth(samples: &mut [f32], channels: u16, sample_rate: u32) {
    let channels = channels.max(1) as usize;
    let frames = samples.len() / channels;

    if frames < 8 {
        return;
    }

    let ramp = |ms: u64| ((sample_rate as u64 * ms / 1000) as usize).clamp(1, frames / 4);

    let fade_in = ramp(1);
    let fade_out = ramp(8);

    for frame in 0..fade_in {
        let gain = frame as f32 / fade_in as f32;

        for channel in 0..channels {
            samples[frame * channels + channel] *= gain;
        }
    }

    for frame in 0..fade_out {
        let gain = frame as f32 / fade_out as f32;
        let index = frames - 1 - frame;

        for channel in 0..channels {
            samples[index * channels + channel] *= gain;
        }
    }
}

fn slice(sheet: &Pcm, offset_ms: u64, duration_ms: u64) -> Option<Pcm> {
    let channels = sheet.channels.max(1) as u64;
    let sample_rate = sheet.sample_rate as u64;

    let start = (offset_ms * sample_rate / 1000 * channels) as usize;
    let length = (duration_ms * sample_rate / 1000 * channels) as usize;

    let start = start.min(sheet.samples.len());
    let end = (start + length).min(sheet.samples.len());

    if end <= start {
        return None;
    }

    let mut samples = sheet.samples[start..end].to_vec();
    smooth(&mut samples, sheet.channels, sheet.sample_rate);

    Some(Pcm {
        channels: sheet.channels,
        sample_rate: sheet.sample_rate,
        samples,
    })
}

/// How much to scale a whole pack so it sits at [`TARGET_RMS`] without clipping.
/// One factor for the entire pack, so a pack's own loud and quiet keys keep
/// their relationship — only the pack-to-pack difference is evened out.
fn normalization(clips: &HashMap<ClipId, Pcm>) -> f32 {
    let mut energy = 0.0f64;
    let mut samples = 0u64;
    let mut peak = 0.0f32;

    for clip in clips.values() {
        for sample in &clip.samples {
            energy += (*sample as f64) * (*sample as f64);
            peak = peak.max(sample.abs());
        }

        samples += clip.samples.len() as u64;
    }

    if samples == 0 || peak <= 0.0 {
        return 1.0;
    }

    let rms = (energy / samples as f64).sqrt() as f32;

    if rms <= 0.0 {
        return 1.0;
    }

    (TARGET_RMS / rms).min(PEAK_CEILING / peak)
}

impl SoundCache {
    pub fn load(pack: &SoundPack) -> Result<Self, Box<dyn std::error::Error>> {
        let mut clips: HashMap<ClipId, Pcm> = HashMap::new();
        let mut assignments: Vec<(KeyCode, ClipId)> = Vec::new();
        let mut sheet: Option<Pcm> = None;

        for (key, define) in &pack.defines {
            let id = match define {
                Define::File(file) => ClipId::File(file.clone()),
                Define::Slice {
                    offset_ms,
                    duration_ms,
                } => ClipId::Slice(*offset_ms, *duration_ms),
            };

            if !clips.contains_key(&id) {
                let clip = match define {
                    Define::File(file) => match decode(&pack.dir.join(file)) {
                        Ok(clip) => Some(clip),
                        Err(err) => {
                            eprintln!("⚠ Skipping {file}: {err}");
                            None
                        }
                    },
                    Define::Slice {
                        offset_ms,
                        duration_ms,
                    } => {
                        if sheet.is_none() {
                            sheet = Some(decode(sheet_path(pack)?)?);
                        }

                        let sheet = sheet.as_ref().expect("sheet decoded above");

                        slice(sheet, *offset_ms, *duration_ms)
                    }
                };

                let Some(clip) = clip else {
                    continue;
                };

                clips.insert(id.clone(), clip);
            }

            assignments.push((*key, id));
        }

        let gain = normalization(&clips);

        let buffers: HashMap<ClipId, SamplesBuffer> = clips
            .into_iter()
            .map(|(id, mut clip)| {
                for sample in &mut clip.samples {
                    *sample *= gain;
                }

                (
                    id,
                    SamplesBuffer::new(clip.channels, clip.sample_rate, clip.samples),
                )
            })
            .collect();

        let sounds: HashMap<KeyCode, SamplesBuffer> = assignments
            .into_iter()
            .filter_map(|(key, id)| buffers.get(&id).map(|sound| (key, sound.clone())))
            .collect();

        let generic = sounds
            .get(&KeyCode::KEY_A)
            .or_else(|| sounds.get(&KeyCode::KEY_SPACE))
            .or_else(|| sounds.values().next())
            .ok_or_else(|| format!("Sound pack '{}' loaded no playable sounds", pack.id))?
            .clone();

        Ok(Self { sounds, generic })
    }
}

fn sheet_path(pack: &SoundPack) -> Result<&PathBuf, Box<dyn std::error::Error>> {
    pack.sheet
        .as_ref()
        .ok_or_else(|| format!("Sound pack '{}' has no sound file", pack.id).into())
}
