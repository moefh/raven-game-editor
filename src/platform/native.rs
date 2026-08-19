use std::io::{Result, Error};
use std::path::PathBuf;
use std::sync::LazyLock;

const APP_ID: &str = "raven-game-editor";

static TIMESTAMP_FORMAT: LazyLock<time::format_description::FormatDescriptionV3> = LazyLock::new(|| {
    time::format_description::parse_borrowed::<3>("[year]-[month]-[day] [hour]:[minute]:[second]").unwrap()
});

fn get_settings_dir() -> Option<PathBuf> {
    use egui::os::OperatingSystem as OS;
    match OS::from_target_os() {
        OS::Nix => std::env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .filter(|p| p.is_absolute())
            .or_else(|| std::env::home_dir().map(|p| p.join(".config")))
            .map(|p| { p.join(APP_ID) }),
        OS::Mac => std::env::home_dir().map(|p| {
            p.join("Library").join("Preferences").join(APP_ID)
        }),
        OS::Windows => std::env::var_os("LOCALAPPDATA")
            .map(PathBuf::from)
            .map(|p| p.join(APP_ID)),
        _ => None,
    }
}

pub fn write_settings_file(filename: impl AsRef<str>, content: &str) -> Result<()> {
    let dir = get_settings_dir().ok_or(Error::other("can't figure out config directory"))?;
    std::fs::create_dir_all(&dir)?;
    let filename = dir.join(filename.as_ref());
    std::fs::write(&filename, content)
}

pub fn read_settings_file(filename: impl AsRef<str>) -> Result<String> {
    let dir = get_settings_dir().ok_or(Error::other("can't figure out config directory"))?;
    std::fs::create_dir_all(&dir)?;
    let filename = dir.join(filename.as_ref());
    std::fs::read_to_string(&filename)
}

pub fn current_time_as_millis() -> u64 {
    use std::time::{SystemTime, Duration, UNIX_EPOCH};
    let start = SystemTime::now();
    let since_the_epoch = start.duration_since(UNIX_EPOCH).unwrap_or(Duration::ZERO);
    since_the_epoch.as_millis() as u64
}

pub fn current_time_as_string() -> String {
    if let Ok(now) = time::OffsetDateTime::now_local() && let Ok(timestamp) = now.format(&TIMESTAMP_FORMAT) {
        timestamp
    } else {
        "<unknown time>".to_owned()
    }
}
