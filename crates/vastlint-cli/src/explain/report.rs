use std::collections::{HashMap, HashSet};
use std::fmt::{self, Write};

use roxmltree::{Document, Node, ParsingOptions};
use vastlint_core::ValidationResult;

use super::guide;

#[path = "reader.rs"]
mod reader;

type Element<'a> = Node<'a, 'a>;

// XML is untrusted, including element values, namespace names and filenames.
struct H<'a>(&'a str);
impl fmt::Display for H<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for part in self.0.split_inclusive(['&', '<', '>', '"', '\'']) {
            let replacement = match part.as_bytes().last() {
                Some(b'&') => "&amp;",
                Some(b'<') => "&lt;",
                Some(b'>') => "&gt;",
                Some(b'"') => "&quot;",
                Some(b'\'') => "&#39;",
                _ => {
                    f.write_str(part)?;
                    continue;
                }
            };
            f.write_str(&part[..part.len() - 1])?;
            f.write_str(replacement)?;
        }
        Ok(())
    }
}

fn id(n: Element<'_>) -> usize {
    n.id().get_usize()
}
fn name(n: Element<'_>) -> &str {
    n.tag_name().name()
}
fn val(n: Element<'_>) -> &str {
    n.text().unwrap_or("").trim()
}
fn child<'a>(n: Element<'a>, key: &str) -> Option<Element<'a>> {
    n.children().find(|c| {
        c.is_element() && name(*c) == key && c.tag_name().namespace() == n.tag_name().namespace()
    })
}

fn qualified<'a>(n: Element<'a>, xml: &'a str) -> &'a str {
    xml[n.range().start + 1..]
        .split([' ', '\t', '\n', '\r', '/', '>'])
        .next()
        .unwrap_or("")
}

struct Index<'a> {
    nodes: Vec<Element<'a>>,
    known: HashSet<usize>,
    lines: Vec<usize>,
}

impl<'a> Index<'a> {
    fn new(doc: &'a Document<'a>, xml: &str) -> Self {
        let root = doc.root_element();
        let nodes: Vec<_> = root.descendants().filter(Node::is_element).collect();
        let mut known = HashSet::new();
        for &n in &nodes {
            if n == root {
                if name(n) == "VAST"
                    && matches!(
                        n.tag_name().namespace(),
                        None | Some("http://www.iab.com/VAST")
                            | Some("http://www.iab.net/videosuite/vast")
                    )
                {
                    known.insert(id(n));
                }
                continue;
            }
            let parent = n.parent_element().unwrap();
            if !known.contains(&id(parent))
                || n.tag_name().namespace() != root.tag_name().namespace()
            {
                continue;
            }
            let k = name(n);
            if guide::element(k).is_none() {
                continue;
            }
            let recognized = match name(parent) {
                "Extension" | "CreativeExtension" => {
                    match (parent.attribute("type"), parent.attribute("ext")) {
                        (Some("ctv_ad_portfolio"), _) => matches!(
                            k,
                            "plcmt"
                                | "pos"
                                | "attr"
                                | "playbackmethod"
                                | "CreativeId"
                                | "Duration"
                                | "MediaFiles"
                                | "TrackingEvents"
                                | "CreativeExtensions"
                                | "Icons"
                                | "NonLinearClickTracking"
                                | "NonLinearCustomClick"
                        ),
                        (Some("AdVerifications"), _) => {
                            k == "AdVerifications" || k == "Verification"
                        }
                        (Some("UniversalAdId"), _) => k == "UniversalAdId",
                        (Some("tl_qrcode"), _) => k.starts_with("QrCode"),
                        (Some(signal), Some("adcom")) => {
                            signal == k && matches!(k, "plcmt" | "pos" | "playbackmethod" | "attr")
                        }
                        _ => false,
                    }
                }
                "AdParameters" | "VerificationParameters" | "HTMLResource" => false,
                _ => !matches!(k, "plcmt" | "pos" | "attr" | "playbackmethod"),
            };
            if recognized {
                known.insert(id(n));
            }
        }
        let mut lines = vec![0];
        lines.extend(xml.match_indices('\n').map(|(i, _)| i + 1));
        Self {
            nodes,
            known,
            lines,
        }
    }
    fn line(&self, n: Element<'_>) -> usize {
        self.lines
            .partition_point(|&offset| offset <= n.range().start)
    }
    fn recognized(&self, n: Element<'_>) -> bool {
        self.known.contains(&id(n))
    }
}

