//! Static, source-ordered annotations. Each disclosure remains usable without JS.
use super::super::assets::{Embedded, Preview};
use super::{child, guide, id, name, qualified, quantity, val, Element, Index, H};
use std::fmt::Write;

pub(super) fn purpose(n: Element<'_>, ix: &Index<'_>) -> &'static str {
    if !ix.recognized(n) {
        return "Extension";
    }
    match name(n) {
        "plcmt" | "pos" | "attr" | "playbackmethod" | "CreativeId" => "Placement",
        "AdVerifications"
        | "Verification"
        | "VerificationParameters"
        | "JavaScriptResource"
        | "ExecutableResource" => "Verification",
        "VideoClicks" | "IconClicks" | "InteractiveCreativeFile" | "QrCodeScanUrl" => "Interaction",
        "Ad" | "VAST" | "AdSystem" | "Wrapper" | "InLine" | "AdServingId" | "VASTAdTagURI"
        | "VASTAdTagURL" | "Expires" => "Delivery",
        "Extensions" | "Extension" | "CreativeExtension" | "CreativeExtensions" => "Extension",
        k if k.contains("Tracking")
            || k.contains("Impression")
            || matches!(k, "Error" | "Viewable" | "NotViewable" | "ViewUndetermined") =>
        {
            "Measurement"
        }
        k if k.contains("Click") => "Interaction",
        _ => "Creative",
    }
}

// Read one opening tag without treating a quoted '>' as its terminator.
// Called on a parsed element, so names and quoted values are already well formed.
fn opening(out: &mut String, n: Element<'_>, xml: &str) {
    let raw = &xml[n.range()];
    let q = qualified(n, xml);
    write!(out, "&lt;<b class=tag>{}</b>", H(q)).unwrap();
    let mut rest = &raw[1 + q.len()..];
    loop {
        rest = rest.trim_start();
        if rest.starts_with("/>") {
            out.push_str("/&gt;");
            return;
        }
        if rest.starts_with('>') {
            out.push_str("&gt;");
            break;
        }
        let Some((attr, after)) = rest.split_once('=') else {
            break;
        };
        let value = after.trim_start();
        let Some(quote) = value.chars().next() else {
            break;
        };
        let Some(end) = value[quote.len_utf8()..].find(quote) else {
            break;
        };
        let stop = end + 2 * quote.len_utf8();
        write!(
            out,
            " <span class=attr>{}</span>=<span class=lit>{}</span>",
            H(attr.trim()),
            H(&value[..stop])
        )
        .unwrap();
        rest = &value[stop..];
    }
    if !n.children().any(|c| c.is_element()) {
        if let Some(close) = rest[1..].rfind("</") {
            write!(
                out,
                "<span class=text>{}</span>&lt;/<b class=tag>{}</b>&gt;",
                H(&rest[1..1 + close]),
                H(q)
            )
            .unwrap();
        }
    }
}

pub(super) fn row(
    out: &mut String,
    n: Element<'_>,
    ix: &Index<'_>,
    xml: &str,
    desc: &str,
    preview: Option<&Preview<'_>>,
    embeds: &mut Embedded,
) {
    let depth = n
        .ancestors()
        .skip(1)
        .filter(|p| p.is_element())
        .take(7)
        .count();
    let category = purpose(n, ix);
    write!(out,"<details class=entry id=e{} data-purpose='{category}'><summary><span class=xmlcell><span class=ln title='Source line {}'>{}</span><code class='snippet d{depth}'>",id(n),ix.line(n),ix.line(n)).unwrap();
    opening(out, n, xml);
    out.push_str("</code></span><span class=annotation>");
    if ix.recognized(n) {
        annotation(out, n, desc);
    } else {
        out.push_str("Vendor-defined payload. Interpretation depends on the integration.");
    }
    if let Some(preview) = preview {
        preview.card(out, embeds);
    }
}

