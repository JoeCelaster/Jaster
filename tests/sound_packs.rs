use evdev::KeyCode;
use jaster::audio::{cache::SoundCache, theme};
use rodio::Source;

#[test]
fn every_installed_pack_loads() {
    let packs = theme::available();

    assert!(!packs.is_empty(), "no sound packs found");

    for pack in packs {
        let cache = SoundCache::load(&pack)
            .unwrap_or_else(|err| panic!("failed to load '{}': {err}", pack.id));

        for key in [
            KeyCode::KEY_A,
            KeyCode::KEY_SPACE,
            KeyCode::KEY_ENTER,
            KeyCode::KEY_BACKSPACE,
        ] {
            assert!(
                cache.sounds.contains_key(&key),
                "'{}' has no sound for {key:?}",
                pack.id
            );
        }

        assert!(
            cache.sounds.len() >= 90,
            "'{}' only mapped {} keys",
            pack.id,
            cache.sounds.len()
        );

        for (key, sound) in &cache.sounds {
            let duration = sound.total_duration().unwrap_or_default();

            assert!(
                duration.as_millis() > 0,
                "'{}' has an empty clip for {key:?}",
                pack.id
            );
        }
    }
}

#[test]
fn single_packs_slice_their_sound_sheet() {
    let pack = theme::find("cherrymx-black-pbt").expect("pack installed");

    let cache = SoundCache::load(&pack).expect("pack loads");
    let key = cache.sounds.get(&KeyCode::KEY_A).expect("a is defined");

    // config.json defines key 30 (a) as 160ms of the sheet.
    let duration = key.total_duration().expect("clip has a known duration");

    assert!(
        (140..=180).contains(&duration.as_millis()),
        "unexpected clip length: {duration:?}"
    );
}

#[test]
fn every_pack_has_a_working_shortcut() {
    let mut seen: Vec<&str> = Vec::new();

    for (alias, id) in theme::SHORTCUTS {
        assert!(
            !seen.contains(alias),
            "shortcut '{alias}' is listed twice"
        );

        seen.push(alias);

        assert_eq!(
            theme::find(alias).map(|pack| pack.id).as_deref(),
            Ok(*id),
            "shortcut '{alias}' does not resolve to '{id}'"
        );
    }

    for pack in theme::available() {
        assert!(
            theme::shortcut(&pack.id).is_some(),
            "'{}' has no shortcut — add one to theme::SHORTCUTS",
            pack.id
        );
    }
}

#[test]
fn shortcuts_do_not_shadow_commands() {
    use clap::CommandFactory;

    let commands: Vec<String> = jaster::cli::args::Cli::command()
        .get_subcommands()
        .flat_map(|command| {
            std::iter::once(command.get_name().to_string()).chain(
                command
                    .get_all_aliases()
                    .map(|alias| alias.to_string())
                    .collect::<Vec<_>>(),
            )
        })
        .collect();

    for (alias, _) in theme::SHORTCUTS {
        assert!(
            !commands.contains(&alias.to_string()),
            "shortcut '{alias}' is also a command, so `jaster {alias}` would never switch sounds"
        );
    }
}

#[test]
fn shorthands_resolve_and_ambiguity_is_reported() {
    assert_eq!(
        theme::find("nk").map(|pack| pack.id).as_deref(),
        Ok("nk-cream")
    );

    assert_eq!(
        theme::find("topre").map(|pack| pack.id).as_deref(),
        Ok("topre-purple-hybrid-pbt")
    );

    // Four Cherry packs share the prefix, so it must not silently pick one.
    let ambiguous = theme::find("cherrymx").err().expect("cherrymx is ambiguous");

    // The message names the shortcuts, so it doubles as the fix.
    assert!(
        ["black", "blue", "brown", "red"]
            .iter()
            .all(|shortcut| ambiguous.contains(shortcut)),
        "unhelpful ambiguity message: {ambiguous}"
    );

    assert_eq!(
        theme::find("cherrymx-blue").map(|pack| pack.id).as_deref(),
        Ok("cherrymx-blue-pbt")
    );

    assert!(theme::find("definitely-not-a-pack").is_err());
}