pub(super) fn render(label: &str, xml: &str) -> String {
    let result = vastlint_core::validate(xml);
    let mut out = String::from("<!doctype html><html lang=en><head><meta charset=utf-8><meta name=viewport content='width=device-width,initial-scale=1'><meta http-equiv='Content-Security-Policy' content=\"default-src 'none'; style-src 'unsafe-inline'; script-src 'unsafe-inline'; base-uri 'none'; form-action 'none'\">");
    write!(out,"<title>{} | VASTlint</title><style>{}</style></head><body><header class=topbar><div class=topinner><a class=brand href='#elements'>VAST<span>lint</span></a><span class=slash>/</span><span>Explain</span><span class=slash>/</span><code class=filename>{}</code></div></header><main>",H(label),include_str!("report.css"),H(label)).unwrap();
    let parsed = Document::parse_with_options(
        xml,
        ParsingOptions {
            nodes_limit: 420_000,
            ..Default::default()
        },
    );
    let doc = match parsed {
        Ok(ref d) => Some(d),
        Err(ref e) => {
            write!(
                out,
                "<p class=notice>Element map unavailable. {}</p>",
                H(&e.to_string())
            )
            .unwrap();
            None
        }
    };
    let index = doc.map(|d| Index::new(d, xml));
    let ads: Vec<_> = doc
        .zip(index.as_ref())
        .map(|(d, ix)| {
            d.root_element()
                .children()
                .filter(|n| n.is_element() && name(*n) == "Ad" && ix.recognized(*n))
                .collect()
        })
        .unwrap_or_default();
    reader::overview(
        &mut out,
        &ads,
        index.as_ref(),
        doc.map(|d| d.root_element()),
    );
    let first = if index.is_some() {
        "elements"
    } else {
        "findings"
    };
    write!(
        out,
        "<nav class=views aria-label='Report views' data-first='{first}'>"
    )
    .unwrap();
    if let Some(ix) = &index {
        write!(out,"<a href='#elements'>Annotated XML <small>{}</small></a><a href='#delivery'>Ad behavior <small>{}</small></a>",ix.nodes.len(),ads.len()).unwrap();
    }
    write!(
        out,
        "<a href='#findings'>Findings <small>{}</small></a><a href='#source'>Raw XML</a></nav>",
        result.issues.len()
    )
    .unwrap();
    if let Some(ix) = &index {
        out.push_str("<section id=elements class=panel aria-label='Annotated XML'><div class=tools hidden><label class=search><span aria-hidden=true>⌕</span><input id=query aria-label='Search elements, values and behavior' type=search placeholder='Find an element, value or behavior'></label><select id=scope aria-label='Filter by purpose'><option value=''>All elements</option><option>Delivery</option><option>Creative</option><option>Placement</option><option>Measurement</option><option>Interaction</option><option>Verification</option><option>Extension</option></select><output id=matches aria-live=polite></output></div><div class=sheet><div class=colheads><span>XML source</span><span>Player behavior</span></div><div id=entries>");
        for &n in &ix.nodes {
            element(&mut out, n, ix, xml);
        }
        out.push_str("</div><p id=empty hidden>No matching elements. <button type=button id=clear>Clear filters</button></p></div></section><section id=delivery class=panel><h2>Ad behavior</h2>");
        if ads.is_empty() {
            out.push_str("<p>No top-level VAST ad is declared.</p>");
        }
        for (i, &ad) in ads.iter().enumerate() {
            ad_card(&mut out, ad, i + 1, ix);
        }
        out.push_str("</section>");
    }
    findings(&mut out, &result, index.as_ref());
    out.push_str("<section id=source class=panel><h2>Raw XML</h2><pre tabindex=0><code>");
    let mut offset = 0;
    if let Some(ix) = &index {
        for &n in &ix.nodes {
            let start = n.range().start;
            let end = start + 1 + qualified(n, xml).len();
            write!(
                out,
                "{}<span id=s{number}><a href='#e{number}'>{}</a></span>",
                H(&xml[offset..start]),
                H(&xml[start..end]),
                number = id(n)
            )
            .unwrap();
            offset = end;
        }
    }
    write!(out,"{}</code></pre></section></main><footer>Generated locally with VASTlint {}. The report includes the supplied XML.</footer><script>{}</script></body></html>",H(&xml[offset..]),env!("CARGO_PKG_VERSION"),include_str!("report.js")).unwrap();
    out
}

