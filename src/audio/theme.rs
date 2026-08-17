use serde::Deserialize;
use serde_json::Value;
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use crate::keyboard::Key;
use crate::utils::paths;

pub const DEFAULT_PACK: &str = "nk-cream";

#[derive(Deserialize)]
struct RawConfig {
    name: Option<String>,
    sound: Option<String>,
    defines: HashMap<String, Value>,
}

/// How a single key is produced from the pack's files.
pub enum Define {
    /// One file per key ("multi" packs).
    File(String),
    /// A slice of the pack's single sound sheet ("single" packs), in milliseconds.
    Slice { offset_ms: u64, duration_ms: u64 },
}

pub struct SoundPack {
    pub id: String,
    pub name: String,
    pub dir: PathBuf,
    pub sheet: Option<PathBuf>,
    pub defines: HashMap<Key, Define>,
}

pub fn sound_root() -> PathBuf {
    paths::sound_root()
}

fn to_define(value: &Value) -> Option<Define> {
    match value {
        Value::String(file) => Some(Define::File(file.clone())),
        Value::Array(range) => {
            let offset_ms = range.first()?.as_f64()?;
            let duration_ms = range.get(1)?.as_f64()?;

            if offset_ms < 0.0 || duration_ms <= 0.0 {
                return None;
            }

            Some(Define::Slice {
                offset_ms: offset_ms as u64,
                duration_ms: duration_ms as u64,
            })
        }
        _ => None,
    }
}

impl SoundPack {
    pub fn load(dir: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let id = dir
            .file_name()
            .ok_or("Invalid sound pack directory")?
            .to_string_lossy()
            .to_string();

        let raw: RawConfig = serde_json::from_str(&fs::read_to_string(dir.join("config.json"))?)?;

        let mut defines = HashMap::new();

        for (code, value) in &raw.defines {
            let Ok(code) = code.parse::<u32>() else {
                continue;
            };

            let Some(key) = Key::from_pack_code(code) else {
                continue;
            };

            if let Some(define) = to_define(value) {
                // Several encodings can name the same physical key. They agree
                // in every pack we ship, so keep the first and stay
                // deterministic rather than racing the hasher.
                defines.entry(key).or_insert(define);
            }
        }

        if defines.is_empty() {
            return Err(format!("Sound pack '{id}' defines no usable keys").into());
        }

        let name = raw.name.unwrap_or_else(|| id.clone());
        let sheet = raw.sound.map(|file| dir.join(file));

        Ok(Self {
            id,
            name,
            dir: dir.to_path_buf(),
            sheet,
            defines,
        })
    }
}

/// Every readable sound pack under the sound root, sorted by id.
pub fn available() -> Vec<SoundPack> {
    let Ok(entries) = fs::read_dir(sound_root()) else {
        return Vec::new();
    };

    let mut packs: Vec<SoundPack> = entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.join("config.json").exists())
        .filter_map(|path| SoundPack::load(&path).ok())
        .collect();

    packs.sort_by(|a, b| a.id.cmp(&b.id));

    packs
}

/// One-word names for the packs Jaster ships, so `jaster oreo` works. The first
/// entry for a pack is the one advertised by `jaster sounds`. Add a line here
/// when adding a pack to `assets/sounds` — `tests/sound_packs.rs` checks that
/// every installed pack has a shortcut, and that none of them shadow a command.
pub const SHORTCUTS: &[(&str, &str)] = &[
    ("nkcream", "nk-cream"),
    ("cream", "nk-cream"),
    ("black", "cherrymx-black-pbt"),
    ("blue", "cherrymx-blue-pbt"),
    ("brown", "cherrymx-brown-pbt"),
    ("red", "cherrymx-red-pbt"),
    ("crystal", "eg-crystal-purple"),
    ("oreo", "eg-oreo"),
    ("topre", "topre-purple-hybrid-pbt"),
];

