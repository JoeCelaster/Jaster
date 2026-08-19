//! The pure half of the macOS backend: the keycode table and the modifier
//! press/release logic.
//!
//! It lives apart from `macos.rs` so it can be compiled — and tested — on any
//! host. `macos.rs` links against CoreGraphics, so nothing in it can run on the
//! Linux and Windows CI runners, and both pieces below are exactly the kind of
//! thing that fails silently: a wrong scancode does not crash, it just plays
//! the generic click forever.

use crate::keyboard::Key;

pub const FLAG_ALPHA_SHIFT: u64 = 0x0001_0000;
pub const FLAG_SHIFT: u64 = 0x0002_0000;
pub const FLAG_CONTROL: u64 = 0x0004_0000;
pub const FLAG_ALTERNATE: u64 = 0x0008_0000;
pub const FLAG_COMMAND: u64 = 0x0010_0000;

pub const VK_CAPS_LOCK: u16 = 0x39;

/// Where keys with no PS/2 equivalent go — the Fn key, the media keys, the JIS
/// block, and whatever an exotic keyboard invents. `0xE0FF` is a well-formed
/// extended code that no pack defines, so these reach the generic clip the way
/// an unknown key does on Linux, without borrowing a real key's sound.
pub const UNKNOWN: Key = Key(0xE0FF);

/// Every modifier that reaches us as `kCGEventFlagsChanged`, paired with the
/// flag bit that says "something in this group is down". Left and right share a
/// bit, which is the whole reason `transition` below is not a one-liner.
///
/// The Mac-only `fn` key is deliberately absent: it has no set-1 scancode, so
/// there is no sound to give it, and `from_virtual` sends it to [`UNKNOWN`].
pub const MODIFIERS: [(u16, u64); 9] = [
    (0x38, FLAG_SHIFT),     // left shift
    (0x3C, FLAG_SHIFT),     // right shift
    (0x3B, FLAG_CONTROL),   // left control
    (0x3E, FLAG_CONTROL),   // right control
    (0x3A, FLAG_ALTERNATE), // left option
    (0x3D, FLAG_ALTERNATE), // right option
    (0x37, FLAG_COMMAND),   // left command
    (0x36, FLAG_COMMAND),   // right command
    (VK_CAPS_LOCK, FLAG_ALPHA_SHIFT),
];

fn modifier_group(virtual_key: u16) -> Option<u64> {
    MODIFIERS
        .iter()
        .find(|(code, _)| *code == virtual_key)
        .map(|(_, group)| *group)
}

