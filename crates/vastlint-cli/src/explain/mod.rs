mod guide;
mod report;

use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;
use std::process::ExitCode;

use super::EXIT_USAGE_ERROR;

// Matches the engine's supported document size. Bound stdin before allocating a report.
const MAX_BYTES: u64 = 4 * 1024 * 1024;

pub(super) fn run(file: &str, output: Option<&Path>) -> ExitCode {
    match generate(file, output) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("vastlint explain: {e}");
            ExitCode::from(EXIT_USAGE_ERROR)
        }
    }
}

fn generate(file: &str, output: Option<&Path>) -> io::Result<()> {
    let reader: Box<dyn Read> = if file == "-" {
        Box::new(io::stdin().lock())
    } else {
        Box::new(fs::File::open(file)?)
    };
    let mut bytes = Vec::new();
    reader.take(MAX_BYTES + 1).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > MAX_BYTES {
        return Err(io::Error::other(
            "input exceeds the 4 MiB VAST document limit",
        ));
    }
    let input = std::str::from_utf8(&bytes).map_err(io::Error::other)?;
    let label = if file == "-" {
        "Standard input"
    } else {
        Path::new(file)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or(file)
    };
    let html = report::render(label, input);
    match output {
        None => io::stdout().lock().write_all(html.as_bytes()),
        Some(path) => {
            if file != "-" && path.canonicalize().ok() == Some(Path::new(file).canonicalize()?) {
                return Err(io::Error::other("output points to the input XML file"));
            }
            let dir = path
                .parent()
                .filter(|p| !p.as_os_str().is_empty())
                .unwrap_or(Path::new("."));
            let mut temp = tempfile::NamedTempFile::new_in(dir)?;
            temp.write_all(html.as_bytes())?;
            temp.persist(path)?;
            Ok(())
        }
    }
}
