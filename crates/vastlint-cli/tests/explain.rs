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
    assert_eq!(report.matches("<article class=ad>").count(), 2);
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
