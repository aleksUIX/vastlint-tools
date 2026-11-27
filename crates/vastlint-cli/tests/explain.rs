use std::io::Write;
use std::process::{Command, Output, Stdio};

fn explain(xml: &str) -> Output {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_vastlint"))
        .args(["explain", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    cmd.stdin.take().unwrap().write_all(xml.as_bytes()).unwrap();
    cmd.wait_with_output().unwrap()
}

#[test]
fn explains_namespaced_elements_and_keeps_private_extensions_opaque() {
    let xml = r#"<v:VAST xmlns:v="http://www.iab.com/VAST" version="4.3"><v:Ad><v:InLine><v:Extensions>
<v:Extension type="pos" ext="adcom"><v:pos>7</v:pos></v:Extension>
<v:Extension type="vendor"><v:pos>14</v:pos><v:MediaFile type="secret/private">host</v:MediaFile></v:Extension>
</v:Extensions></v:InLine></v:Ad></v:VAST>"#;
    let output = explain(xml);
    assert!(output.status.success());
    let report = String::from_utf8(output.stdout).unwrap();
    assert!(report.starts_with("<!doctype html>"));
    assert_eq!(report.matches("<details class=entry").count(), 9);
    assert!(report.contains("pos 7 · Fullscreen"));
    assert!(!report.contains("pos 14 · Bottom left"));
    assert!(!report.contains("<h4>Declared resources</h4>"));
    assert!(report.contains("unrecognized vocabulary"));
    assert!(report.contains("v:MediaFile"));
}

#[test]
fn reports_wrapper_and_inline_separately_and_leaves_creative_markup_inert() {
    let xml = r#"<VAST version="3.0"><Ad id="hop"><Wrapper><VASTAdTagURI>https://example.org/next</VASTAdTagURI></Wrapper></Ad><Ad><InLine><AdTitle>&lt;script&gt;</AdTitle><Creatives><Creative><CompanionAds><Companion><HTMLResource><![CDATA[</script><img src=x>]]></HTMLResource></Companion></CompanionAds></Creative></Creatives></InLine></Ad></VAST>"#;
    let report = String::from_utf8(explain(xml).stdout).unwrap();
    assert_eq!(report.matches("<article class=ad data-ad=").count(), 4);
    assert!(report.contains("Next tag · "));
    assert!(report.contains("a companion surface"));
    assert!(report.contains("&lt;/script&gt;&lt;img src=x&gt;"));
    assert!(!report.contains("<img src=x>"));
    assert_eq!(report.matches("</script>").count(), 1);
}

#[test]
fn partially_parseable_input_keeps_its_source_and_validator_findings() {
    let report = String::from_utf8(explain("<VAST><Ad></VAST>").stdout).unwrap();
    assert!(report.contains("Element map unavailable"));
    assert!(report.contains("id=findings"));
    assert!(report.contains("&lt;VAST&gt;&lt;Ad&gt;&lt;/VAST&gt;"));
}

#[test]
fn output_can_be_regenerated_and_input_cannot_be_overwritten() {
    let tmp = tempfile::tempdir().unwrap();
    let file = tmp.path().join("tag.xml");
    let html = tmp.path().join("report.html");
    let xml = "<VAST version=\"4.2\"/>";
    std::fs::write(&file, xml).unwrap();
    let run = |out: &std::path::Path| {
        Command::new(env!("CARGO_BIN_EXE_vastlint"))
            .arg("explain")
            .arg(&file)
            .arg("--output")
            .arg(out)
            .output()
            .unwrap()
    };
    for _ in 0..2 {
        let result = run(&html);
        assert!(result.status.success());
        assert!(result.stdout.is_empty());
    }
    assert!(std::fs::read_to_string(&html).unwrap().contains("Raw XML"));
    assert_eq!(run(&file).status.code(), Some(2));
    assert_eq!(std::fs::read_to_string(file).unwrap(), xml);
}

#[test]
fn ctv_portfolio_resolves_fallback_only_for_the_bound_creative() {
    let xml = r#"<VAST version="2.0"><Ad><InLine><Creatives><Creative id="one"><NonLinearAds><NonLinear><StaticResource creativeType="image/png">img</StaticResource></NonLinear></NonLinearAds></Creative></Creatives><Extensions><Extension type="ctv_ad_portfolio"><CreativeId>one</CreativeId><plcmt>5</plcmt></Extension></Extensions></InLine></Ad></VAST>"#;
    let report = String::from_utf8(explain(xml).stdout).unwrap();
    assert!(report.contains("can still use"));
    assert!(report.contains("Position unspecified"));
    let unbound = xml.replace("<CreativeId>one", "<CreativeId>two");
    assert!(!String::from_utf8(explain(&unbound).stdout)
        .unwrap()
        .contains("can still use"));
}

#[test]
fn annotated_rows_show_exact_source_tokens_and_decode_their_behavior() {
    let xml = r#"<VAST version='2.0'><Ad><InLine><Extensions><Extension
 type='ctv_ad_portfolio'><pos>7</pos><plcmt>5</plcmt></Extension></Extensions><AdTitle id="x>y">A &amp; B</AdTitle></InLine></Ad></VAST>"#;
    let report = String::from_utf8(explain(xml).stdout).unwrap();
    assert!(report.contains("<h1>Fullscreen pause ad</h1>"));
    assert!(report.contains("On-screen position · <b>Fullscreen</b>"));
    assert!(report.contains("class=lit>&quot;x&gt;y&quot;</span>"));
    assert!(report.contains("<span class=text>A &amp;amp; B</span>"));
    assert!(report.contains("title='Source line 2'"));
    assert_eq!(
        report.matches("class=xmlcell").count(),
        report.matches("class=annotation").count()
    );
    assert!(report.contains("id=scope"));
}

#[test]
fn previews_only_declared_media_and_embeds_reused_files_once() {
    let tmp = tempfile::tempdir().unwrap();
    let xml = tmp.path().join("test.xml");
    let bytes = tmp.path().join("poster.jpg");
    let url = "https://cdn.example.test/a.jpg?k=1&v=2";
    std::fs::write(&bytes, [0xff, 0xd8, 0xff, 0xd9]).unwrap();
    let tag = format!(
        r#"<VAST version="4.2"><Ad><InLine><Impression><![CDATA[{url}]]></Impression><Creatives><Creative><NonLinearAds><NonLinear><StaticResource creativeType="image/jpeg"><![CDATA[{url}]]></StaticResource><MediaFiles><MediaFile type="image/jpeg"><![CDATA[{url}]]></MediaFile><MediaFile type="video/mp4">https://media.example.test/video.mp4</MediaFile><InteractiveCreativeFile apiFramework="SIMID" type="text/html">https://media.example.test/ad.html</InteractiveCreativeFile></MediaFiles></NonLinear></NonLinearAds></Creative></Creatives><Extensions><Extension type="private"><MediaFile type="image/jpeg">https://example.test/hidden.jpg</MediaFile></Extension></Extensions></InLine></Ad></VAST>"#
    );
    std::fs::write(&xml, tag).unwrap();
    let cmd = |url: &str| {
        Command::new(env!("CARGO_BIN_EXE_vastlint"))
            .arg("explain")
            .arg(&xml)
            .arg("--embed-asset")
            .arg(url)
            .arg(&bytes)
            .output()
            .unwrap()
    };
    let out = cmd(url);
    assert!(out.status.success());
    let html = String::from_utf8(out.stdout).unwrap();
    assert_eq!(html.matches("data-file='asset-0'").count(), 2);
    assert_eq!(
        html.matches("href='data:image/jpeg;base64,/9j/2Q=='")
            .count(),
        1
    );
    assert_eq!(html.matches("<span class=asset data-kind=").count(), 3);
    assert!(html.contains("data-kind='video' data-mime='video/mp4' data-remote"));
    assert!(!html.contains("src=\"https://media.example.test"));
    assert_eq!(
        cmd("https://media.example.test/ad.html").status.code(),
        Some(2)
    );
}

#[test]
fn recognizes_inline_media_and_keeps_other_schemes_inert() {
    let xml = r#"<VAST><Ad><InLine><Creatives><Creative><Linear><MediaFiles><MediaFile type="audio/wav">data:audio/wav;base64,UklGRg==</MediaFile><MediaFile type="image/png">javascript:alert(1)</MediaFile><MediaFile type="video/mp4" apiFramework="VPAID">https://example.test/code</MediaFile></MediaFiles></Linear></Creative></Creatives></InLine></Ad></VAST>"#;
    let html = String::from_utf8(explain(xml).stdout).unwrap();
    assert!(html.contains("data-kind='audio' data-mime='audio/wav' data-inline"));
    assert!(!html.contains(" data-remote>"));
    assert_eq!(html.matches("class=asset-action type=button").count(), 1);
}

#[test]
fn trafficking_counts_exact_references_per_ad_and_separates_destinations() {
    let xml = r#"<VAST version="4.2"><Ad id="one"><InLine><Creatives><Creative><NonLinearAds><NonLinear width="960" height="540"><StaticResource creativeType="image/jpeg">https://assets.test/a?k=1</StaticResource><NonLinearClickThrough>https://brand.test</NonLinearClickThrough></NonLinear></NonLinearAds></Creative></Creatives><Extensions><Extension type="ctv_ad_portfolio"><MediaFiles><MediaFile>https://assets.test/a?k=1</MediaFile><MediaFile>https://assets.test/a?k=2</MediaFile></MediaFiles></Extension></Extensions><Impression>https://metrics.test/one</Impression></InLine></Ad><Ad id="two"><Wrapper><VASTAdTagURI>https://wrap.test/two</VASTAdTagURI></Wrapper></Ad></VAST>"#;
    let report = String::from_utf8(explain(xml).stdout).unwrap();
    let handoff = report
        .split("<section id=delivery")
        .nth(1)
        .unwrap()
        .split("</section>")
        .next()
        .unwrap();
    let ops = report
        .split("<section id=trafficking")
        .nth(1)
        .unwrap()
        .split("</section>")
        .next()
        .unwrap();
    let (one, two) = ops.split_once("<article class=ad data-ad=2>").unwrap();
    assert!(one.contains("2 distinct file references across 3 declarations"));
    assert!(one.contains("Slot 960 × 540"));
    assert!(one.contains("https://brand.test"));
    assert!(one.contains("1 impression declaration in this response"));
    assert!(!two.contains("assets.test"));
    assert!(two.contains("https://wrap.test/two"));
    assert!(two.contains("0 impression declarations"));
    assert!(!one.contains("wrap.test"));
    assert_eq!(
        handoff
            .matches("Can you share the downstream response")
            .count(),
        1
    );
    assert_eq!(handoff.matches("Does creative review accept").count(), 1);
    assert!(!handoff.contains("Which target players support this SIMID"));
}

#[test]
fn located_finding_links_back_to_its_exact_minified_xml_row() {
    let report = String::from_utf8(explain(r#"<VAST version="4.2"><Ad><InLine><AdTitle>Hé</AdTitle><AdSystem>ads</AdSystem></InLine></Ad></VAST>"#).stdout).unwrap();
    let rule = "VAST-2.0-adsystem-no-version";
    let row = report
        .split("<details class=entry id=e")
        .skip(1)
        .map(|s| s.split("</details>").next().unwrap())
        .find(|s| s.contains(rule))
        .unwrap();
    let (element_id, _) = row.split_once(' ').unwrap();
    assert!(row.contains("class=issue-count"));
    assert!(row.contains(">AdSystem</b>"));
    let (anchor, _) = row
        .split("class=row-finding><a href='#f")
        .find(|s| s.contains(rule) && s.starts_with(|c: char| c.is_ascii_digit()))
        .unwrap()
        .split_once('\'')
        .unwrap();
    let card = report
        .split(&format!("class=finding id=f{anchor}>"))
        .nth(1)
        .unwrap()
        .split("</div>")
        .next()
        .unwrap();
    assert!(card.contains(rule));
    assert!(card.contains(&format!("href='#e{element_id}'")));
}

#[test]
fn player_keeps_pod_assets_and_tracking_scoped_and_wrapper_unresolved() {
    let xml = "<VAST version='4.2'>\r\n<Ad id='upstream'><Wrapper><VASTAdTagURI>https://wrapper.test/next</VASTAdTagURI></Wrapper></Ad><Ad id='second'><InLine><Creatives><Creative><Linear><Duration>00:00:12</Duration><TrackingEvents><Tracking event='midpoint'>https://track.test/m</Tracking></TrackingEvents><MediaFiles><MediaFile width='320' height='180' type='video/mp4'>https://cdn.test/file.mp4</MediaFile></MediaFiles></Linear></Creative></Creatives></InLine></Ad></VAST>";
    let report = String::from_utf8(explain(xml).stdout).unwrap();
    let panel = report
        .split_once("<section id=player")
        .unwrap()
        .1
        .split_once("</section>")
        .unwrap()
        .0;
    assert!(panel.contains("label='Ad 1 · upstream'></optgroup>"));
    assert!(panel.contains("label='Ad 2 · second'><option"));
    assert!(panel.contains("data-duration='00:00:12'"));
    assert!(panel.contains("data-width='320' data-height='180'"));
    assert!(panel.contains("Inspect wrapper URL"));
    let owner = panel
        .split_once("data-owner='")
        .unwrap()
        .1
        .split_once('\'')
        .unwrap()
        .0;
    assert!(panel.contains(&format!("data-owner='{owner}' data-event='midpoint'")));
    assert!(report
        .split_once("<section id=source")
        .unwrap()
        .1
        .contains("&gt;&#13;\n"));
}

#[test]
fn learning_links_use_public_vocabulary_and_ignore_private_payloads() {
    let report = String::from_utf8(explain(r#"<VAST><Ad><InLine><Creatives><Creative><Linear><TrackingEvents><Tracking event="firstQuartile">https://private.test/token</Tracking><Tracking event="secret-partner-name">x</Tracking></TrackingEvents></Linear></Creative></Creatives><Extensions><Extension type="vendor"><pos>7</pos></Extension></Extensions></InLine></Ad></VAST>"#).stdout).unwrap();
    let topics: Vec<_> = report
        .split("href='https://vastlint.org/search/explain/")
        .skip(1)
        .map(|s| s.split_once('\'').unwrap().0)
        .collect();
    assert!(topics.contains(&"tracking-firstquartile/"));
    assert!(topics.contains(&"tracking-events/"));
    assert!(!topics.contains(&"on-screen-position/"));
    assert!(topics
        .iter()
        .all(|s| !s.contains("private") && !s.contains("secret")));
}