fn annotation(out: &mut String, n: Element<'_>, desc: &str) {
    let key = name(n);
    let value = val(n);
    if let Some(decoded) = guide::signal(key, value) {
        let label = match key {
            "plcmt" => "Intended experience",
            "pos" => "On-screen position",
            "attr" => "Creative property",
            _ => "Playback trigger",
        };
        write!(out, "{label} · <b>{}</b>.", H(decoded)).unwrap();
        return;
    }
    match key {
        "AdSystem" if !value.is_empty() => {
            write!(out, "Issued by <b>{}</b>.", H(value)).unwrap();
        }
        "AdTitle" if !value.is_empty() => {
            write!(out, "Creative title · <b>{}</b>.", H(value)).unwrap();
        }
        "CreativeId" if !value.is_empty() => {
            write!(
                out,
                "Applies these instructions to creative <b>{}</b>.",
                H(value)
            )
            .unwrap();
        }
        "Tracking" if n.has_attribute("event") => {
            let event = n.attribute("event").unwrap();
            write!(
                out,
                "Request this beacon for <b>{}</b>. {}",
                H(event),
                H(guide::event(event))
            )
            .unwrap();
        }
        "Impression" => out.push_str("Request this beacon when the player records the impression."),
        "Duration" if !value.is_empty() => {
            write!(out, "Declared running time · <b>{}</b>.", H(value)).unwrap();
        }
        "StaticResource" | "MediaFile" => {
            let mime = n
                .attribute("creativeType")
                .or(n.attribute("type"))
                .unwrap_or("unspecified");
            write!(out, "Offers a resource with media type <b>{}</b>.", H(mime)).unwrap();
        }
        "InteractiveCreativeFile" => {
            let framework = n.attribute("apiFramework").unwrap_or("interactive");
            write!(
                out,
                "A compatible player can launch this <b>{}</b> creative.",
                H(framework)
            )
            .unwrap();
        }
        "NonLinear" if n.has_attribute("width") && n.has_attribute("height") => {
            write!(
                out,
                "Declares a <b>{} × {}</b> non-linear creative surface.",
                H(n.attribute("width").unwrap()),
                H(n.attribute("height").unwrap())
            )
            .unwrap();
        }
        "Extension" if n.attribute("type") == Some("ctv_ad_portfolio") => {
            out.push_str("Supplies placement and media instructions to CTV portfolio players.");
        }
        "VAST" => out.push_str("The player reads this response to discover the available ads."),
        "Ad" => match n.attribute("sequence") {
            Some(seq) => write!(out, "Offer at pod position <b>{}</b>.", H(seq)).unwrap(),
            None => out.push_str("One ad offer in this response."),
        },
        "InLine" => out.push_str("The player gets creative instructions directly from this tag."),
        "Wrapper" => out.push_str("The player needs the downstream VAST response to continue."),
        _ => {
            let first = desc.find(". ").map_or(desc, |end| &desc[..=end]);
            write!(out, "{}", H(first)).unwrap();
        }
    }
}

/// A repeated URL only identifies a fallback pair when both declarations belong
/// to the same creative. Keep the one-line overview conservative for ad pods
/// and exports with alternative resources.
pub(super) fn shared_pause_image<'a>(body: Element<'a>, ix: &Index<'_>) -> Option<Element<'a>> {
    let one = |group, key| {
        let mut candidates = Element::children(&group)
            .filter(move |n| n.is_element() && name(*n) == key && ix.recognized(*n));
        let first = candidates.next()?;
        candidates.next().is_none().then_some(first)
    };
    let creative = one(child(body, "Creatives")?, "Creative")?;
    let unit = one(child(creative, "NonLinearAds")?, "NonLinear")?;
    let image = one(unit, "StaticResource")?;
    let extensions = child(body, "Extensions")?;
    let mut portfolio = extensions.children().filter(|n| {
        n.is_element()
            && name(*n) == "Extension"
            && ix.recognized(*n)
            && n.attribute("type") == Some("ctv_ad_portfolio")
    });
    let ext = portfolio.next()?;
    if portfolio.next().is_some()
        || child(ext, "CreativeId")
            .is_some_and(|binding| Some(val(binding)) != creative.attribute("id"))
    {
        return None;
    }
    let media = one(child(ext, "MediaFiles")?, "MediaFile")?;
    let mime = image.attribute("creativeType")?;
    (mime.starts_with("image/")
        && media.attribute("type") == Some(mime)
        && !val(image).is_empty()
        && val(image) == val(media))
    .then_some(image)
}