/// macOS virtual keycode to set-1 scancode.
///
/// Unlike evdev there is no identity range to lean on — the Mac layout is its
/// own numbering, ordered by position on an ADB keyboard — so the whole table
/// is spelled out. Anything unrecognised becomes [`UNKNOWN`] and lands on the
/// pack's generic sound.
pub fn from_virtual(code: u16) -> Key {
    Key(match code {
        // Letters
        0x00 => 0x1E, // a
        0x0B => 0x30, // b
        0x08 => 0x2E, // c
        0x02 => 0x20, // d
        0x0E => 0x12, // e
        0x03 => 0x21, // f
        0x05 => 0x22, // g
        0x04 => 0x23, // h
        0x22 => 0x17, // i
        0x26 => 0x24, // j
        0x28 => 0x25, // k
        0x25 => 0x26, // l
        0x2E => 0x32, // m
        0x2D => 0x31, // n
        0x1F => 0x18, // o
        0x23 => 0x19, // p
        0x0C => 0x10, // q
        0x0F => 0x13, // r
        0x01 => 0x1F, // s
        0x11 => 0x14, // t
        0x20 => 0x16, // u
        0x09 => 0x2F, // v
        0x0D => 0x11, // w
        0x07 => 0x2D, // x
        0x10 => 0x15, // y
        0x06 => 0x2C, // z

        // Number row
        0x12 => 0x02, // 1
        0x13 => 0x03, // 2
        0x14 => 0x04, // 3
        0x15 => 0x05, // 4
        0x17 => 0x06, // 5
        0x16 => 0x07, // 6
        0x1A => 0x08, // 7
        0x1C => 0x09, // 8
        0x19 => 0x0A, // 9
        0x1D => 0x0B, // 0
        0x1B => 0x0C, // minus
        0x18 => 0x0D, // equal

        // Punctuation
        0x21 => 0x1A, // left bracket
        0x1E => 0x1B, // right bracket
        0x2A => 0x2B, // backslash
        0x29 => 0x27, // semicolon
        0x27 => 0x28, // quote
        0x32 => 0x29, // grave
        0x2B => 0x33, // comma
        0x2F => 0x34, // period
        0x2C => 0x35, // slash
        0x0A => 0x56, // ISO section, on non-ANSI layouts

        // Editing and whitespace
        0x24 => 0x1C, // return
        0x30 => 0x0F, // tab
        0x31 => 0x39, // space
        0x33 => 0x0E, // delete, which is backspace
        0x35 => 0x01, // escape

        // Modifiers
        0x38 => 0x2A,   // left shift
        0x3C => 0x36,   // right shift
        0x3B => 0x1D,   // left control
        0x3E => 0xE01D, // right control
        0x3A => 0x38,   // left option
        0x3D => 0xE038, // right option
        0x37 => 0xE05B, // left command
        0x36 => 0xE05C, // right command
        0x39 => 0x3A,   // caps lock
        0x6E => 0xE05D, // contextual menu

        // Function row
        0x7A => 0x3B,   // f1
        0x78 => 0x3C,   // f2
        0x63 => 0x3D,   // f3
        0x76 => 0x3E,   // f4
        0x60 => 0x3F,   // f5
        0x61 => 0x40,   // f6
        0x62 => 0x41,   // f7
        0x64 => 0x42,   // f8
        0x65 => 0x43,   // f9
        0x6D => 0x44,   // f10
        0x67 => 0x57,   // f11
        0x6F => 0x58,   // f12
        0x69 => 0xE037, // f13, where a PC keyboard has print screen
        0x6B => 0x46,   // f14, scroll lock
        0x71 => 0x45,   // f15, pause — shares 0x45 with keypad clear below,
                        // exactly as num lock and pause do on the Linux side

        // Navigation
        0x72 => 0xE052, // help, where a PC keyboard has insert
        0x73 => 0xE047, // home
        0x74 => 0xE049, // page up
        0x75 => 0xE053, // forward delete
        0x77 => 0xE04F, // end
        0x79 => 0xE051, // page down
        0x7B => 0xE04B, // left
        0x7C => 0xE04D, // right
        0x7D => 0xE050, // down
        0x7E => 0xE048, // up

        // Keypad
        0x52 => 0x52,   // 0
        0x53 => 0x4F,   // 1
        0x54 => 0x50,   // 2
        0x55 => 0x51,   // 3
        0x56 => 0x4B,   // 4
        0x57 => 0x4C,   // 5
        0x58 => 0x4D,   // 6
        0x59 => 0x47,   // 7
        0x5B => 0x48,   // 8
        0x5C => 0x49,   // 9
        0x41 => 0x53,   // decimal
        0x43 => 0x37,   // multiply
        0x45 => 0x4E,   // plus
        0x4E => 0x4A,   // minus
        0x4B => 0xE035, // divide
        0x4C => 0xE01C, // enter
        0x51 => 0x59,   // equals
        0x47 => 0x45,   // clear, where a PC keyboard has num lock

        // No `other => other` here, unlike `from_evdev`. That fall-through is
        // safe on Linux because evdev 1..=88 *is* set-1; the Mac numbering
        // merely occupies the same range while meaning something else, so a
        // passthrough would hand every unmapped key another key's sound — the
        // Fn key would play F5. Land them on the generic clip instead.
        _ => return UNKNOWN,
    })
}

/// Whether a `kCGEventFlagsChanged` for `virtual_key` is a *press*, updating
/// `down` — our belief about which modifiers are physically held.
///
/// The event fires on both press and release and says which it was nowhere.
/// The flag bit is no help on its own either: it means "some key in this group
/// is down", so left shift and right shift are indistinguishable by flags
/// alone. Hence tracking each key, with the group bit used to resynchronise.
pub fn transition(virtual_key: u16, flags: u64, down: &mut [bool; 128]) -> bool {
    let Some(group) = modifier_group(virtual_key) else {
        return false;
    };

    let index = virtual_key as usize;

    // Caps lock is the odd one: its flag reports the *lock* state rather than
    // whether the key is down, so it flips exactly once per press. Comparing
    // against the last value gives one sound per press, whether or not macOS
    // also sends us the release.
    if virtual_key == VK_CAPS_LOCK {
        let lit = flags & FLAG_ALPHA_SHIFT != 0;
        let changed = down[index] != lit;

        down[index] = lit;

        return changed;
    }

    if flags & group == 0 {
        // Nothing in the group is down any more. Clearing the whole group
        // rather than just this key is what repairs the state after a release
        // we never saw — an app switch or Secure Input can eat one.
        for (code, other) in MODIFIERS {
            if other == group {
                down[code as usize] = false;
            }
        }

        return false;
    }

    // The group is still active, so either this key just went down, or it went
    // up while its twin on the other side holds the flag on.
    if down[index] {
        down[index] = false;
        return false;
    }

    down[index] = true;
    true
}