/// Lowercase, letters and digits only, so `NK Cream`, `nk-cream` and `nkcream`
/// all compare equal.
fn normalize(value: &str) -> String {
    value
        .chars()
        .filter(|character| character.is_alphanumeric())
        .flat_map(|character| character.to_lowercase())
        .collect()
}

/// The shortcut to advertise for a pack, if it has one.
pub fn shortcut(id: &str) -> Option<&'static str> {
    SHORTCUTS
        .iter()
        .find(|(_, pack)| *pack == id)
        .map(|(alias, _)| *alias)
}

/// Resolve a pack from an id, a display name, a shortcut such as `oreo`, or any
/// unambiguous fragment of those. Errors carry a message ready to show the user.
pub fn find(query: &str) -> Result<SoundPack, String> {
    let query = query.trim();
    let wanted = normalize(query);
    let mut packs = available();

    if packs.is_empty() {
        return Err(format!(
            "No sound packs found in {}",
            sound_root().display()
        ));
    }

    if wanted.is_empty() {
        return Err(format!("Name a sound pack. Available: {}", ids(&packs)));
    }

    let exact = packs
        .iter()
        .position(|pack| normalize(&pack.id) == wanted || normalize(&pack.name) == wanted);

    if let Some(index) = exact {
        return Ok(packs.swap_remove(index));
    }

    if let Some((_, id)) = SHORTCUTS.iter().find(|(alias, _)| *alias == wanted)
        && let Some(index) = packs.iter().position(|pack| pack.id == *id)
    {
        return Ok(packs.swap_remove(index));
    }

    // A prefix is the more predictable match, so only fall back to a fragment
    // ("blue", "oreo") when no pack starts with what was typed.
    let by_prefix = matching(&packs, |candidate| candidate.starts_with(&wanted));

    let matches = if by_prefix.is_empty() {
        matching(&packs, |candidate| candidate.contains(&wanted))
    } else {
        by_prefix
    };

    match matches.as_slice() {
        [index] => Ok(packs.swap_remove(*index)),
        [] => Err(format!(
            "Unknown sound pack '{query}'. Available: {}",
            ids(&packs)
        )),
        several => Err(format!(
            "'{query}' matches several sound packs: {}",
            several
                .iter()
                .map(|index| {
                    let pack = &packs[*index];
                    shortcut(&pack.id).unwrap_or(&pack.id).to_string()
                })
                .collect::<Vec<_>>()
                .join(", ")
        )),
    }
}

fn matching(packs: &[SoundPack], predicate: impl Fn(&str) -> bool) -> Vec<usize> {
    packs
        .iter()
        .enumerate()
        .filter(|(_, pack)| predicate(&normalize(&pack.id)) || predicate(&normalize(&pack.name)))
        .map(|(index, _)| index)
        .collect()
}

fn ids(packs: &[SoundPack]) -> String {
    packs
        .iter()
        .map(|pack| shortcut(&pack.id).unwrap_or(&pack.id).to_string())
        .collect::<Vec<_>>()
        .join(", ")
}

fn selection_file() -> Option<PathBuf> {
    Some(paths::data_dir()?.join("sound-pack"))
}

pub fn save_selection(id: &str) {
    let Some(path) = selection_file() else {
        return;
    };

    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }

    let _ = fs::write(path, id);
}

pub fn load_selection() -> Option<String> {
    let contents = fs::read_to_string(selection_file()?).ok()?;
    let id = contents.trim().to_string();

    if id.is_empty() { None } else { Some(id) }
}

/// The pack to play: an explicit request, else the last selection, else the
/// default pack, else whatever is installed.
pub fn resolve(requested: Option<&str>) -> Result<SoundPack, Box<dyn std::error::Error>> {
    if let Some(requested) = requested {
        return find(requested).map_err(Into::into);
    }

    if let Some(pack) = load_selection().and_then(|id| find(&id).ok()) {
        return Ok(pack);
    }

    if let Ok(pack) = find(DEFAULT_PACK) {
        return Ok(pack);
    }

    available()
        .into_iter()
        .next()
        .ok_or_else(|| format!("No sound packs found in {}", sound_root().display()).into())
}
