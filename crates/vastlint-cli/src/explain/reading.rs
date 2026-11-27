//! Public topic paths are fixed vocabulary. Never interpolate user-supplied XML into them.
use super::{id, link, name, val, Element, H};
use std::fmt::Write;

pub(super) fn reading(out: &mut String, n: Element<'_>) {
    // Only event names with published reference pages can become URL segments.
    if name(n) == "Tracking" {
        if let Some(event) = n.attribute("event").filter(|e| {
            matches!(
                *e,
                "creativeView"
                    | "start"
                    | "firstQuartile"
                    | "midpoint"
                    | "thirdQuartile"
                    | "complete"
                    | "mute"
                    | "unmute"
                    | "pause"
                    | "rewind"
                    | "resume"
                    | "close"
                    | "acceptInvitation"
                    | "fullscreen"
                    | "exitFullscreen"
                    | "expand"
                    | "collapse"
                    | "skip"
                    | "progress"
                    | "acceptInvitationLinear"
                    | "closeLinear"
                    | "playerExpand"
                    | "playerCollapse"
                    | "timeSpentViewing"
                    | "adExpand"
                    | "adCollapse"
                    | "minimize"
                    | "overlayViewDuration"
                    | "otherAdInteraction"
                    | "loaded"
                    | "interactiveStart"
            )
        }) {
            write!(out,"<a class=topic-link href='https://vastlint.org/search/explain/tracking-{}/' target='_blank' rel='noopener noreferrer'>When {} fires</a>",event.to_ascii_lowercase(), H(event)).unwrap();
            return;
        }
    }
    let topic = match name(n) {
        "plcmt" if val(n) == "5" => ("pause-ads", "Pause ad delivery"),
        "plcmt" if val(n) == "6" => ("screensaver-ads", "Screensaver ad delivery"),
        "pos" => ("on-screen-position", "On-screen position"),
        "attr" => ("creative-attributes", "CTV creative attributes"),
        "playbackmethod" => ("playback-method", "CTV playback triggers"),
        "plcmt" => ("placement-signals", "CTV placement signals"),
        "Extension" if n.attribute("type") == Some("ctv_ad_portfolio") => {
            ("placement-signals", "CTV Ad Portfolio extensions")
        }
        "StaticResource" if n.parent_element().is_some_and(|p| name(p) == "NonLinear") => {
            ("static-fallback", "Non-linear static fallback")
        }
        "NonLinear" | "NonLinearAds" => ("placement-signals", "Non-linear placement signals"),
        "InteractiveCreativeFile"
            if n.attribute("apiFramework")
                .is_some_and(|s| s.eq_ignore_ascii_case("SIMID")) =>
        {
            ("simid", "SIMID initialization and fallback")
        }
        "MediaFile" | "MediaFiles" => ("media-renditions", "Media rendition selection"),
        "Mezzanine" => ("mezzanine", "Mezzanine files and SSAI"),
        "Companion" | "CompanionAds" => ("companions", "Companion sizes and slot matching"),
        "Wrapper" | "VASTAdTagURI" | "VASTAdTagURL" => ("wrappers", "Resolving wrapper chains"),
        "Tracking" | "TrackingEvents" | "Duration" => {
            ("tracking-events", "Playback tracking events")
        }
        "Impression" => ("impressions", "Impressions and auction notices"),
        "ClickThrough"
        | "ClickTracking"
        | "VideoClicks"
        | "NonLinearClickThrough"
        | "NonLinearClickTracking"
        | "CompanionClickThrough"
        | "CompanionClickTracking"
        | "IconClickThrough"
        | "IconClickTracking" => ("clicks", "Click destinations and trackers"),
        "AdVerifications" | "Verification" | "VerificationParameters" | "JavaScriptResource" => {
            ("verification", "Ad verification and OMID")
        }
        "UniversalAdId" | "AdCreativeId" | "CreativeId" => ("creative-ids", "Creative identifiers"),
        "QrCodeScanUrl" | "QrCodeImageUrl" | "QrCodePosition" | "QrCodeSize" => {
            ("qr-codes", "CTV QR codes")
        }
        "Error" => ("error-codes", "VAST error codes"),
        "Ad" if n.has_attribute("sequence") => ("ad-pods", "Pod sequence"),
        _ => return,
    };
    write!(out, "<a class=topic-link href='https://vastlint.org/search/explain/{}/' target='_blank' rel='noopener noreferrer'>Read about {} on VASTlint</a>", topic.0, H(topic.1)).unwrap();
}

pub(super) fn booking(out: &mut String, body: Element<'_>, members: &[Element<'_>]) {
    if !members.iter().any(|&n| {
        matches!(name(n), "NonLinear" | "NonLinearAds" | "plcmt" | "pos")
            || (name(n) == "Extension" && n.attribute("type") == Some("ctv_ad_portfolio"))
    }) {
        return;
    }
    out.push_str("<details class=booking><summary>Compare with the partner booking</summary><p>Choose the expected AdCOM values for this ad. Each comparison uses declarations in this response.</p>");
    for (key, label, range) in [
        ("plcmt", "Expected experience", 1..=9),
        ("pos", "Expected position", 0..=17),
    ] {
        write!(out, "<div class=booking-field><label for='book-{}-{key}'>{label}</label><select hidden id='book-{}-{key}'><option value=''>Select a booked value</option>", id(body),id(body)).unwrap();
        for code in range {
            if let Some(meaning) = super::guide::signal(key, &code.to_string()) {
                write!(out, "<option value='{code}'>{code} · {meaning}</option>").unwrap();
            }
        }
        out.push_str("</select><span class=book-declared>");
        let mut any = false;
        for &n in members.iter().filter(|&&n| name(n) == key) {
            if any {
                out.push_str(", ");
            }
            any = true;
            write!(out, "<span data-code='{}'>", H(val(n))).unwrap();
            link(out, n, &format!("{key} {}", val(n)));
            out.push_str("</span>");
        }
        if !any {
            out.push_str("No declaration");
        }
        out.push_str("</span><output aria-live=polite></output></div>");
    }
    out.push_str("</details>");
}