fn quantity(n: usize, noun: &str) -> String {
    format!("{n} {noun}{}", if n == 1 { "" } else { "s" })
}
fn link(out: &mut String, n: Element<'_>, label: &str) {
    write!(out, "<a href='#e{}'>{}</a>", id(n), H(label)).unwrap();
}

const ASSETS: &[&str] = &[
    "MediaFile",
    "StaticResource",
    "IFrameResource",
    "HTMLResource",
    "InteractiveCreativeFile",
    "Mezzanine",
    "ClosedCaptionFile",
    "JavaScriptResource",
    "ExecutableResource",
    "QrCodeImageUrl",
];
fn ad_card(out: &mut String, ad: Element<'_>, number: usize, ix: &Index<'_>) {
    let title = child(ad, "InLine")
        .or_else(|| child(ad, "Wrapper"))
        .and_then(|b| child(b, "AdTitle"))
        .map(val)
        .filter(|s| !s.is_empty())
        .or(ad.attribute("id"))
        .unwrap_or("Untitled ad");
    write!(
        out,
        "<article class=ad><div class=adhead><div><h3>{}</h3><small>Ad {number}</small></div>",
        H(title)
    )
    .unwrap();
    link(out, ad, "Inspect ad");
    out.push_str("</div>");
    if let Some(seq) = ad.attribute("sequence") {
        write!(out, "<p>Pod sequence <b>{}</b></p>", H(seq)).unwrap();
    }
    for body in ad
        .children()
        .filter(|n| n.is_element() && ix.recognized(*n) && matches!(name(*n), "Wrapper" | "InLine"))
    {
        let wrapper = name(body) == "Wrapper";
        out.push_str(if wrapper {"<p class=flow><b>Wrapper</b> <span>Delegates delivery to the next VAST response.</span></p>"} else {"<p class=flow><b>InLine</b> <span>Supplies creative instructions in this response.</span></p>"});
        if let Some(next) = child(body, "VASTAdTagURI").or_else(|| child(body, "VASTAdTagURL")) {
            out.push_str("<p>Next tag · ");
            link(out, next, val(next));
            out.push_str("</p>");
        }
        let members: Vec<_> = body
            .descendants()
            .filter(|n| n.is_element() && ix.recognized(*n))
            .collect();
        describe_delivery(out, &members, wrapper);
        placement(out, &members);
        creatives(out, body, ix);
        resources(out, &members);
        destinations(out, &members);
        measurement(out, &members);
    }
    out.push_str("</article>");
}

