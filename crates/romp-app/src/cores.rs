use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::io::Read;
use std::path::PathBuf;
use std::sync::RwLock;

pub const BUILDBOT: &str = "https://buildbot.libretro.com/nightly";
pub const ARMSX2_RELEASE: &str = "https://github.com/RompEmu/ARMSX2/releases/latest/download";
const ARMSX2_ASSET: &str = "armsx2_libretro-macos-arm64.zip";
pub const FAKE08_RELEASE: &str = "https://github.com/RompEmu/fake-08/releases/latest/download";
#[cfg(target_os = "macos")]
const FAKE08_ASSET: &str = "fake08_libretro-macos-arm64.zip";
#[cfg(windows)]
const FAKE08_ASSET: &str = "fake08_libretro-windows-x86_64.zip";
#[cfg(not(any(target_os = "macos", windows)))]
const FAKE08_ASSET: &str = "fake08_libretro-linux-x86_64.zip";

#[derive(Debug, PartialEq, Eq)]
pub struct CoreInfo {
    pub id: &'static str,
    pub name: &'static str,
    pub lib: &'static str,
    pub jit: bool,
}

const fn core(id: &'static str, name: &'static str, lib: &'static str, jit: bool) -> CoreInfo {
    CoreInfo { id, name, lib, jit }
}

const ARMSX2: CoreInfo = core("armsx2", "ARMSX2", "armsx2_libretro", true);
const FAKE08: CoreInfo = core("fake08", "FAKE-08", "fake08_libretro", false);
const FBNEO: CoreInfo = core("fbneo", "FinalBurn Neo", "fbneo_libretro", false);
/// The MAME 2003-plus build handles the same arcade sets as FBNeo in practice, and
/// is the escape when FBNeo refuses a romset — or when its save states misbehave.
const MAME2003_PLUS: CoreInfo = core(
    "mame2003_plus",
    "MAME 2003-Plus",
    "mame2003_plus_libretro",
    false,
);

/// Cores the libretro buildbot doesn't build, published from RompEmu's own forks:
/// (core, where its releases are, this computer's file).
const RELEASED: [(&CoreInfo, &str, &str); 2] = [
    (&ARMSX2, ARMSX2_RELEASE, ARMSX2_ASSET),
    (&FAKE08, FAKE08_RELEASE, FAKE08_ASSET),
];

fn released(core: &CoreInfo) -> Option<(&'static str, &'static str)> {
    RELEASED
        .iter()
        .find(|(c, _, _)| c.id == core.id)
        .map(|(_, base, asset)| (*base, *asset))
}

#[cfg(target_os = "macos")]
const PS2: CoreInfo = ARMSX2;
#[cfg(all(target_arch = "x86_64", not(target_os = "macos")))]
const PS2: CoreInfo = core("pcsx2", "PCSX2", "pcsx2_libretro", true);
#[cfg(not(any(target_os = "macos", target_arch = "x86_64")))]
const PS2: CoreInfo = core("play", "Play!", "play_libretro", true);

static CORES: &[(&[&str], CoreInfo)] = &[
    (
        &["snes", "sfam"],
        core("snes9x", "Snes9x", "snes9x_libretro", false),
    ),
    (
        &["nes", "famicom", "fds"],
        core("nestopia", "Nestopia UE", "nestopia_libretro", false),
    ),
    (
        &["genesis", "sms", "gamegear", "segacd", "sg1000"],
        core(
            "genesis_plus_gx",
            "Genesis Plus GX",
            "genesis_plus_gx_libretro",
            false,
        ),
    ),
    (
        &["gb", "gbc", "gba"],
        core("mgba", "mGBA", "mgba_libretro", false),
    ),
    (
        &["dc"],
        core("flycast", "Flycast", "flycast_libretro", true),
    ),
    (
        &["n64"],
        core(
            "mupen64plus_next",
            "Mupen64Plus-Next",
            "mupen64plus_next_libretro",
            true,
        ),
    ),
    (
        &["tg16"],
        core(
            "mednafen_pce_fast",
            "Beetle PCE Fast",
            "mednafen_pce_fast_libretro",
            false,
        ),
    ),
    (
        &["turbografx-cd"],
        core("mednafen_pce", "Beetle PCE", "mednafen_pce_libretro", false),
    ),
    (
        &["supergrafx"],
        core(
            "mednafen_supergrafx",
            "Beetle SuperGrafx",
            "mednafen_supergrafx_libretro",
            false,
        ),
    ),
    (
        &["psx"],
        core(
            "mednafen_psx_hw",
            "Beetle PSX HW",
            "mednafen_psx_hw_libretro",
            false,
        ),
    ),
    (
        &["pc-fx"],
        core(
            "mednafen_pcfx",
            "Beetle PC-FX",
            "mednafen_pcfx_libretro",
            false,
        ),
    ),
    (
        &["neo-geo-cd"],
        core("neocd", "NeoCD", "neocd_libretro", false),
    ),
    (
        &["saturn"],
        core(
            "mednafen_saturn",
            "Beetle Saturn",
            "mednafen_saturn_libretro",
            false,
        ),
    ),
    (
        &["atari2600"],
        core("stella", "Stella", "stella_libretro", false),
    ),
    (
        &["atari5200"],
        core("a5200", "a5200", "a5200_libretro", false),
    ),
    (
        &["atari7800"],
        core("prosystem", "ProSystem", "prosystem_libretro", false),
    ),
    (
        &["atari800", "atari8bit", "atari-xegs"],
        core("atari800", "Atari800", "atari800_libretro", false),
    ),
    (
        &["neo-geo-pocket", "neo-geo-pocket-color"],
        core(
            "mednafen_ngp",
            "Beetle NeoPop",
            "mednafen_ngp_libretro",
            false,
        ),
    ),
    (
        &["sega32"],
        core("picodrive", "PicoDrive", "picodrive_libretro", false),
    ),
    (
        &["jaguar", "atari-jaguar-cd"],
        core(
            "virtualjaguar",
            "Virtual Jaguar",
            "virtualjaguar_libretro",
            false,
        ),
    ),
    (
        &["wonderswan", "wonderswan-color"],
        core(
            "mednafen_wswan",
            "Beetle WonderSwan",
            "mednafen_wswan_libretro",
            false,
        ),
    ),
    (
        &["virtualboy"],
        core("mednafen_vb", "Beetle VB", "mednafen_vb_libretro", false),
    ),
    (&["vectrex"], core("vecx", "Vecx", "vecx_libretro", false)),
    (
        &["pokemon-mini"],
        core("pokemini", "PokeMini", "pokemini_libretro", false),
    ),
    (
        &["g-and-w"],
        core("gw", "Game & Watch", "gw_libretro", false),
    ),
    (
        &["supervision"],
        core("potator", "Potator", "potator_libretro", false),
    ),
    (
        &["mega-duck-slash-cougar-boy"],
        core("sameduck", "SameDuck", "sameduck_libretro", false),
    ),
    (
        &["fairchild-channel-f"],
        core("freechaf", "FreeChaF", "freechaf_libretro", false),
    ),
    (
        &["tic-80"],
        core("tic80", "TIC-80", "tic80_libretro", false),
    ),
    (&["pico"], FAKE08),
    (
        &["c64"],
        core("vice_x64sc", "VICE x64sc", "vice_x64sc_libretro", false),
    ),
    (
        &["vic-20"],
        core("vice_xvic", "VICE xvic", "vice_xvic_libretro", false),
    ),
    (
        &["c-plus-4", "c16"],
        core("vice_xplus4", "VICE xplus4", "vice_xplus4_libretro", false),
    ),
    (
        &["c128"],
        core("vice_x128", "VICE x128", "vice_x128_libretro", false),
    ),
    (&["zxs"], core("fuse", "Fuse", "fuse_libretro", false)),
    (
        &["amiga", "amiga-cd32", "commodore-cdtv"],
        core("puae", "PUAE", "puae_libretro", false),
    ),
    (
        &["atari-st"],
        core("hatari", "Hatari", "hatari_libretro", false),
    ),
    (
        &["acpc"],
        core("cap32", "Caprice32", "cap32_libretro", false),
    ),
    (
        &["lynx"],
        core(
            "mednafen_lynx",
            "Beetle Lynx",
            "mednafen_lynx_libretro",
            false,
        ),
    ),
    (
        &["intellivision"],
        core("freeintv", "FreeIntv", "freeintv_libretro", false),
    ),
    (&["odyssey-2"], core("o2em", "O2EM", "o2em_libretro", false)),
    (&["psp"], core("ppsspp", "PPSSPP", "ppsspp_libretro", true)),
    (&["3do"], core("opera", "Opera", "opera_libretro", false)),
    (
        &["dos"],
        core("dosbox_pure", "DOSBox Pure", "dosbox_pure_libretro", true),
    ),
    (
        &["nds"],
        core("melondsds", "melonDS DS", "melondsds_libretro", true),
    ),
    (&["ps2"], PS2),
    (
        &["ngc", "wii"],
        core("dolphin", "Dolphin", "dolphin_libretro", true),
    ),
    (
        &["msx", "msx2", "msx2plus"],
        core("bluemsx", "blueMSX", "bluemsx_libretro", false),
    ),
    (
        &["colecovision"],
        core("gearcoleco", "Gearcoleco", "gearcoleco_libretro", false),
    ),
    (
        &["philips-cd-i"],
        core("same_cdi", "SAME CDi", "same_cdi_libretro", false),
    ),
    (&crate::bios::ARCADE, MAME2003_PLUS),
];

