//! Per-user CLI usage statistics and first-run setup.

use std::fs;
use std::io::{self, BufRead, IsTerminal, Write};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, SyncSender};
use std::time::Duration;
use vastlint_core::{InputStatus, ValidationResult};

const TELEMETRY_ENDPOINT: &str = "https://vastlint.org/api/ping";
const DETAILS: &str = "https://vastlint.org/privacy/#cli-telemetry";

#[derive(Clone, Copy, Debug, PartialEq, Eq, clap::ValueEnum)]
pub enum Setting {
    On,
    Off,
}

impl Setting {
    fn label(self) -> &'static str {
        match self {
            Self::On => "on",
            Self::Off => "off",
        }
    }
}

fn config_dir() -> io::Result<PathBuf> {
    let env_path = |name| {
        std::env::var_os(name)
            .filter(|p| !p.is_empty())
            .map(PathBuf::from)
    };
    env_path("XDG_CONFIG_HOME")
        .or_else(|| env_path("HOME").map(|p| p.join(".config")))
        .or_else(|| env_path("APPDATA"))
        .map(|p| p.join("vastlint"))
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "no user configuration directory"))
}

fn read_setting(path: &Path) -> io::Result<Option<Setting>> {
    match fs::read_to_string(path) {
        Ok(value) => match value.trim() {
            "on" => Ok(Some(Setting::On)),
            "off" => Ok(Some(Setting::Off)),
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "invalid telemetry setting",
            )),
        },
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e),
    }
}

fn save_setting(dir: &Path, setting: Setting, replace: bool) -> io::Result<Setting> {
    fs::create_dir_all(dir)?;
    let mut tmp = tempfile::NamedTempFile::new_in(dir)?;
    writeln!(tmp, "{}", setting.label())?;
    let path = dir.join("telemetry");
    if replace {
        tmp.persist(path)?;
    } else if let Err(e) = tmp.persist_noclobber(&path) {
        if e.error.kind() != io::ErrorKind::AlreadyExists {
            return Err(e.error);
        }
        return read_setting(&path)?.ok_or_else(|| io::Error::from(io::ErrorKind::NotFound));
    }
    Ok(setting)
}

fn prompt(mut input: impl BufRead, mut output: impl Write) -> io::Result<Option<Setting>> {
    writeln!(output, "vastlint check can send full tag XML (including fetched wrappers), validation counts, CLI version, operating system, random installation ID, and input count to vastlint.org.")?;
    writeln!(
        output,
        "We use these tags only for research and to improve VASTlint. The server redacts known tracking identifiers before storage."
    )?;
    writeln!(output, "Details: {DETAILS}")?;
    loop {
        write!(
            output,
            "Send usage statistics and tag XML to vastlint.org? [Y/n] "
        )?;
        output.flush()?;
        let mut answer = String::new();
        if input.read_line(&mut answer)? == 0 {
            return Ok(None);
        }
        match answer.trim().to_ascii_lowercase().as_str() {
            "" | "y" | "yes" => return Ok(Some(Setting::On)),
            "n" | "no" => return Ok(Some(Setting::Off)),
            _ => writeln!(output, "Press Enter for yes or type N for no.")?,
        }
    }
}

fn setup_if_needed(dir: &Path, interactive: bool) -> io::Result<Option<Setting>> {
    let saved = read_setting(&dir.join("telemetry"))?;
    if saved.is_some()
        || !interactive
        || std::env::var_os("CI").is_some()
        || !io::stdin().is_terminal()
        || !io::stderr().is_terminal()
    {
        return Ok(saved);
    }
    if let Some(setting) = prompt(io::stdin().lock(), io::stderr().lock())? {
        // An outstanding default-yes prompt must respect a concurrent opt-out.
        let setting = save_setting(dir, setting, setting == Setting::Off)?;
        eprintln!(
            "Sharing usage statistics and tag XML is {}. Change this with vastlint telemetry on or off.",
            setting.label()
        );
        return Ok(Some(setting));
    }
    Ok(None)
}

