//! Stage options link to the existing annotated assets and never expose active markup as media.
use super::{child, id, link, name, val, Element, Index, H};
use std::fmt::Write;

pub(super) fn panel(out: &mut String, ads: &[Element<'_>], ix: &Index<'_>) {
    out.push_str("<section id=player class=panel><h2>Creative QA player</h2>");
    let mut options = String::new();
    let mut events = String::new();
    let mut interactive = Vec::new();
    let mut wrappers = Vec::new();
    for (i, &ad) in ads.iter().enumerate() {
        let label = format!(
            "Ad {} · {}",
            i + 1,
            ad.attribute("id").unwrap_or("Untitled")
        );
        write!(options, "<optgroup label='{}'>", H(&label)).unwrap();
        for n in ad
            .descendants()
            .filter(|n| n.is_element() && ix.recognized(*n))
        {
            match name(n) {
                "VASTAdTagURI" | "VASTAdTagURL" => wrappers.push(n),
                "InteractiveCreativeFile"
                    if n.attribute("apiFramework")
                        .is_some_and(|s| s.eq_ignore_ascii_case("SIMID")) =>
                {
                    interactive.push(n)
                }
                "StaticResource" | "MediaFile" | "QrCodeImageUrl" | "Mezzanine" => {
                    if !n
                        .ancestors()
                        .any(|p| p.is_element() && name(p) == "Wrapper")
                    {
                        option(&mut options, n, ix);
                    }
                }
                "Tracking" => {
                    if let Some(group) = n
                        .parent_element()
                        .filter(|p| name(*p) == "TrackingEvents")
                        .and_then(|p| p.parent_element())
                    {
                        if let Some(event) = n.attribute("event") {
                            write!(
                                events,
                                "<a data-owner='{}' data-event='{}' href='#e{}'></a>",
                                id(group),
                                H(event),
                                id(n)
                            )
                            .unwrap();
                        }
                    }
                }
                _ => {}
            }
        }
        options.push_str("</optgroup>");
    }
    out.push_str("<p class=player-intro>Check the media against its VAST declaration. Playback events stay in this report.</p><div id=qa-controls hidden><div class=qa-toolbar><label>Asset<select id=qa-asset>");
    out.push_str(&options);
    out.push_str("</select></label><label>Viewport<select id=qa-ratio><option value='16 / 9'>16:9 widescreen</option><option value='9 / 16'>9:16 portrait</option><option value='1'>1:1 square</option><option value='asset'>Asset aspect ratio</option></select></label><label class=guide-control><input type=checkbox id=qa-guides> 5% inset guide</label></div><div class=qa-grid><div class=qa-monitor><div id=qa-stage><div id=qa-media></div><span id=qa-placeholder>Select Load preview to inspect this asset.</span><i id=qa-guide hidden></i></div></div><div class=qa-sidebar><h3>Asset check</h3><a id=qa-declaration>Inspect XML declaration</a><dl><dt>Declared</dt><dd id=qa-declared></dd><dt>Observed</dt><dd id=qa-observed></dd></dl><p id=qa-result role=status></p><button type=button id=qa-load>Load preview</button></div></div><div class=qa-log><div class=adhead><h3>Playback log</h3><button type=button id=qa-copy>Copy QA notes</button></div><p id=qa-copy-status role=status></p><ol id=qa-events></ol></div></div><p id=qa-empty>Use the annotated XML to inspect the declarations. The player needs JavaScript and a supported image, video, or audio reference.</p><div hidden id=qa-tracking>");
    out.push_str(&events);
    out.push_str("</div>");
    if !interactive.is_empty() {
        out.push_str("<h3>SIMID creative</h3><p>Use the QA player for the media fallback. For a live handshake, open the full tag in SIMID Studio. The studio receives the XML and its URLs.</p><p><a id=qa-studio href='https://iab-tech-lab-vast-tester.vastlint.org/' target='_blank' rel='noopener noreferrer'>Open SIMID Studio</a> <small id=studio-help>Copy the full XML and paste it into the studio.</small></p><div class=chips>");
        for n in interactive {
            link(out, n, &format!("Interactive file · line {}", ix.line(n)));
        }
        out.push_str("<a href='#source'>Full XML</a></div>");
    }
    for n in wrappers {
        out.push_str(
            "<p>For the final creative, open the downstream response in a separate report. ",
        );
        link(out, n, "Inspect wrapper URL");
        out.push_str("</p>");
    }
    out.push_str("</section>");
}

fn option(out: &mut String, n: Element<'_>, ix: &Index<'_>) {
    let owner = n.ancestors().skip(1).filter(|p| p.is_element()).find(|p| {
        matches!(
            name(*p),
            "Linear" | "NonLinear" | "Companion" | "Icon" | "Extension" | "CreativeExtension"
        )
    });
    let label = owner
        .map(|o| o.attribute("type").unwrap_or(name(o)))
        .unwrap_or(name(n));
    let dims = |el: Element<'_>| {
        el.attribute("width")
            .zip(el.attribute("height"))
            .map(|(w, h)| format!("{w} × {h}"))
    };
    let slot = owner
        .filter(|o| matches!(name(*o), "NonLinear" | "Companion" | "Icon"))
        .and_then(dims)
        .unwrap_or_default();
    let duration = owner
        .filter(|o| matches!(name(*o), "NonLinear" | "Linear" | "Extension"))
        .and_then(|o| child(o, "Duration"))
        .map(val)
        .unwrap_or("");
    let group = owner.map(id).unwrap_or(0);
    let shared = owner
        .filter(|o| name(*o) == "NonLinear")
        .and_then(|o| o.parent_element())
        .map(id)
        .unwrap_or(0);
    write!(out,"<option value='e{}' data-owner='{group}' data-shared='{shared}' data-width='{}' data-height='{}' data-slot='{}' data-duration='{}'>{} · {} · line {}</option>",id(n), H(n.attribute("width").unwrap_or("")),H(n.attribute("height").unwrap_or("")),H(&slot),H(duration),H(label),H(n.attribute("type").or(n.attribute("creativeType")).unwrap_or(name(n))),ix.line(n)).unwrap();
}