pub fn all_cores() -> impl Iterator<Item = &'static CoreInfo> {
    CORES.iter().map(|(_, core)| core)
}

/// The core RomP picks for a platform when the user has not chosen another one.
pub fn default_core_for_platform(slug: &str) -> Option<&'static CoreInfo> {
    CORES
        .iter()
        .find(|(slugs, _)| slugs.contains(&slug))
        .map(|(_, c)| c)
}

/// Cores that can stand in for a platform's default one, in the order the settings
/// offer them. Only cores known to run that platform's games belong here: choosing
/// one routes both the launch and every capability check that follows the core.
///
/// Arcade defaults to MAME 2003-Plus because RomM's arcade folders hold MAME
/// 2003-Plus sets, which FBNeo rejects with "No romset found"; FBNeo stays
/// available for libraries built for it.
pub fn alternates(slug: &str) -> &'static [&'static CoreInfo] {
    static ARCADE: &[&CoreInfo] = &[&MAME2003_PLUS, &FBNEO];
    if crate::bios::ARCADE.contains(&slug) {
        ARCADE
    } else {
        &[]
    }
}

/// The core the user chose per platform, as platform slug -> core id. Kept here
/// rather than threaded through every call site because the whole app asks which
/// core a platform runs — the launch, save states, core options, Vulkan support,
/// split screen and the BIOS directory all read it through `core_for_platform`.
static CHOSEN: RwLock<BTreeMap<String, String>> = RwLock::new(BTreeMap::new());

/// Loads the saved choices, as stored in the app's settings store.
pub fn load_choices(json: Option<&str>) {
    let map: BTreeMap<String, String> = json
        .and_then(|t| serde_json::from_str(t).ok())
        .unwrap_or_default();
    if let Ok(mut chosen) = CHOSEN.write() {
        *chosen = map;
    }
}

pub fn choices_json() -> String {
    let map = CHOSEN.read().map(|c| c.clone()).unwrap_or_default();
    serde_json::to_string(&map).expect("core choices serialize")
}

/// The user's choice for a platform, if it names a core that platform can actually
/// run. A stale or hand-edited value falls back to the default.
pub fn chosen_core(slug: &str) -> Option<&'static CoreInfo> {
    let id = CHOSEN.read().ok()?.get(slug).cloned()?;
    if default_core_for_platform(slug).is_some_and(|c| c.id == id) {
        return default_core_for_platform(slug);
    }
    alternates(slug).iter().find(|c| c.id == id).copied()
}

/// Records a choice; `None` restores the default.
pub fn set_chosen_core(slug: &str, core_id: Option<&str>) -> bool {
    let Ok(mut chosen) = CHOSEN.write() else {
        return false;
    };
    match core_id {
        Some(id) if default_core_for_platform(slug).is_some_and(|c| c.id == id) => {
            chosen.remove(slug);
        }
        Some(id) if alternates(slug).iter().any(|c| c.id == id) => {
            chosen.insert(slug.to_string(), id.to_string());
        }
        Some(_) => return false,
        None => {
            chosen.remove(slug);
        }
    }
    true
}

/// The core that will run this platform's games: the user's choice, else the default.
pub fn core_for_platform(slug: &str) -> Option<&'static CoreInfo> {
    chosen_core(slug).or_else(|| default_core_for_platform(slug))
}