#[cfg(test)]
mod tests {
    use super::{
        FLAG_ALPHA_SHIFT, FLAG_SHIFT, MODIFIERS, UNKNOWN, VK_CAPS_LOCK, from_virtual, transition,
    };
    use crate::keyboard::Key;
    use std::collections::HashMap;

    const LEFT_SHIFT: u16 = 0x38;
    const RIGHT_SHIFT: u16 = 0x3C;

    #[test]
    fn letters_and_whitespace_land_where_the_packs_are() {
        assert_eq!(from_virtual(0x00), Key::A);
        assert_eq!(from_virtual(0x31), Key::SPACE);
        assert_eq!(from_virtual(0x24), Key::ENTER);
        assert_eq!(from_virtual(0x33), Key::BACKSPACE);
    }

    #[test]
    fn arrows_reach_the_extended_encoding() {
        assert_eq!(from_virtual(0x7E), Key::UP);
        assert_eq!(from_virtual(0x7B), Key::LEFT);
        assert_eq!(from_virtual(0x7C), Key::RIGHT);
        assert_eq!(from_virtual(0x7D), Key::DOWN);
    }

    /// Left and right have to stay distinct, or one side of the keyboard plays
    /// the other's sound.
    #[test]
    fn modifiers_keep_their_sides_apart() {
        assert_eq!(from_virtual(LEFT_SHIFT), Key(0x2A));
        assert_eq!(from_virtual(RIGHT_SHIFT), Key(0x36));
        assert_ne!(from_virtual(0x3B), from_virtual(0x3E)); // control
        assert_ne!(from_virtual(0x3A), from_virtual(0x3D)); // option
        assert_ne!(from_virtual(0x37), from_virtual(0x36)); // command
    }

    /// Keypad enter must not collide with return — the same trap the pack
    /// parser has its own test for.
    #[test]
    fn keypad_stays_distinct_from_the_main_block() {
        assert_ne!(from_virtual(0x4C), from_virtual(0x24));
        assert_ne!(from_virtual(0x4B), from_virtual(0x2C));
    }

    /// The check that really matters on a table this size: whatever the Mac
    /// reports has to land where the pack loader put its sounds. A typo here is
    /// silent — the key just falls back to the generic click — so nothing else
    /// would catch it.
    #[test]
    fn every_mapping_agrees_with_the_pack_parser() {
        for code in 0x00..=0x7Fu16 {
            let key = from_virtual(code);

            assert_eq!(
                Key::from_pack_code(key.0 as u32),
                Some(key),
                "virtual key {code:#04x} maps to {:#06x}, which no pack can express",
                key.0
            );
        }
    }

    /// `transition` looks its group up in this table, so a modifier missing
    /// from it is a modifier that never makes a sound.
    #[test]
    fn every_modifier_is_mapped_and_grouped() {
        for (code, group) in MODIFIERS {
            assert_ne!(group, 0, "modifier {code:#04x} has no flag");
            assert_ne!(
                from_virtual(code),
                Key(code),
                "modifier {code:#04x} falls through the table unmapped"
            );
        }
    }

    #[test]
    fn one_press_one_sound() {
        let mut down = [false; 128];

        assert!(transition(LEFT_SHIFT, FLAG_SHIFT, &mut down));
        assert!(!transition(LEFT_SHIFT, 0, &mut down));
        assert!(transition(LEFT_SHIFT, FLAG_SHIFT, &mut down));
    }

    /// Both shifts held is where a naive "did the flag change" check breaks:
    /// the bit is already on when the second one goes down, and still on when
    /// the first comes up.
    #[test]
    fn twin_modifiers_are_counted_separately() {
        let mut down = [false; 128];

        assert!(transition(LEFT_SHIFT, FLAG_SHIFT, &mut down));
        assert!(transition(RIGHT_SHIFT, FLAG_SHIFT, &mut down));

        // Left comes up while right holds the flag on: no sound, and left must
        // be free to sound again.
        assert!(!transition(LEFT_SHIFT, FLAG_SHIFT, &mut down));
        assert!(transition(LEFT_SHIFT, FLAG_SHIFT, &mut down));

        // Everything up.
        assert!(!transition(LEFT_SHIFT, FLAG_SHIFT, &mut down));
        assert!(!transition(RIGHT_SHIFT, 0, &mut down));

        assert!(transition(LEFT_SHIFT, FLAG_SHIFT, &mut down));
    }