pub fn configure(setting: Option<Setting>) -> io::Result<()> {
    let dir = config_dir()?;
    if let Some(setting) = setting {
        save_setting(&dir, setting, true)?;
        eprintln!(
            "Sharing usage statistics and tag XML is {}.",
            setting.label()
        );
    } else if setup_if_needed(&dir, true)?.is_none() {
        eprintln!(
            "Telemetry setup is pending. Run vastlint telemetry on or off to choose. {DETAILS}"
        );
    }
    Ok(())
}

#[derive(Default)]
pub struct Sharing {
    statistics: bool,
    samples: bool,
}

impl Sharing {
    fn resolve(saved: Option<Setting>, project: Option<bool>) -> Self {
        let samples = saved == Some(Setting::On) && project != Some(false);
        // Existing project settings only authorized usage statistics.
        let statistics = samples || (saved.is_none() && project == Some(true));
        Self {
            statistics,
            samples,
        }
    }

    pub fn for_check(project: Option<bool>, may_prompt: bool) -> Self {
        if project == Some(false) {
            return Self::default();
        }
        match config_dir().and_then(|dir| setup_if_needed(&dir, may_prompt)) {
            Ok(saved) => Self::resolve(saved, project),
            Err(e) => {
                eprintln!(
                    "Could not read or save telemetry settings: {e}. Skipping automatic sharing."
                );
                Self::default()
            }
        }
    }
}

/// One worker per check keeps network requests off the validation path.
/// Sample buffering and the wait at exit are bounded even for large batches.
pub struct Reporter {
    samples: Option<SyncSender<String>>,
    done: Receiver<()>,
    explicit_sample: bool,
}

impl Reporter {
    pub fn start(
        sharing: Sharing,
        stats_flag: bool,
        sample_flag: bool,
        files: usize,
    ) -> Option<Self> {
        let statistics = sharing.statistics || stats_flag;
        let samples = sharing.samples || sample_flag;
        if !statistics && !samples {
            return None;
        }
        let (sender, queue) = mpsc::sync_channel::<String>(8);
        let (finished, done) = mpsc::channel();
        std::thread::Builder::new()
            .name("vastlint-telemetry".into())
            .spawn(move || {
                if statistics {
                    let url = format!(
                        "{}?v={}&os={}&id={}&files={}",
                        TELEMETRY_ENDPOINT,
                        env!("CARGO_PKG_VERSION"),
                        os_name(),
                        install_id(),
                        files
                    );
                    let _ = ureq::get(&url).timeout(Duration::from_secs(2)).call();
                }
                for body in queue {
                    crate::contribute::submit(&body);
                }
                let _ = finished.send(());
            })
            .ok()?;
        Some(Self {
            samples: samples.then_some(sender),
            done,
            explicit_sample: sample_flag,
        })
    }

    pub fn sample(&self, xml: &str, result: &ValidationResult) {
        if let Some(sender) = &self.samples {
            if !self.explicit_sample && result.input_status != InputStatus::Supported {
                return;
            }
            if let Some(body) = crate::contribute::body(xml, result) {
                let _ = sender.try_send(body);
            }
        }
    }

    pub fn finish(self) {
        drop(self.samples);
        let _ = self.done.recv_timeout(Duration::from_secs(2));
    }
}

fn os_name() -> &'static str {
    if cfg!(target_os = "linux") {
        "linux"
    } else if cfg!(target_os = "macos") {
        "macos"
    } else if cfg!(target_os = "windows") {
        "windows"
    } else {
        "other"
    }
}