/// Every core that can run a platform, default first. A romset library is rarely
/// all one flavour — RomM's arcade folders mix MAME 2003-Plus and FBNeo sets — so a
/// single game may need a different core from the rest of its platform.
pub fn candidates(slug: &str) -> Vec<&'static CoreInfo> {
    let mut all: Vec<&'static CoreInfo> = Vec::new();
    if let Some(default) = default_core_for_platform(slug) {
        all.push(default);
    }
    for core in alternates(slug) {
        if !all.iter().any(|c| c.id == core.id) {
            all.push(core);
        }
    }
    all
}

/// The per-game choice, keyed by the RomM game id.
static CHOSEN_GAMES: RwLock<BTreeMap<i64, String>> = RwLock::new(BTreeMap::new());

pub fn load_game_choices(json: Option<&str>) {
    let map: BTreeMap<i64, String> = json
        .and_then(|t| serde_json::from_str(t).ok())
        .unwrap_or_default();
    if let Ok(mut chosen) = CHOSEN_GAMES.write() {
        *chosen = map;
    }
}

pub fn game_choices_json() -> String {
    let map = CHOSEN_GAMES.read().map(|c| c.clone()).unwrap_or_default();
    serde_json::to_string(&map).expect("game core choices serialize")
}

/// Records which core runs one game; refused when the platform cannot run it.
pub fn set_game_core(game_id: i64, slug: &str, core_id: Option<&str>) -> bool {
    let Ok(mut chosen) = CHOSEN_GAMES.write() else {
        return false;
    };
    match core_id {
        Some(id) => {
            if !candidates(slug).iter().any(|c| c.id == id) {
                return false;
            }
            chosen.insert(game_id, id.to_string());
        }
        None => {
            chosen.remove(&game_id);
        }
    }
    true
}

/// The core that runs this particular game: the one the user picked for it, else the
/// one its library folder's romsets are built for, else whatever its platform runs on.
pub fn core_for_game(game_id: i64, slug: &str, fs_path: &str) -> Option<&'static CoreInfo> {
    let id = CHOSEN_GAMES
        .read()
        .ok()
        .and_then(|c| c.get(&game_id).cloned());
    id.and_then(|id| candidates(slug).into_iter().find(|c| c.id == id))
        .or_else(|| core_for_folder(fs_path, slug))
        .or_else(|| core_for_platform(slug))
}

/// The RomM library folder a rom sits in: `roms/NEOGEO/turfmast.zip` -> `NEOGEO`.
pub fn folder_of(fs_path: &str) -> Option<&str> {
    let path = fs_path.trim().trim_end_matches('/');
    let folder = path.rsplit('/').next()?.trim();
    (!folder.is_empty() && !folder.eq_ignore_ascii_case("roms")).then_some(folder)
}

/// Which core a folder's romsets belong to. RomM keeps one folder per set family
/// (`MAME 2003-plus`, `FB Neo`, `NEOGEO`, …) while every arcade folder shares the
/// platform slug `arcade`, so for arcade the folder says what the platform cannot.
/// A folder that is not listed here falls back to the platform's own choice.
const FOLDERS: &[(&[&str], &str)] = &[
    (
        &["mame 2003-plus", "mame 2003 plus", "mame2003-plus"],
        "mame2003_plus",
    ),
    (&["fb neo", "fbneo", "fb alpha", "fba", "neogeo"], "fbneo"),
];

/// The core a rom's folder calls for, when its platform can actually run it.
pub fn core_for_folder(fs_path: &str, slug: &str) -> Option<&'static CoreInfo> {
    let folder = folder_of(fs_path)?;
    let id = FOLDERS.iter().find(|(names, _)| {
        names
            .iter()
            .any(|name| name.eq_ignore_ascii_case(folder))
    })?;
    candidates(slug).into_iter().find(|c| c.id == id.1)
}


const NINTENDO: [&str; 11] = [
    "nes",
    "famicom",
    "fds",
    "snes",
    "sfam",
    "n64",
    "gb",
    "gbc",
    "gba",
    "nds",
    "virtualboy",
];

/// FAKE-08 offers save states, but upstream turned off saving the cart's Lua state, so one
/// brings back only the screen and memory while the game carries on where it was.
pub fn saves_states(core_id: &str) -> bool {
    core_id != FAKE08.id
}

pub fn resumes_reliably(core_id: &str) -> bool {
    core_id != "play"
}

pub fn is_computer(core_id: &str) -> bool {
    matches!(
        core_id,
        "puae"
            | "dosbox_pure"
            | "vice_x64sc"
            | "vice_xvic"
            | "vice_xplus4"
            | "vice_x128"
            | "fuse"
            | "atari800"
            | "hatari"
            | "cap32"
    )
}

// These draw through Vulkan where this computer has a working Vulkan device, which on
// macOS is the bundled MoltenVK.
const VULKAN_CORES: [(&str, &[(&str, &str)]); 6] = [
    ("dolphin", &[]),
    ("flycast", &[]),
    (
        "mednafen_psx_hw",
        &[("beetle_psx_hw_renderer", "hardware_vk")],
    ),
    (
        "mupen64plus_next",
        &[("mupen64plus-rdp-plugin", "parallel")],
    ),
    ("ppsspp", &[("ppsspp_backend", "vulkan")]),
    ("pcsx2", &[("pcsx2_renderer", "Vulkan")]),
];

fn vulkan_options(core_id: &str) -> Option<&'static [(&'static str, &'static str)]> {
    if core_id == ARMSX2.id {
        return Some(&[("armsx2_renderer", "Vulkan")]);
    }
    if !crate::vulkan::available() {
        return None;
    }
    VULKAN_CORES
        .iter()
        .find(|(id, _)| *id == core_id)
        .map(|(_, options)| *options)
}

pub fn uses_vulkan(core_id: &str) -> bool {
    vulkan_options(core_id).is_some()
}

pub fn download_base(core: &CoreInfo) -> &'static str {
    released(core).map_or(BUILDBOT, |(base, _)| base)
}

fn core_url(base: &str, core: &CoreInfo) -> String {
    match released(core) {
        Some((_, asset)) => format!("{base}/{asset}"),
        None => buildbot_url(base, core),
    }
}

/// Cores whose disk list can name zipped disks, which they open themselves.
pub fn reads_zipped_disks(core_id: &str) -> bool {
    core_id == "puae"
}

pub fn uses_mouse(core_id: &str) -> bool {
    matches!(core_id, "puae" | "dosbox_pure" | "hatari")
}

pub fn is_nintendo(platform_slug: &str) -> bool {
    NINTENDO.contains(&platform_slug)
}