    /// A release eaten by an app switch or Secure Input leaves us believing a
    /// key is held. The next event that clears the group has to unstick it,
    /// otherwise that modifier is silent for the rest of the session.
    #[test]
    fn a_cleared_group_resyncs_a_missed_release() {
        let mut down = [false; 128];

        assert!(transition(LEFT_SHIFT, FLAG_SHIFT, &mut down));

        // Pretend the release never arrived, and the next thing we see is the
        // right shift going down and up.
        assert!(transition(RIGHT_SHIFT, FLAG_SHIFT, &mut down));
        assert!(!transition(RIGHT_SHIFT, 0, &mut down));

        assert!(!down[LEFT_SHIFT as usize]);
        assert!(transition(LEFT_SHIFT, FLAG_SHIFT, &mut down));
    }

    /// Caps lock reports the lock, not the key, so the state it is compared
    /// against has to be the lock too — otherwise switching it back off is
    /// silent.
    #[test]
    fn caps_lock_sounds_on_and_off() {
        let mut down = [false; 128];

        assert!(transition(VK_CAPS_LOCK, FLAG_ALPHA_SHIFT, &mut down));
        // A release event, if macOS sends one, must not double up.
        assert!(!transition(VK_CAPS_LOCK, FLAG_ALPHA_SHIFT, &mut down));

        assert!(transition(VK_CAPS_LOCK, 0, &mut down));
        assert!(!transition(VK_CAPS_LOCK, 0, &mut down));
    }

    /// The failure mode a hundred hand-written arms invites: two Mac keys
    /// pointing at one scancode, so one plays the other's sound. Num lock and
    /// pause genuinely share `0x45` in set-1 — the Linux backend collides them
    /// the same way — and that is the only pair allowed.
    #[test]
    fn no_two_keys_share_a_scancode() {
        let mut seen: HashMap<Key, u16> = HashMap::new();

        for code in 0x00..=0x7Fu16 {
            let key = from_virtual(code);

            if key == UNKNOWN {
                continue;
            }

            if let Some(first) = seen.insert(key, code) {
                let clear_and_f15 = [0x47, 0x71];

                assert!(
                    clear_and_f15.contains(&first) && clear_and_f15.contains(&code),
                    "kVK {first:#04x} and kVK {code:#04x} both map to {key:?}"
                );
            }
        }
    }

    /// Every key on a MacBook has to reach a real sound, checked against the
    /// packs we actually ship rather than a second copy of the table — so it
    /// fails if either side drifts. Parsing a pack reads its config.json
    /// without decoding any audio, so this stays cheap.
    #[test]
    fn every_key_on_a_mac_keyboard_has_a_sound() {
        // A MacBook keyboard by virtual keycode. No keypad, no f13+, no JIS:
        // those are not on the machine most people will run this on.
        const KEYBOARD: &[u16] = &[
            0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0B, 0x0C, 0x0D, 0x0E,
            0x0F, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1A, 0x1B, 0x1C,
            0x1D, 0x1E, 0x1F, 0x20, 0x21, 0x22, 0x23, 0x24, 0x25, 0x26, 0x27, 0x28, 0x29, 0x2A,
            0x2B, 0x2C, 0x2D, 0x2E, 0x2F, 0x30, 0x31, 0x32, 0x33, 0x35, 0x36, 0x37, 0x38, 0x39,
            0x3A, 0x3B, 0x3C, 0x3D, 0x3E, 0x60, 0x61, 0x62, 0x63, 0x64, 0x65, 0x67, 0x6D, 0x6F,
            0x72, 0x73, 0x74, 0x75, 0x77, 0x79, 0x7A, 0x7B, 0x7C, 0x7D, 0x7E,
        ];

        let packs = crate::audio::theme::available();

        assert!(!packs.is_empty(), "no packs found to check against");

        for pack in packs {
            for &code in KEYBOARD {
                let key = from_virtual(code);

                assert!(
                    pack.defines.contains_key(&key),
                    "'{}' has no sound for virtual key {code:#04x} ({key:?})",
                    pack.id
                );
            }
        }
    }

    /// The sentinel has to stay unclaimable, or an exotic key would start
    /// borrowing a real one's sound instead of falling back to the clip.
    #[test]
    fn the_unknown_sentinel_is_never_a_pack_define() {
        for pack in crate::audio::theme::available() {
            assert!(
                !pack.defines.contains_key(&UNKNOWN),
                "'{}' defines the unknown-key sentinel",
                pack.id
            );
        }
    }

    #[test]
    fn ordinary_keys_are_not_modifiers() {
        let mut down = [false; 128];

        assert!(!transition(0x00, FLAG_SHIFT, &mut down)); // a
        assert!(!transition(0x31, 0, &mut down)); // space
    }
}
