//! Preview only passive media. Source URLs are inert until the reader chooses to load them.
use base64::{engine::general_purpose::STANDARD, Engine};
use std::collections::BTreeMap;
use std::fmt::Write;
use std::fs::File;
use std::io::{self, Read};
use std::path::PathBuf;
use url::Url;

use super::report::H;

pub(super) struct Preview<'a> {
    kind: &'static str,
    mime: String,
    source: &'a str,
    url: Option<Url>,
    inline: bool,
}

impl<'a> Preview<'a> {
    pub fn for_element(
        name: &str,
        mime: Option<&str>,
        framework: Option<&str>,
        source: &'a str,
    ) -> Option<Self> {
        if !matches!(
            name,
            "StaticResource" | "MediaFile" | "Mezzanine" | "QrCodeImageUrl"
        ) || framework.is_some_and(|s| !s.trim().is_empty())
            || source.is_empty()
        {
            return None;
        }
        let mime = mime
            .unwrap_or("")
            .split(';')
            .next()?
            .trim()
            .to_ascii_lowercase();
        let kind = match mime.split_once('/').map(|(family, _)| family) {
            Some("image") => "image",
            Some("video") => "video",
            Some("audio") => "audio",
            _ if name == "QrCodeImageUrl" => "image",
            _ => return None,
        };
        let url = Url::parse(source).ok().filter(|u| {
            u.scheme() == "https"
                && u.host_str().is_some()
                && u.username().is_empty()
                && u.password().is_none()
        });
        let inline = source
            .split_once(',')
            .is_some_and(|(head, _)| head.eq_ignore_ascii_case(&format!("data:{mime};base64")));
        Some(Self {
            kind,
            mime,
            source,
            url,
            inline,
        })
    }

    pub fn card(&self, out: &mut String, embeds: &mut Embedded) {
        let embedded = embeds.files.get_mut(self.source).map(|file| {
            file.used = true;
            if self.kind == "image" && file.image_mime.is_none() {
                file.image_mime = Some(if self.mime.is_empty() {
                    "application/octet-stream".into()
                } else {
                    self.mime.clone()
                });
            }
            file.id
        });
        let filename = self
            .url
            .as_ref()
            .and_then(|u| u.path_segments()?.next_back())
            .filter(|s| !s.is_empty())
            .unwrap_or(self.kind);
        write!(
            out,
            "<span class=asset data-kind='{}' data-mime='{}'",
            self.kind,
            H(&self.mime)
        )
        .unwrap();
        if let Some(id) = embedded {
            write!(out, " data-file='asset-{id}'").unwrap();
        } else if self.inline {
            out.push_str(" data-inline");
        } else if self.url.is_some() {
            out.push_str(" data-remote");
        }
        out.push_str("><span class=asset-thumb aria-hidden=true>");
        match (self.kind, embedded, self.inline) {
            ("image", Some(id), _) => write!(
                out,
                "<svg viewBox='0 0 160 90'><use href='#asset-{id}'/></svg>"
            )
            .unwrap(),
            ("image", None, true) => write!(out, "<img alt='' src='{}'>", H(self.source)).unwrap(),
            _ => out.push_str(&self.kind.to_uppercase()),
        }
        write!(
            out,
            "</span><span class=asset-name><strong>{}</strong><small>",
            H(filename)
        )
        .unwrap();
        if embedded.is_some() || self.inline {
            out.push_str("Embedded in report");
        } else if let Some(host) = self.url.as_ref().and_then(Url::host_str) {
            write!(out, "{}", H(host)).unwrap();
        } else {
            out.push_str("Resource reference");
        }
        out.push_str("</small></span>");
        if embedded.is_some() || self.inline || self.url.is_some() {
            write!(
                out,
                "<button class=asset-action type=button hidden>{} {}</button>",
                if embedded.is_some() || self.inline {
                    "View"
                } else {
                    "Load"
                },
                self.kind
            )
            .unwrap();
        }
        out.push_str("</span>");
    }
}

struct LocalFile {
    path: PathBuf,
    id: usize,
    used: bool,
    image_mime: Option<String>,
}

pub(super) struct Embedded {
    files: BTreeMap<String, LocalFile>,
}
impl Embedded {
    pub fn new(args: &[String]) -> io::Result<Self> {
        let mut files = BTreeMap::new();
        for (id, pair) in args.chunks_exact(2).enumerate() {
            if files
                .insert(
                    pair[0].clone(),
                    LocalFile {
                        id,
                        path: PathBuf::from(&pair[1]),
                        used: false,
                        image_mime: None,
                    },
                )
                .is_some()
            {
                return Err(io::Error::other("each --embed-asset URL must be unique"));
            }
        }
        Ok(Self { files })
    }

    pub fn write_payloads(&self, out: &mut String) -> io::Result<()> {
        // The report is built in memory, so cap the aggregate before encoding.
        let mut remaining = 64 * 1024 * 1024u64;
        for (url, file) in &self.files {
            if !file.used {
                return Err(io::Error::other(format!(
                    "no previewable media declaration matches {url}"
                )));
            }
            let mut bytes = Vec::new();
            File::open(&file.path)
                .and_then(|f| f.take(remaining + 1).read_to_end(&mut bytes))
                .map_err(|e| io::Error::other(format!("{}: {e}", file.path.display())))?;
            remaining = remaining.checked_sub(bytes.len() as u64).ok_or_else(|| {
                io::Error::other("embedded media exceeds the 64 MiB combined limit")
            })?;
            if let Some(mime) = &file.image_mime {
                // One image definition feeds every declaration, even with scripting disabled.
                write!(out,"<svg aria-hidden='true' width='0' height='0' class='asset-def'><defs><image id='asset-{}' width='160' height='90' href='data:{};base64,", file.id,H(mime)).unwrap();
                STANDARD.encode_string(&bytes, out);
                out.push_str("'/></defs></svg>");
            } else {
                write!(out, "<template id='asset-{}'>", file.id).unwrap();
                STANDARD.encode_string(&bytes, out);
                out.push_str("</template>");
            }
        }
        Ok(())
    }
}