pub fn lib_file(core: &CoreInfo) -> String {
    let ext = if cfg!(target_os = "macos") {
        "dylib"
    } else if cfg!(windows) {
        "dll"
    } else {
        "so"
    };
    format!("{}.{ext}", core.lib)
}

pub fn buildbot_url(base: &str, core: &CoreInfo) -> String {
    let target = if cfg!(target_os = "macos") {
        "apple/osx/arm64"
    } else if cfg!(windows) {
        "windows/x86_64"
    } else {
        "linux/x86_64"
    };
    format!("{base}/{target}/latest/{}.zip", lib_file(core))
}

pub fn default_options(core_id: &str) -> Vec<(String, String)> {
    let options: &[(&str, &str)] = match core_id {
        "puae" => &[
            // Automatic picks the model a file's name asks for, such as an A1200 for "AGA".
            ("puae_model", "auto"),
            ("puae_model_fd", "A600"),
            ("puae_kickstart", "auto"),
            ("puae_chipmem_size", "4"),
            ("puae_fastmem_size", "8"),
            ("puae_floppy_speed", "800"),
            ("puae_immediate_blits", "immediate"),
        ],
        "mednafen_psx_hw" => &[("beetle_psx_analog_toggle", "enabled")],
        "vice_x64sc" | "vice_xvic" | "vice_xplus4" | "vice_x128" => {
            &[("vice_drive_true_emulation", "disabled")]
        }
        "nestopia" => &[("nestopia_zapper_device", "lightgun")],
        "pcsx2" => &[
            ("pcsx2_renderer", "OpenGL"),
            ("pcsx2_shared_memory_cards", "disabled"),
        ],
        // Both screens stacked with no gap, which Romp splits into two windows, touched with the mouse.
        "melondsds" => &[
            ("melonds_touch_mode", "touch"),
            ("melonds_show_cursor", "disabled"),
            ("melonds_number_of_screen_layouts", "1"),
            ("melonds_screen_layout1", "top-bottom"),
            ("melonds_screen_gap", "0"),
        ],
        _ => &[],
    };
    let mut options: Vec<(String, String)> = options
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();
    set_options(&mut options, vulkan_options(core_id).unwrap_or_default());
    options
}

fn set_options(options: &mut Vec<(String, String)>, changes: &[(&str, &str)]) {
    for (key, value) in changes {
        match options.iter_mut().find(|(k, _)| k == key) {
            Some(existing) => existing.1 = value.to_string(),
            None => options.push((key.to_string(), value.to_string())),
        }
    }
}

/// A console that shares its core with another one, set up as that console instead.
pub fn with_platform_options(
    platform_slug: &str,
    mut options: Vec<(String, String)>,
) -> Vec<(String, String)> {
    let changes: &[(&str, &str)] = match platform_slug {
        "amiga-cd32" => &[
            ("puae_model", "CD32"),
            ("puae_chipmem_size", "auto"),
            ("puae_fastmem_size", "auto"),
        ],
        "commodore-cdtv" => &[
            ("puae_model", "CDTV"),
            ("puae_chipmem_size", "auto"),
            ("puae_fastmem_size", "auto"),
        ],
        _ => &[],
    };
    set_options(&mut options, changes);
    options
}

#[derive(serde::Serialize, serde::Deserialize)]
struct Manifest {
    id: String,
    file: String,
    sha256: String,
    version: String,
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

pub const SYSTEM_FILES: &str = "https://buildbot.libretro.com/assets/system";

const SYSTEM_FILE_SETS: [(&str, &str, &str); 3] = [
    ("dolphin", "Dolphin.zip", "dolphin-emu/Sys/codehandler.bin"),
    ("bluemsx", "blueMSX.zip", "Databases/msxromdb.xml"),
    ("ppsspp", "PPSSPP.zip", "PPSSPP/ppge_atlas.zim"),
];

fn system_file_set(core: &CoreInfo) -> Option<(&'static str, &'static str)> {
    SYSTEM_FILE_SETS
        .iter()
        .find(|(id, _, _)| *id == core.id)
        .map(|(_, zip, marker)| (*zip, *marker))
}

pub fn system_files_present(core: &CoreInfo, system_dir: &std::path::Path) -> bool {
    system_file_set(core).is_none_or(|(_, marker)| system_dir.join(marker).exists())
}

pub async fn install_system_files(
    http: &reqwest::Client,
    base: &str,
    core: &CoreInfo,
    system_dir: &std::path::Path,
) -> Result<(), String> {
    let Some((zip_name, marker)) = system_file_set(core) else {
        return Ok(());
    };
    if system_dir.join(marker).exists() {
        return Ok(());
    }
    let resp = http
        .get(format!("{base}/{zip_name}"))
        .send()
        .await
        .map_err(|_| tr::tr!("Could not reach the download server"))?;
    if !resp.status().is_success() {
        return Err(format!(
            "{}'s system files are not available ({})",
            core.name,
            resp.status()
        ));
    }
    let bytes = resp.bytes().await.map_err(|e| e.to_string())?;
    let system_dir = system_dir.to_path_buf();
    let marker = PathBuf::from(marker);
    tokio::task::spawn_blocking(move || extract_into(&bytes, &system_dir, &marker))
        .await
        .map_err(|e| e.to_string())?
}

fn extract_into(
    bytes: &[u8],
    dir: &std::path::Path,
    marker: &std::path::Path,
) -> Result<(), String> {
    let mut archive =
        zip::ZipArchive::new(std::io::Cursor::new(bytes)).map_err(|e| e.to_string())?;
    let mut entries = Vec::new();
    for i in 0..archive.len() {
        let entry = archive.by_index(i).map_err(|e| e.to_string())?;
        let name = entry.enclosed_name().ok_or_else(|| {
            format!(
                "unsafe path in download: {}",
                entry.name().unwrap_or_default()
            )
        })?;
        entries.push((i, name, entry.is_dir()));
    }
    entries.sort_by_key(|(_, name, _)| name == marker);
    for (i, name, is_dir) in entries {
        let target = dir.join(&name);
        if is_dir {
            std::fs::create_dir_all(&target).map_err(|e| e.to_string())?;
            continue;
        }
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let mut entry = archive.by_index(i).map_err(|e| e.to_string())?;
        let mut out = std::fs::File::create(&target).map_err(|e| e.to_string())?;
        std::io::copy(&mut entry, &mut out).map_err(|e| e.to_string())?;
    }
    Ok(())
}

async fn published_checksum(http: &reqwest::Client, url: &str) -> Result<String, String> {
    let published = http
        .get(format!("{url}.sha256"))
        .send()
        .await
        .and_then(reqwest::Response::error_for_status)
        .map_err(|_| tr::tr!("Could not check the core download"))?
        .text()
        .await
        .map_err(|e| e.to_string())?;
    Ok(published
        .split_whitespace()
        .next()
        .unwrap_or_default()
        .to_ascii_lowercase())
}

pub struct Cores {
    dir: PathBuf,
}

impl Cores {
    pub fn new(dir: PathBuf) -> Self {
        Self { dir }
    }