pub(super) fn overview(
    out: &mut String,
    ads: &[Element<'_>],
    ix: Option<&Index<'_>>,
    root: Option<Element<'_>>,
) {
    let Some((ad, ix)) = ads.first().zip(ix).filter(|_| ads.len() == 1) else {
        let title = match ads.len() {
            0 => "VAST document".into(),
            n => format!("{} in this response", quantity(n, "ad")),
        };
        write!(out, "<div class=hero><h1>{}</h1></div>", H(&title)).unwrap();
        return;
    };
    let body = child(*ad, "InLine").or_else(|| child(*ad, "Wrapper"));
    let nodes: Vec<_> = body
        .into_iter()
        .flat_map(|b| b.descendants())
        .filter(|n| n.is_element() && ix.recognized(*n))
        .collect();
    let mut positions = nodes.iter().copied().filter(|n| name(*n) == "pos");
    let pos = positions.next().filter(|_| positions.next().is_none());
    let mut formats = nodes.iter().copied().filter(|n| name(*n) == "plcmt");
    let format = formats.next().filter(|_| formats.next().is_none());
    let wrapper = body.is_some_and(|n| name(n) == "Wrapper");
    let title = if wrapper {
        "VAST wrapper".to_owned()
    } else if let Some(f) = format.and_then(|n| guide::signal("plcmt", val(n))) {
        if pos.is_some_and(|n| val(n) == "7") {
            format!("Fullscreen {} ad", f.to_lowercase())
        } else {
            format!("{f} ad")
        }
    } else if let Some(t) = body
        .and_then(|b| child(b, "AdTitle"))
        .map(val)
        .filter(|s| !s.is_empty())
    {
        t.to_owned()
    } else {
        "VAST ad".into()
    };
    write!(out, "<div class=hero><div><h1>{}</h1><p>", H(&title)).unwrap();
    if wrapper {
        out.push_str("The player needs the downstream tag to determine the final creative.");
    } else if format.is_some_and(|n| val(n) == "5") {
        out.push_str("The ad can appear when the viewer pauses the program.");
    } else if format.is_some_and(|n| val(n) == "7") {
        out.push_str("The ad is designed to appear over the program.");
    } else if nodes.iter().any(|n| name(*n) == "Linear") {
        out.push_str("This response declares a creative for playback in the content timeline.");
    } else {
        out.push_str(
            "The tag describes the creative and the instructions available to the player.",
        );
    }
    if !wrapper {
        if let Some(n) = nodes
            .iter()
            .find(|n| name(**n) == "InteractiveCreativeFile")
        {
            write!(
                out,
                " A compatible player can launch the {} resource.",
                H(n.attribute("apiFramework").unwrap_or("interactive"))
            )
            .unwrap();
        } else if let Some(image) = body.and_then(|b| shared_pause_image(b, ix)) {
            out.push_str(" The CTV extension and the native fallback reference ");
            super::link(out, image, "the same image");
            out.push('.');
        } else if let Some(n) = nodes.iter().find(|n| {
            name(**n) == "StaticResource"
                && n.parent_element().is_some_and(|p| name(p) == "NonLinear")
        }) {
            write!(
                out,
                " The non-linear creative supplies an asset of type {}.",
                H(n.attribute("creativeType").unwrap_or("file-based"))
            )
            .unwrap();
        }
    }
    out.push_str("</p><div class=metadata>");
    if let Some(v) = root.and_then(|r| r.attribute("version")) {
        write!(out, "<span>VAST {}</span>", H(v)).unwrap();
    }
    if let Some(b) = body {
        write!(out, "<span>{}</span>", H(name(b))).unwrap();
    }
    if let Some(n) = body.and_then(|b| child(b, "AdSystem")) {
        write!(out, "<span>{}</span>", H(val(n))).unwrap();
    }
    out.push_str("</div></div>");
    if let Some((p, label)) = pos
        .filter(|n| {
            matches!(
                val(*n),
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
            )
        })
        .and_then(|n| guide::signal("pos", val(n)).map(|s| (n, s)))
    {
        write!(out,"<a class=schematic href='#e{}'><span aria-hidden=true class='screen p{}'><i></i></span><small>{} · schematic</small></a>",id(p),H(val(p)),H(label)).unwrap();
    }
    out.push_str("</div>");
}