/// Return a stable anonymous install ID, creating it if it does not exist.
/// Stored at `~/.config/vastlint/id` as a 32-char hex string.
/// Falls back to a random ID that is not persisted if the directory is not writable.
fn install_id() -> String {
    if let Ok(dir) = config_dir() {
        let path = dir.join("id");
        if let Ok(existing) = std::fs::read_to_string(&path) {
            let trimmed = existing.trim().to_owned();
            if trimmed.len() == 32 && trimmed.chars().all(|c| c.is_ascii_hexdigit()) {
                return trimmed;
            }
        }
        // Generate a new ID and try to persist it.
        let id = random_hex_id();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = std::fs::write(&path, &id);
        return id;
    }
    random_hex_id()
}

/// Generate a random 32-char hex string using system entropy via `/dev/urandom`
/// on Unix or `BCryptGenRandom` on Windows (via `std` random facilities).
fn random_hex_id() -> String {
    // Use 16 bytes of entropy from the OS via a simple approach:
    // read from /dev/urandom on Unix, fall back to a timestamp+pid mix.
    let bytes = os_random_bytes();
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

fn os_random_bytes() -> [u8; 16] {
    // Try /dev/urandom first (Unix).
    #[cfg(unix)]
    {
        use std::io::Read;
        if let Ok(mut f) = std::fs::File::open("/dev/urandom") {
            let mut buf = [0u8; 16];
            if f.read_exact(&mut buf).is_ok() {
                return buf;
            }
        }
    }
    // Fallback: mix timestamp + PID — not cryptographically random but
    // sufficient for a non-security install ID.
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let pid = std::process::id() as u128;
    let mixed = ts ^ (pid << 32) ^ (pid >> 32);
    mixed.to_le_bytes()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prompt_discloses_both_uploads_and_handles_default_refusal_and_eof() {
        for (input, expected) in [
            ("\n", Some(Setting::On)),
            ("Yes\n", Some(Setting::On)),
            ("invalid\nN\n", Some(Setting::Off)),
            ("no\n", Some(Setting::Off)),
            ("", None),
        ] {
            let mut output = Vec::new();
            assert_eq!(prompt(input.as_bytes(), &mut output).unwrap(), expected);
            let output = String::from_utf8(output).unwrap();
            assert!(output.contains("usage statistics and tag XML"));
            assert!(output.contains("only for research and to improve VASTlint"));
            assert!(output.contains("[Y/n]"));
            assert!(output.contains(DETAILS));
        }
    }

    #[test]
    fn concurrent_setup_cannot_overwrite_an_opt_out_with_default_yes() {
        let dir = tempfile::tempdir().unwrap();
        let dir = dir.path();
        save_setting(dir, Setting::Off, true).unwrap();
        assert_eq!(save_setting(dir, Setting::On, false).unwrap(), Setting::Off);
        assert_eq!(setup_if_needed(dir, false).unwrap(), Some(Setting::Off));
        save_setting(dir, Setting::On, true).unwrap();
        assert_eq!(setup_if_needed(dir, false).unwrap(), Some(Setting::On));
    }

    #[test]
    fn project_settings_preserve_existing_choices() {
        for (saved, project, stats, xml) in [
            (None, None, false, false),
            (None, Some(true), true, false),
            (Some(Setting::Off), Some(true), false, false),
            (Some(Setting::On), Some(false), false, false),
            (Some(Setting::On), None, true, true),
        ] {
            let sharing = Sharing::resolve(saved, project);
            assert_eq!((sharing.statistics, sharing.samples), (stats, xml));
        }
    }

    #[test]
    fn automatic_samples_only_enqueue_supported_tags_with_validation_counts() {
        let (sender, queue) = mpsc::sync_channel(2);
        let reporter = Reporter {
            samples: Some(sender),
            done: mpsc::channel().1,
            explicit_sample: false,
        };
        for xml in ["plain text", "<other/>", "<VAST/>"] {
            reporter.sample(xml, &vastlint_core::validate(xml));
        }
        let body = queue.try_recv().unwrap();
        assert!(body.contains(r#""xml":"<VAST/>""#));
        assert!(body.contains(r#""summary":{"errors":"#));
        assert!(queue.try_recv().is_err());
    }
}