    /// Whether a core published as a release has a newer build than the one installed.
    pub async fn update_available(
        &self,
        http: &reqwest::Client,
        base: &str,
        core: &CoreInfo,
    ) -> bool {
        if released(core).is_none() {
            return false;
        }
        let Some(manifest) = std::fs::read(self.dir.join(core.id).join("manifest.json"))
            .ok()
            .and_then(|m| serde_json::from_slice::<Manifest>(&m).ok())
        else {
            return false;
        };
        published_checksum(http, &core_url(base, core))
            .await
            .is_ok_and(|latest| latest != manifest.version)
    }

    pub fn installed(&self, core: &CoreInfo) -> Option<PathBuf> {
        let dir = self.dir.join(core.id);
        let manifest: Manifest =
            serde_json::from_slice(&std::fs::read(dir.join("manifest.json")).ok()?).ok()?;
        let path = dir.join(&manifest.file);
        (sha256_hex(&std::fs::read(&path).ok()?) == manifest.sha256).then_some(path)
    }

    pub async fn install(
        &self,
        http: &reqwest::Client,
        base: &str,
        core: &CoreInfo,
    ) -> Result<PathBuf, String> {
        let url = core_url(base, core);
        let resp = http
            .get(&url)
            .send()
            .await
            .map_err(|_| tr::tr!("Could not reach the core download server"))?;
        if !resp.status().is_success() {
            return Err(tr::tr!(
                "{0} is not available for this computer ({1})",
                core.name,
                resp.status()
            ));
        }
        let mut version = resp
            .headers()
            .get(reqwest::header::LAST_MODIFIED)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("nightly")
            .to_string();
        let zip_bytes = resp.bytes().await.map_err(|e| e.to_string())?;
        if released(core).is_some() {
            version = published_checksum(http, &url).await?;
            if !version.eq_ignore_ascii_case(&sha256_hex(&zip_bytes)) {
                return Err(tr::tr!("The core download did not match its checksum"));
            }
        }
        let file = lib_file(core);
        let lib = tokio::task::spawn_blocking({
            let file = file.clone();
            move || -> Result<Vec<u8>, String> {
                let mut archive = zip::ZipArchive::new(std::io::Cursor::new(zip_bytes))
                    .map_err(|e| e.to_string())?;
                let index = (0..archive.len())
                    .find(|&i| {
                        archive.name_for_index(i).is_some_and(|name| {
                            name.is_ok_and(|name| {
                                std::path::Path::new(name.as_ref()).file_name()
                                    == Some(std::ffi::OsStr::new(&file))
                            })
                        })
                    })
                    .ok_or_else(|| format!("{file} is missing from the download"))?;
                let mut entry = archive.by_index(index).map_err(|e| e.to_string())?;
                let mut out = Vec::new();
                entry.read_to_end(&mut out).map_err(|e| e.to_string())?;
                Ok(out)
            }
        })
        .await
        .map_err(|e| e.to_string())??;
        let dir = self.dir.join(core.id);
        tokio::fs::create_dir_all(&dir)
            .await
            .map_err(|e| e.to_string())?;
        let path = dir.join(&file);
        let tmp = dir.join(format!("{file}.tmp"));
        tokio::fs::write(&tmp, &lib)
            .await
            .map_err(|e| e.to_string())?;
        tokio::fs::rename(&tmp, &path)
            .await
            .map_err(|e| e.to_string())?;
        let manifest = Manifest {
            id: core.id.into(),
            file,
            sha256: sha256_hex(&lib),
            version,
        };
        tokio::fs::write(
            dir.join("manifest.json"),
            serde_json::to_vec_pretty(&manifest).expect("manifest json"),
        )
        .await
        .map_err(|e| e.to_string())?;
        Ok(path)
    }
}

#[cfg(test)]
mod core_choice_tests {
    use super::*;
    use std::sync::Mutex;

    /// The chosen-core map is process-wide, so the tests that change it take turns.
    static TURN: Mutex<()> = Mutex::new(());

    #[test]
    fn arcade_offers_mame_2003_plus_and_fbneo() {
        for slug in crate::bios::ARCADE {
            let ids: Vec<&str> = alternates(slug).iter().map(|c| c.id).collect();
            assert!(ids.contains(&"fbneo"), "{slug} should keep FBNeo");
            assert!(
                ids.contains(&"mame2003_plus"),
                "{slug} should offer MAME 2003-Plus"
            );
            // MAME 2003-Plus sets are what RomM serves, so it is the default.
            assert_eq!(
                ids[0], "mame2003_plus",
                "{slug} should default to MAME 2003-Plus"
            );
        }
    }

    #[test]
    fn a_chosen_core_replaces_the_default() {
        let _turn = TURN.lock().unwrap_or_else(|e| e.into_inner());
        assert!(set_chosen_core("cps1", Some("fbneo")));
        assert_eq!(core_for_platform("cps1").unwrap().id, "fbneo");
        // The default is untouched, and other platforms are unaffected.
        assert_eq!(
            default_core_for_platform("cps1").unwrap().id,
            "mame2003_plus"
        );
        assert_eq!(core_for_platform("snes").unwrap().id, "snes9x");
        assert!(set_chosen_core("cps1", None));
        assert_eq!(core_for_platform("cps1").unwrap().id, "mame2003_plus");
    }

    #[test]
    fn a_core_that_platform_cannot_run_is_refused() {
        let _turn = TURN.lock().unwrap_or_else(|e| e.into_inner());
        assert!(!set_chosen_core("cps1", Some("dolphin")));
        assert_eq!(core_for_platform("cps1").unwrap().id, "mame2003_plus");
    }

