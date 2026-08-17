/// A physical key, identified by its PS/2 set-1 scancode. Extended keys — the
/// ones the wire prefixes with `0xE0` — are stored as `0xE000 | low byte`, so
/// keypad Enter (`E0 1C`) stays distinct from Return (`1C`).
///
/// This is the space sound packs are already written in, which is why
/// `theme.rs` needs no translation table: Windows' `KBDLLHOOKSTRUCT` hands us
/// a scancode plus an "extended" flag directly, and Linux converts from evdev
/// at the device boundary.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Key(pub u16);

impl Key {
    pub const BACKSPACE: Key = Key(0x0E);
    pub const ENTER: Key = Key(0x1C);
    pub const A: Key = Key(0x1E);
    pub const SPACE: Key = Key(0x39);
    pub const UP: Key = Key(0xE048);
    pub const LEFT: Key = Key(0xE04B);
    pub const RIGHT: Key = Key(0xE04D);
    pub const DOWN: Key = Key(0xE050);

    pub const fn new(scancode: u8, extended: bool) -> Self {
        if extended {
            Key(0xE000 | scancode as u16)
        } else {
            Key(scancode as u16)
        }
    }

    /// Read a sound pack's `defines` key.
    ///
    /// Packs in the wild carry the same physical key under any of three
    /// extended-key encodings — `0x0E00 | low` (the Mechvibes form), a literal
    /// `0xE000 | low`, and a legacy `0xEE00 | low`. They all mean "extended,
    /// low byte is the scancode", so they fold onto one another. Reading only
    /// the first is what used to drop the arrow keys.
    pub fn from_pack_code(code: u32) -> Option<Self> {
        Some(match code {
            0x0001..=0x00FF => Key(code as u16),
            0x0E00..=0x0EFF | 0xE000..=0xE0FF | 0xEE00..=0xEEFF => {
                Key::new(code as u8, true)
            }
            _ => return None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::Key;

    #[test]
    fn plain_scancodes_pass_through() {
        assert_eq!(Key::from_pack_code(30), Some(Key::A));
        assert_eq!(Key::from_pack_code(57), Some(Key::SPACE));
        assert_eq!(Key::from_pack_code(0), None);
    }

    #[test]
    fn every_extended_encoding_folds_together() {
        // Home, written three ways across the packs we ship.
        assert_eq!(Key::from_pack_code(3655), Key::from_pack_code(60999));
        // Up arrow: the two encodings packs actually use for it.
        assert_eq!(Key::from_pack_code(57416), Some(Key::UP));
        assert_eq!(Key::from_pack_code(61000), Some(Key::UP));
        // Keypad Enter must not collide with Return.
        assert_ne!(Key::from_pack_code(3612), Key::from_pack_code(28));
    }
}