fn describe_delivery(out: &mut String, members: &[Element<'_>], wrapper: bool) {
    let mut formats = Vec::new();
    for &(key, label) in &[
        ("Linear", "linear playback"),
        ("NonLinear", "a non-linear presentation"),
        ("Companion", "a companion surface"),
    ] {
        if members.iter().any(|n| name(*n) == key) {
            formats.push(label);
        }
    }
    if !formats.is_empty() {
        let verb = if wrapper {
            "provides wrapper instructions for"
        } else {
            "declares"
        };
        write!(
            out,
            "<p class=narrative>This ad {verb} {}.",
            formats.join(" and ")
        )
        .unwrap();
        let clickable = members
            .iter()
            .filter(|n| {
                matches!(
                    name(**n),
                    "ClickThrough"
                        | "NonLinearClickThrough"
                        | "CompanionClickThrough"
                        | "IconClickThrough"
                )
            })
            .count();
        if clickable > 0 {
            write!(
                out,
                " It specifies {}.",
                quantity(clickable, "click destination")
            )
            .unwrap();
        }
        out.push_str("</p>");
    }
    for files in members.iter().copied().filter(|n| name(*n) == "MediaFiles") {
        if let Some(interactive) = child(files, "InteractiveCreativeFile") {
            out.push_str("<p class=narrative>A compatible player can load the ");
            link(
                out,
                interactive,
                interactive
                    .attribute("apiFramework")
                    .unwrap_or("interactive"),
            );
            out.push_str(" creative");
            if let Some(media) = child(files, "MediaFile") {
                out.push_str(" and has a ");
                link(out, media, media.attribute("type").unwrap_or("media"));
                out.push_str(" rendition available for fallback");
            }
            out.push_str(".</p>");
        }
    }
    let creatives: Vec<_> = members
        .iter()
        .copied()
        .filter(|n| name(*n) == "Creative")
        .collect();
    let mut by_id = HashMap::new();
    for &c in &creatives {
        if let Some(key) = c.attribute("id") {
            // Duplicate IDs make the binding ambiguous.
            by_id
                .entry(key)
                .and_modify(|target| *target = None)
                .or_insert(Some(c));
        }
    }
    for ext in members
        .iter()
        .copied()
        .filter(|n| name(*n) == "Extension" && n.attribute("type") == Some("ctv_ad_portfolio"))
    {
        let target = match child(ext, "CreativeId").map(val) {
            Some(key) => by_id.get(key).copied().flatten(),
            None if creatives.len() == 1 => Some(creatives[0]),
            _ => None,
        };
        if let Some(image) = target
            .and_then(|c| child(c, "NonLinearAds"))
            .and_then(|g| child(g, "NonLinear"))
            .and_then(|n| child(n, "StaticResource"))
        {
            out.push_str("<p class=narrative>A player that ignores the ");
            link(out, ext, "CTV portfolio extension");
            out.push_str(" can still use the associated creative’s native ");
            link(out, image, "static resource");
            out.push_str(".</p>");
        }
    }
}

fn placement(out: &mut String, members: &[Element<'_>]) {
    let signals: Vec<_> = members
        .iter()
        .copied()
        .filter(|n| matches!(name(*n), "plcmt" | "pos" | "attr" | "playbackmethod"))
        .collect();
    let nonlinear = members.iter().any(|n| name(*n) == "NonLinear");
    if nonlinear || !signals.is_empty() {
        out.push_str("<div class=placement><h4>Placement declarations</h4><div class=chips>");
        for &n in &signals {
            let label = guide::signal(name(n), val(n)).unwrap_or("Unmapped value");
            link(out, n, &format!("{} {} · {label}", name(n), val(n)));
        }
        for (key, missing) in [
            ("plcmt", "Format unspecified"),
            ("pos", "Position unspecified"),
        ] {
            if !signals.iter().any(|n| name(*n) == key) {
                write!(out, "<span>{missing}</span>").unwrap();
            }
        }
        out.push_str("</div>");
        let positions: Vec<_> = signals.iter().filter(|n| name(**n) == "pos").collect();
        if let [position] = positions.as_slice() {
            let p = val(**position);
            if matches!(
                p,
                "4" | "5"
                    | "6"
                    | "7"
                    | "8"
                    | "9"
                    | "10"
                    | "11"
                    | "12"
                    | "13"
                    | "14"
                    | "15"
                    | "16"
                    | "17"
            ) {
                write!(out,"<div class=visuals><figure class=geometry><div aria-hidden=true class='screen p{}'><i></i></div><figcaption>",H(p)).unwrap();
                link(out, **position, guide::signal("pos", p).unwrap());
                out.push_str(
                    "<small>Declared position · schematic</small></figcaption></figure></div>",
                );
            }
        }
        out.push_str("</div>");
    }
}
fn creatives(out: &mut String, body: Element<'_>, ix: &Index<'_>) {
    if let Some(creatives) = child(body, "Creatives") {
        for c in creatives
            .children()
            .filter(|n| n.is_element() && name(*n) == "Creative" && ix.recognized(*n))
        {
            out.push_str("<div class=creative><b>Creative");
            if let Some(cid) = c.attribute("id") {
                write!(out, " · {}", H(cid)).unwrap();
            }
            out.push_str("</b><div class=chips>");
            for shape in c.children().filter(|x| {
                x.is_element()
                    && ix.recognized(*x)
                    && matches!(name(*x), "Linear" | "NonLinearAds" | "CompanionAds")
            }) {
                link(
                    out,
                    shape,
                    match name(shape) {
                        "NonLinearAds" => "Non-linear",
                        "CompanionAds" => "Companion",
                        _ => "Linear",
                    },
                );
                if let Some(duration) = child(shape, "Duration") {
                    link(out, duration, &format!("Duration {}", val(duration)));
                }
                if let Some(skip) = shape.attribute("skipoffset") {
                    link(out, shape, &format!("Skip at {skip}"));
                }
            }
            out.push_str("</div></div>");
            if let Some(linear) = child(c, "Linear") {
                let events = child(linear, "TrackingEvents");
                let milestones: Vec<_> = [
                    ("start", "0%"),
                    ("firstQuartile", "25%"),
                    ("midpoint", "50%"),
                    ("thirdQuartile", "75%"),
                    ("complete", "100%"),
                ]
                .into_iter()
                .filter_map(|(key, tick)| {
                    events?
                        .children()
                        .find(|n| {
                            n.is_element()
                                && ix.recognized(*n)
                                && name(*n) == "Tracking"
                                && n.attribute("event") == Some(key)
                        })
                        .map(|n| (n, tick))
                })
                .collect();
                if !milestones.is_empty() {
                    out.push_str("<h4>Declared playback milestones</h4><ol class=timeline>");
                    for (n, tick) in milestones {
                        write!(out, "<li class=t{}>", tick.trim_end_matches('%')).unwrap();
                        link(out, n, tick);
                        out.push_str("</li>");
                    }
                    out.push_str("</ol>");
                }
            }
        }
    }
}
fn resources(out: &mut String, members: &[Element<'_>]) {
    let assets: Vec<_> = members
        .iter()
        .copied()
        .filter(|n| ASSETS.contains(&name(*n)))
        .collect();
    if !assets.is_empty() {
        out.push_str("<h4>Declared resources</h4><ul class=resources>");
        for n in assets {
            out.push_str("<li><div>");
            link(out, n, name(n));
            let context = n.ancestors().skip(1).find(|p| {
                matches!(
                    name(*p),
                    "Icon"
                        | "Companion"
                        | "NonLinear"
                        | "Linear"
                        | "Verification"
                        | "Extension"
                        | "CreativeExtension"
                )
            });
            if let Some(owner) = context {
                write!(
                    out,
                    " <small>{}</small>",
                    H(owner.attribute("type").unwrap_or(name(owner)))
                )
                .unwrap();
            }
            for a in ["type", "creativeType", "apiFramework", "codec"] {
                if let Some(v) = n.attribute(a) {
                    write!(out, " <small>{}</small>", H(v)).unwrap();
                }
            }
            if let (Some(w), Some(h)) = (n.attribute("width"), n.attribute("height")) {
                write!(out, " <small>{} × {}</small>", H(w), H(h)).unwrap();
            }
            out.push_str("</div><code>");
            if name(n) == "HTMLResource" {
                out.push_str("Embedded markup")
            } else {
                write!(out, "{}", H(val(n))).unwrap();
            }
            out.push_str("</code></li>");
        }
        out.push_str("</ul>");
    }
}
fn destinations(out: &mut String, members: &[Element<'_>]) {
    let mut links = members
        .iter()
        .copied()
        .filter(|n| {
            matches!(
                name(*n),
                "ClickThrough"
                    | "NonLinearClickThrough"
                    | "CompanionClickThrough"
                    | "IconClickThrough"
                    | "QrCodeScanUrl"
            )
        })
        .peekable();
    if links.peek().is_none() {
        return;
    }
    out.push_str("<h4>Viewer destinations</h4><ul class=resources>");
    for n in links {
        let label = match name(n) {
            "NonLinearClickThrough" => "Activate the non-linear unit",
            "CompanionClickThrough" => "Select the companion",
            "IconClickThrough" => "Select the icon",
            "QrCodeScanUrl" => "Scan the QR code",
            _ => "Select the linear ad",
        };
        out.push_str("<li>");
        link(out, n, label);
        write!(out, "<code>{}</code></li>", H(val(n))).unwrap();
    }
    out.push_str("</ul>");
}
fn measurement(out: &mut String, members: &[Element<'_>]) {
    let beacons: Vec<_> = members
        .iter()
        .copied()
        .filter(|n| {
            matches!(
                name(*n),
                "Tracking"
                    | "Impression"
                    | "Error"
                    | "ClickTracking"
                    | "NonLinearClickTracking"
                    | "CompanionClickTracking"
                    | "IconClickTracking"
                    | "IconViewTracking"
                    | "Viewable"
                    | "NotViewable"
                    | "ViewUndetermined"
            )
        })
        .collect();
    if !beacons.is_empty() {
        out.push_str("<h4>Measurement events</h4><div class=chips>");
        for n in beacons {
            let event = n.attribute("event").unwrap_or(name(n));
            link(out, n, event);
        }
        out.push_str("</div>");
    }
}
fn findings(out: &mut String, result: &ValidationResult, index: Option<&Index<'_>>) {
    out.push_str("<section id=findings class=panel><h2>Validation findings</h2>");
    let summary = &result.summary;
    write!(
        out,
        "<p>{} · {} · {} informational</p>",
        quantity(summary.errors, "error"),
        quantity(summary.warnings, "warning"),
        summary.infos
    )
    .unwrap();
    if result.issues.is_empty() {
        out.push_str("<p class=success>VASTlint reported no findings for this document.</p>");
    }
    for finding in &result.issues {
        write!(
            out,
            "<div class=finding><strong>{}</strong> <code>{}</code><p>{}</p>",
            H(finding.severity.as_str()),
            H(finding.id),
            H(finding.message)
        )
        .unwrap();
        if let Some(p) = &finding.path {
            write!(out, "<code>{}</code> ", H(p)).unwrap();
        }
        if let (Some(line), Some(ix)) = (finding.line, index) {
            let label = format!("Line {line}");
            let at = ix
                .lines
                .get(line.saturating_sub(1) as usize)
                .map(|s| s + finding.col.unwrap_or(1).saturating_sub(1) as usize);
            if let Some(node) = at
                .and_then(|offset| {
                    ix.nodes
                        .binary_search_by_key(&offset, |n| n.range().start)
                        .ok()
                })
                .map(|i| ix.nodes[i])
            {
                link(out, node, &label);
            } else {
                write!(out, "{}", H(&label)).unwrap();
            }
        }
        out.push_str("</div>");
    }
    out.push_str("</section>");
}

fn element(out: &mut String, n: Element<'_>, ix: &Index<'_>, xml: &str) {
    let key = name(n);
    let number = id(n);
    let line = ix.line(n);
    let known = ix.recognized(n);
    let generic = "This element belongs to an unrecognized vocabulary or extension. Its meaning depends on the producer and the receiving integration.";
    let desc = if known {
        guide::element(key).unwrap_or(generic)
    } else {
        generic
    };
    reader::row(out, n, ix, xml, desc);
    write!(out,"<div class=inside><div class=detailhead><strong>{}</strong><span>{} · line {line}</span></div><p>{}</p>",H(qualified(n,xml)),reader::purpose(n,ix),H(desc)).unwrap();
    if known && key == "Tracking" {
        if let Some(ev) = n.attribute("event") {
            write!(out, "<p><b>{}</b> · {}</p>", H(ev), guide::event(ev)).unwrap();
        }
    }
    if known {
        if let Some(signal) = guide::signal(key, val(n)) {
            write!(out, "<p><b>{}</b></p>", H(signal)).unwrap();
        }
    }
    if n.attributes().len() > 0 {
        out.push_str("<dl>");
        for a in n.attributes() {
            let attr = &xml[a.range_qname()];
            write!(
                out,
                "<dt><code>{}</code></dt><dd><code>{}</code>",
                H(attr),
                H(a.value())
            )
            .unwrap();
            if known && a.namespace().is_none() {
                if let Some(meaning) = guide::attribute(a.name()) {
                    write!(out, "<small>{}</small>", H(meaning)).unwrap();
                }
            }
            out.push_str("</dd>");
        }
        out.push_str("</dl>");
    }
    for t in n
        .children()
        .filter(Node::is_text)
        .filter_map(|t| t.text())
        .filter(|s| !s.trim().is_empty())
    {
        write!(out, "<pre class=value>{}</pre>", H(t)).unwrap();
    }
    write!(
        out,
        "<div class=references><a href='#s{number}'>Source · line {line}</a>"
    )
    .unwrap();
    if let Some(parent) = n.parent_element() {
        link(out, parent, &format!("Parent · {}", qualified(parent, xml)));
    }
    out.push_str("</div>");
    let children: Vec<_> = n.children().filter(Node::is_element).collect();
    if !children.is_empty() {
        out.push_str("<div class='children chips'><b>Children</b>");
        for c in children {
            link(out, c, qualified(c, xml));
        }
        out.push_str("</div>");
    }
    out.push_str("</div></details>");
}