    #[test]
    fn one_game_can_run_on_a_different_core_from_its_platform() {
        let _turn = TURN.lock().unwrap_or_else(|e| e.into_inner());
        // RomM's arcade folders mix sets: Gang Wars is a FBNeo set in a
        // MAME 2003-Plus library, so the game overrides its platform.
        assert!(set_game_core(292, "arcade", Some("fbneo")));
        assert_eq!(core_for_game(292, "arcade", "roms/MAME 2003-plus").unwrap().id, "fbneo");
        // Other games on the platform are untouched.
        assert_eq!(core_for_game(391, "arcade", "roms/MAME 2003-plus").unwrap().id, "mame2003_plus");
        // A core the platform cannot run is refused.
        assert!(!set_game_core(292, "arcade", Some("dolphin")));
        assert!(set_game_core(292, "arcade", None));
        assert_eq!(core_for_game(292, "arcade", "roms/MAME 2003-plus").unwrap().id, "mame2003_plus");
    }

    #[test]
    fn game_choices_survive_a_round_trip() {
        let _turn = TURN.lock().unwrap_or_else(|e| e.into_inner());
        assert!(set_game_core(292, "arcade", Some("fbneo")));
        let json = game_choices_json();
        load_game_choices(None);
        assert_eq!(core_for_game(292, "arcade", "roms/MAME 2003-plus").unwrap().id, "mame2003_plus");
        load_game_choices(Some(&json));
        assert_eq!(core_for_game(292, "arcade", "roms/MAME 2003-plus").unwrap().id, "fbneo");
        load_game_choices(None);
    }

    #[test]
    fn a_roms_folder_picks_the_core_its_sets_belong_to() {
        let _turn = TURN.lock().unwrap_or_else(|e| e.into_inner());
        load_game_choices(None);
        // RomM's arcade folders are separate set families behind one `arcade` slug:
        // NEOGEO holds FBNeo's dumps, so it must not fall back to MAME 2003-Plus.
        assert_eq!(
            core_for_game(1306, "arcade", "roms/NEOGEO").unwrap().id,
            "fbneo"
        );
        assert_eq!(
            core_for_game(1163, "arcade", "roms/MAME 2003-plus")
                .unwrap()
                .id,
            "mame2003_plus"
        );
        // A folder nobody has classified leaves the platform's own choice alone.
        assert_eq!(
            core_for_game(1941, "arcade", "roms/CPS1").unwrap().id,
            "mame2003_plus"
        );
        // The rule never routes a platform to a core it cannot run.
        assert_eq!(core_for_game(7, "snes", "roms/NEOGEO").unwrap().id, "snes9x");
    }

    #[test]
    fn a_hand_picked_core_beats_the_folder() {
        let _turn = TURN.lock().unwrap_or_else(|e| e.into_inner());
        assert_eq!(
            core_for_game(1306, "arcade", "roms/NEOGEO").unwrap().id,
            "fbneo"
        );
        assert!(set_game_core(1306, "arcade", Some("mame2003_plus")));
        assert_eq!(
            core_for_game(1306, "arcade", "roms/NEOGEO").unwrap().id,
            "mame2003_plus"
        );
        assert!(set_game_core(1306, "arcade", None));
        assert_eq!(
            core_for_game(1306, "arcade", "roms/NEOGEO").unwrap().id,
            "fbneo",
            "clearing the choice returns the game to its folder's core"
        );
    }

    #[test]
    fn folder_of_names_the_library_folder() {
        assert_eq!(folder_of("roms/NEOGEO"), Some("NEOGEO"));
        assert_eq!(folder_of("roms/MAME 2003-plus/"), Some("MAME 2003-plus"));
        assert_eq!(folder_of(" ROMS - Favourites/roms/FB Neo "), Some("FB Neo"));
        assert_eq!(folder_of("roms/"), None);
        assert_eq!(folder_of(""), None);
        assert_eq!(folder_of("roms"), None);
    }

    #[test]
    fn candidates_lead_with_the_default_and_list_each_core_once() {
        let ids: Vec<&str> = candidates("arcade").iter().map(|c| c.id).collect();
        assert_eq!(ids, vec!["mame2003_plus", "fbneo"]);
        assert_eq!(candidates("snes").len(), 1);
    }