/// Measures each pack the way the ear does — one level for the whole pack — and
/// checks they all land together, so switching packs does not change how loud
/// typing is.
#[test]
fn packs_are_normalized_to_a_common_level() {
    let mut levels: Vec<(String, f64, f32)> = Vec::new();

    for pack in theme::available() {
        let cache = SoundCache::load(&pack).expect("pack loads");

        let mut energy = 0.0f64;
        let mut count = 0u64;
        let mut peak = 0.0f32;

        for sound in cache.sounds.values() {
            for sample in sound.clone() {
                energy += (sample as f64) * (sample as f64);
                peak = peak.max(sample.abs());
                count += 1;
            }
        }

        let rms = (energy / count as f64).sqrt();

        assert!(
            peak <= 0.96,
            "'{}' peaks at {peak:.3}, which clips",
            pack.id
        );

        levels.push((pack.id, 20.0 * rms.log10(), peak));
    }

    let loudest = levels
        .iter()
        .max_by(|a, b| a.1.total_cmp(&b.1))
        .expect("at least one pack");

    let quietest = levels
        .iter()
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .expect("at least one pack");

    assert!(
        loudest.1 - quietest.1 <= 2.0,
        "packs are uneven: {} at {:.1} dBFS vs {} at {:.1} dBFS",
        loudest.0,
        loudest.1,
        quietest.0,
        quietest.1
    );
}

#[test]
fn volume_input_is_forgiving_but_bounded() {
    use jaster::audio::volume;

    assert_eq!(volume::parse("60", 100), Ok(60));
    assert_eq!(volume::parse("60%", 100), Ok(60));
    assert_eq!(volume::parse(" UP ", 100), Ok(110));
    assert_eq!(volume::parse("down", 100), Ok(90));
    assert_eq!(volume::parse("down", 0), Ok(0), "must not wrap below zero");
    assert_eq!(volume::parse("mute", 80), Ok(0));
    assert_eq!(volume::parse("max", 10), Ok(volume::MAX));
    assert_eq!(volume::parse("999", 100), Ok(volume::MAX), "clamped");
    assert_eq!(volume::parse("up", volume::MAX), Ok(volume::MAX));

    assert!(volume::parse("loud", 100).is_err());
    assert!(volume::parse("-5", 100).is_err());
}

/// The default volume is above the level packs are normalized to, so the
/// limiter is what keeps the loudest key transients from crackling.
#[test]
fn loud_volumes_stay_below_clipping() {
    use jaster::audio::{player::soften, volume};

    for quiet in [0.0, 0.1, 0.5, 0.8] {
        assert_eq!(soften(quiet), quiet, "quiet samples must pass through");
        assert_eq!(soften(-quiet), -quiet, "quiet samples must pass through");
    }

    // The worst case Jaster can produce: a pack at the peak ceiling, played at
    // the default volume and at the maximum.
    let ceiling = 0.95;

    for percent in [volume::DEFAULT, volume::MAX] {
        let raw = ceiling * percent as f32 / 100.0;

        assert!(raw > 1.0, "test is pointless unless {percent}% would clip");
        assert!(
            soften(raw) < 1.0,
            "{percent}% clips: {raw} -> {}",
            soften(raw)
        );
        assert!(soften(-raw) > -1.0, "{percent}% clips on the negative side");
    }

    // Louder in must stay louder out, or the shape of a keypress changes. Above
    // the reachable range the curve flattens into f32's resolution, so it only
    // has to stop going back down.
    let reachable = 0.95 * volume::MAX as f32 / 100.0;
    let mut previous = 0.0;

    for step in 1..400 {
        let sample = step as f32 / 100.0;
        let level = soften(sample);

        if sample <= reachable {
            assert!(level > previous, "limiter is not monotonic at {sample}");
        } else {
            assert!(level >= previous, "limiter dips at {sample}");
        }

        previous = level;
    }
}
