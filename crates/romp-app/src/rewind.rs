//! Which games the L2 button rewinds.
//!
//! Most controllers' L2 never reaches the game — a NES pad has two buttons and a SNES
//! pad has four — so handing the button to the rewind costs nothing there. The pads
//! that do use it in play keep it: rewinding is not worth losing a brake or an aim.

use std::collections::BTreeMap;
use std::sync::RwLock;

/// The platforms whose controllers use L2 (or its trigger) in the game itself.
const USES_L2: &[&str] = &["psx", "ps2", "psp", "saturn", "dc"];

/// Whether L2 rewinds a game on this platform unless the game says otherwise.
pub fn default_for(platform: &str) -> bool {
    !USES_L2.contains(&platform)
}

/// The games that were set by hand, keyed by the RomM game id. Only the ones that
/// differ from their platform's default are kept, so the map stays small.
static CHOSEN: RwLock<BTreeMap<i64, bool>> = RwLock::new(BTreeMap::new());

pub fn load_choices(json: Option<&str>) {
    let map: BTreeMap<i64, bool> = json
        .and_then(|t| serde_json::from_str(t).ok())
        .unwrap_or_default();
    if let Ok(mut chosen) = CHOSEN.write() {
        *chosen = map;
    }
}

pub fn choices_json() -> String {
    let map = CHOSEN.read().map(|c| c.clone()).unwrap_or_default();
    serde_json::to_string(&map).expect("rewind choices serialize")
}

/// Whether L2 rewinds this game.
pub fn enabled_for(game_id: i64, platform: &str) -> bool {
    CHOSEN
        .read()
        .ok()
        .and_then(|chosen| chosen.get(&game_id).copied())
        .unwrap_or_else(|| default_for(platform))
}

/// Records the choice for one game, forgetting it when it matches the platform again.
pub fn set_game_rewind(game_id: i64, platform: &str, on: bool) -> bool {
    let Ok(mut chosen) = CHOSEN.write() else {
        return false;
    };
    if on == default_for(platform) {
        chosen.remove(&game_id);
    } else {
        chosen.insert(game_id, on);
    }
    true
}

/// Whether the game should rewind after the button is pressed, as a toggle does.
pub fn toggled(game_id: i64, platform: &str) -> bool {
    !enabled_for(game_id, platform)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_pads_that_use_l2_keep_it() {
        for platform in USES_L2 {
            assert!(!default_for(platform), "{platform}");
        }
        for platform in ["nes", "snes", "genesis", "gba", "arcade", "n64"] {
            assert!(default_for(platform), "{platform}");
        }
    }

    #[test]
    fn a_game_can_be_set_against_its_platform() {
        let id = 900_001;
        assert!(enabled_for(id, "snes"));
        assert!(set_game_rewind(id, "snes", false));
        assert!(!enabled_for(id, "snes"));
        assert!(!choices_json().is_empty());
        // Setting it back to the platform's default forgets it again.
        assert!(set_game_rewind(id, "snes", true));
        assert!(!choices_json().contains("900001"));
    }

    #[test]
    fn a_playstation_game_can_opt_in() {
        let id = 900_002;
        assert!(!enabled_for(id, "psx"));
        assert!(set_game_rewind(id, "psx", true));
        assert!(enabled_for(id, "psx"));
        assert!(set_game_rewind(id, "psx", false));
        assert!(!enabled_for(id, "psx"));
    }

    #[test]
    fn saved_choices_survive_a_reload() {
        let id = 900_003;
        assert!(set_game_rewind(id, "nes", false));
        let json = choices_json();
        load_choices(Some(&json));
        assert!(!enabled_for(id, "nes"));
        load_choices(Some("not json"));
        assert!(enabled_for(id, "nes"));
    }
}