    #[test]
    fn choices_survive_a_round_trip() {
        let _turn = TURN.lock().unwrap_or_else(|e| e.into_inner());
        assert!(set_chosen_core("arcade", Some("fbneo")));
        let json = choices_json();
        load_choices(None);
        assert_eq!(core_for_platform("arcade").unwrap().id, "mame2003_plus");
        load_choices(Some(&json));
        assert_eq!(core_for_platform("arcade").unwrap().id, "fbneo");
        load_choices(None);
        assert_eq!(core_for_platform("arcade").unwrap().id, "mame2003_plus");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn zip_with(name: &str, body: &[u8]) -> Vec<u8> {
        use std::io::Write;
        let mut out = std::io::Cursor::new(Vec::new());
        let mut zip = zip::ZipWriter::new(&mut out);
        zip.start_file(name, zip::write::SimpleFileOptions::default())
            .unwrap();
        zip.write_all(body).unwrap();
        zip.finish().unwrap();
        out.into_inner()
    }

    #[test]
    fn platforms_map_to_cores() {
        assert_eq!(core_for_platform("snes").unwrap().id, "snes9x");
        assert_eq!(
            core_for_platform("psx").unwrap().lib,
            "mednafen_psx_hw_libretro"
        );
        assert!(core_for_platform("dc").unwrap().jit);
        assert!(core_for_platform("xbox").is_none());
        for slug in ["ngc", "wii"] {
            let core = core_for_platform(slug).unwrap();
            assert_eq!((core.id, core.jit), ("dolphin", true));
        }
        for slug in ["msx", "msx2", "msx2plus"] {
            assert_eq!(core_for_platform(slug).unwrap().id, "bluemsx");
        }
        assert_eq!(core_for_platform("colecovision").unwrap().id, "gearcoleco");
        assert_eq!(core_for_platform("atari5200").unwrap().id, "a5200");
        for slug in ["jaguar", "atari-jaguar-cd"] {
            assert_eq!(core_for_platform(slug).unwrap().id, "virtualjaguar");
        }
        assert_eq!(core_for_platform("atari7800").unwrap().id, "prosystem");
        for slug in ["atari800", "atari8bit", "atari-xegs"] {
            assert_eq!(core_for_platform(slug).unwrap().id, "atari800");
        }
        assert!(is_computer("atari800"));
        assert_eq!(core_for_platform("vic-20").unwrap().id, "vice_xvic");
        for slug in ["c-plus-4", "c16"] {
            assert_eq!(core_for_platform(slug).unwrap().id, "vice_xplus4");
        }
        assert_eq!(core_for_platform("c128").unwrap().id, "vice_x128");
        assert_eq!(core_for_platform("atari-st").unwrap().id, "hatari");
        assert_eq!(core_for_platform("pc-fx").unwrap().id, "mednafen_pcfx");
        assert_eq!(core_for_platform("neo-geo-cd").unwrap().id, "neocd");
        for (slug, id) in [
            ("pokemon-mini", "pokemini"),
            ("g-and-w", "gw"),
            ("supervision", "potator"),
            ("mega-duck-slash-cougar-boy", "sameduck"),
            ("fairchild-channel-f", "freechaf"),
            ("tic-80", "tic80"),
        ] {
            assert_eq!(core_for_platform(slug).unwrap().id, id);
        }
        assert!(is_computer("hatari") && uses_mouse("hatari"));
        assert_eq!(core_for_platform("acpc").unwrap().id, "cap32");
        assert!(is_computer("cap32") && !uses_mouse("cap32"));
        assert!(["vice_xvic", "vice_xplus4", "vice_x128"]
            .into_iter()
            .all(|id| is_computer(id)
                && default_options(id)
                    .contains(&("vice_drive_true_emulation".into(), "disabled".into()))));
        assert_eq!(core_for_platform("philips-cd-i").unwrap().id, "same_cdi");
        // RomM's arcade folders hold MAME 2003-Plus sets, so that is the default.
        for slug in ["arcade", "neogeoaes", "cps2"] {
            assert_eq!(core_for_platform(slug).unwrap().id, "mame2003_plus");
        }
    }

    #[test]
    fn cores_draw_with_vulkan_where_it_works() {
        let vulkan = crate::vulkan::available();
        assert!(uses_vulkan("armsx2"));
        assert!(!uses_vulkan("snes9x"));
        for core in [
            "dolphin",
            "flycast",
            "mednafen_psx_hw",
            "mupen64plus_next",
            "ppsspp",
            "pcsx2",
        ] {
            assert_eq!(uses_vulkan(core), vulkan, "{core}");
        }
        let n64 = default_options("mupen64plus_next");
        let rdp = n64.iter().find(|(k, _)| k == "mupen64plus-rdp-plugin");
        assert_eq!(rdp.map(|(_, v)| v.as_str()), vulkan.then_some("parallel"));
        let ps2 = default_options("pcsx2");
        let renderer = ps2.iter().filter(|(k, _)| k == "pcsx2_renderer").count();
        assert_eq!(renderer, 1, "one renderer choice, Vulkan replacing OpenGL");
        assert!(default_options("mednafen_psx_hw")
            .contains(&("beetle_psx_analog_toggle".into(), "enabled".into())));
        assert_eq!(download_base(&ARMSX2), ARMSX2_RELEASE);
        assert_eq!(download_base(core_for_platform("snes").unwrap()), BUILDBOT);
        if cfg!(target_os = "macos") {
            assert_eq!(core_for_platform("ps2").unwrap().id, "armsx2");
        }
    }

    #[test]
    fn pico8_plays_on_fake08_from_romps_own_release() {
        let fake08 = core_for_platform("pico").unwrap();
        assert_eq!((fake08.id, fake08.lib), ("fake08", "fake08_libretro"));
        assert_eq!(download_base(fake08), FAKE08_RELEASE);
        let asset = if cfg!(target_os = "macos") {
            "fake08_libretro-macos-arm64.zip"
        } else if cfg!(windows) {
            "fake08_libretro-windows-x86_64.zip"
        } else {
            "fake08_libretro-linux-x86_64.zip"
        };
        assert_eq!(core_url("https://x", fake08), format!("https://x/{asset}"));
        assert_eq!(
            core_url("https://x", core_for_platform("snes").unwrap()),
            buildbot_url("https://x", core_for_platform("snes").unwrap())
        );
    }

    async fn armsx2_release(zip: Vec<u8>, checksum: String) -> MockServer {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path(format!("/{ARMSX2_ASSET}")))
            .respond_with(ResponseTemplate::new(200).set_body_bytes(zip))
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path(format!("/{ARMSX2_ASSET}.sha256")))
            .respond_with(ResponseTemplate::new(200).set_body_string(checksum))
            .mount(&server)
            .await;
        server
    }

    #[tokio::test]
    async fn release_cores_are_checked_and_found_inside_folders() {
        let zip = zip_with(
            &format!("armsx2-macos-arm64-libretro/{}", lib_file(&ARMSX2)),
            b"ps2",
        );
        let checksum = format!("{}  {ARMSX2_ASSET}\n", sha256_hex(&zip));
        let server = armsx2_release(zip, checksum).await;
        let dir = tempfile::tempdir().unwrap();
        let cores = Cores::new(dir.path().to_path_buf());
        let installed = cores
            .install(&reqwest::Client::new(), &server.uri(), &ARMSX2)
            .await
            .unwrap();
        assert_eq!(std::fs::read(&installed).unwrap(), b"ps2");
        assert!(installed.ends_with(format!("armsx2/{}", lib_file(&ARMSX2))));
    }

    #[tokio::test]
    async fn a_newer_release_core_is_noticed() {
        let zip = zip_with(&lib_file(&ARMSX2), b"ps2");
        let checksum = sha256_hex(&zip);
        let server = armsx2_release(zip, format!("{checksum}  {ARMSX2_ASSET}\n")).await;
        let dir = tempfile::tempdir().unwrap();
        let cores = Cores::new(dir.path().to_path_buf());
        let http = reqwest::Client::new();
        assert!(!cores.update_available(&http, &server.uri(), &ARMSX2).await);
        cores.install(&http, &server.uri(), &ARMSX2).await.unwrap();
        assert!(!cores.update_available(&http, &server.uri(), &ARMSX2).await);

        let newer = armsx2_release(Vec::new(), format!("{}  x\n", "1".repeat(64))).await;
        assert!(cores.update_available(&http, &newer.uri(), &ARMSX2).await);
        let offline = "http://127.0.0.1:9";
        assert!(!cores.update_available(&http, offline, &ARMSX2).await);
    }

    #[tokio::test]
    async fn release_cores_that_do_not_match_their_checksum_are_refused() {
        let zip = zip_with(&lib_file(&ARMSX2), b"ps2");
        let server = armsx2_release(zip, format!("{}  x\n", "0".repeat(64))).await;
        let dir = tempfile::tempdir().unwrap();
        let cores = Cores::new(dir.path().to_path_buf());
        let result = cores
            .install(&reqwest::Client::new(), &server.uri(), &ARMSX2)
            .await;
        assert!(result.is_err());
        assert!(cores.installed(&ARMSX2).is_none());
    }

    #[test]
    fn play_does_not_resume_from_automatic_saves() {
        assert!(!resumes_reliably("play"));
        assert!(!saves_states("fake08"));
        assert!(saves_states("snes9x") && saves_states("play"));
        assert!(resumes_reliably("snes9x"));
    }

    #[test]
    fn buildbot_url_targets_this_host() {
        let url = buildbot_url(BUILDBOT, core_for_platform("snes").unwrap());
        if cfg!(target_os = "macos") {
            assert_eq!(url, "https://buildbot.libretro.com/nightly/apple/osx/arm64/latest/snes9x_libretro.dylib.zip");
        } else if cfg!(windows) {
            assert_eq!(
                url,
                "https://buildbot.libretro.com/nightly/windows/x86_64/latest/snes9x_libretro.dll.zip"
            );
        } else {
            assert_eq!(
                url,
                "https://buildbot.libretro.com/nightly/linux/x86_64/latest/snes9x_libretro.so.zip"
            );
        }
    }

    fn zip_of(entries: &[(&str, &[u8])]) -> Vec<u8> {
        use std::io::Write;
        let mut out = std::io::Cursor::new(Vec::new());
        let mut zip = zip::ZipWriter::new(&mut out);
        for (name, body) in entries {
            zip.start_file(*name, zip::write::SimpleFileOptions::default())
                .unwrap();
            zip.write_all(body).unwrap();
        }
        zip.finish().unwrap();
        out.into_inner()
    }

    #[tokio::test]
    async fn system_files_are_installed_once_into_the_system_folder() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/Dolphin.zip"))
            .respond_with(ResponseTemplate::new(200).set_body_bytes(zip_of(&[
                ("dolphin-emu/Sys/codehandler.bin", b"code"),
                ("dolphin-emu/Sys/GC/font.bin", b"font"),
            ])))
            .expect(1)
            .mount(&server)
            .await;
        let dir = tempfile::tempdir().unwrap();
        let core = core_for_platform("ngc").unwrap();
        assert!(!system_files_present(core, dir.path()));
        let http = reqwest::Client::new();
        install_system_files(&http, &server.uri(), core, dir.path())
            .await
            .unwrap();
        assert!(system_files_present(core, dir.path()));
        assert_eq!(
            std::fs::read(dir.path().join("dolphin-emu/Sys/GC/font.bin")).unwrap(),
            b"font"
        );
        install_system_files(&http, &server.uri(), core, dir.path())
            .await
            .unwrap();
        let snes = core_for_platform("snes").unwrap();
        assert!(system_files_present(snes, dir.path()));
    }

    #[tokio::test]
    async fn system_files_that_escape_the_folder_are_refused() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/blueMSX.zip"))
            .respond_with(ResponseTemplate::new(200).set_body_bytes(zip_of(&[
                ("Databases/msxromdb.xml", b"db"),
                ("../evil.txt", b"x"),
            ])))
            .mount(&server)
            .await;
        let root = tempfile::tempdir().unwrap();
        let system = root.path().join("system");
        let core = core_for_platform("msx").unwrap();
        let result =
            install_system_files(&reqwest::Client::new(), &server.uri(), core, &system).await;
        assert!(result.is_err());
        assert!(!root.path().join("evil.txt").exists());
        assert!(!system_files_present(core, &system));
    }

    #[tokio::test]
    async fn install_extracts_and_verifies() {
        let core = core_for_platform("snes").unwrap();
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path(buildbot_url("", core).as_str()))
            .respond_with(
                ResponseTemplate::new(200).set_body_bytes(zip_with(&lib_file(core), b"core-bytes")),
            )
            .mount(&server)
            .await;
        let dir = tempfile::tempdir().unwrap();
        let cores = Cores::new(dir.path().to_path_buf());
        assert!(cores.installed(core).is_none());
        let installed = cores
            .install(&reqwest::Client::new(), &server.uri(), core)
            .await
            .unwrap();
        assert_eq!(std::fs::read(&installed).unwrap(), b"core-bytes");
        assert_eq!(cores.installed(core), Some(installed));
    }

    #[tokio::test]
    async fn tampered_core_is_not_installed() {
        let core = core_for_platform("snes").unwrap();
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .respond_with(
                ResponseTemplate::new(200).set_body_bytes(zip_with(&lib_file(core), b"core-bytes")),
            )
            .mount(&server)
            .await;
        let dir = tempfile::tempdir().unwrap();
        let cores = Cores::new(dir.path().to_path_buf());
        let installed = cores
            .install(&reqwest::Client::new(), &server.uri(), core)
            .await
            .unwrap();
        std::fs::write(&installed, b"evil").unwrap();
        assert!(cores.installed(core).is_none());
    }

    #[tokio::test]
    async fn archive_without_the_library_fails() {
        let core = core_for_platform("snes").unwrap();
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(200).set_body_bytes(zip_with("other.txt", b"x")))
            .mount(&server)
            .await;
        let dir = tempfile::tempdir().unwrap();
        assert!(Cores::new(dir.path().to_path_buf())
            .install(&reqwest::Client::new(), &server.uri(), core)
            .await
            .is_err());
    }

    #[test]
    fn amiga_defaults_boot_a_real_kickstart() {
        let options = default_options("puae");
        assert!(options.contains(&("puae_kickstart".into(), "auto".into())));
        assert!(options.contains(&("puae_model".into(), "auto".into())));
        assert!(options.contains(&("puae_model_fd".into(), "A600".into())));
        assert!(options.contains(&("puae_immediate_blits".into(), "immediate".into())));
        assert!(default_options("mednafen_psx_hw")
            .contains(&("beetle_psx_analog_toggle".into(), "enabled".into())));
        assert!(default_options("snes9x").is_empty());
    }

    #[test]
    fn the_cd32_and_cdtv_run_puae_as_themselves() {
        for (slug, model) in [("amiga-cd32", "CD32"), ("commodore-cdtv", "CDTV")] {
            assert_eq!(core_for_platform(slug).unwrap().id, "puae");
            let options = with_platform_options(slug, default_options("puae"));
            assert!(options.contains(&("puae_model".into(), model.into())));
            assert!(options.contains(&("puae_fastmem_size".into(), "auto".into())));
            assert!(options.contains(&("puae_kickstart".into(), "auto".into())));
        }
        let amiga = with_platform_options("amiga", default_options("puae"));
        assert_eq!(amiga, default_options("puae"));
    }
}
