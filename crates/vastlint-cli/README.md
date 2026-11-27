# vastlint-cli

Command line validator for IAB VAST, VMAP, and DAAST. It checks a tag, inspects a wrapper chain, and can apply the fixes the engine knows how to make.

The binary is `vastlint`. It links the published `vastlint-core` crate and does not build the engine from a sibling checkout.

```sh
cargo install --locked --git https://github.com/aleksUIX/vastlint-tools --tag cli-v0.19.0 vastlint-cli && vastlint telemetry
vastlint check tag.xml
```

Releases of this crate use tags of the form `cli-vX.Y.Z` in [vastlint-tools](https://github.com/aleksUIX/vastlint-tools). Application releases in that repository do not publish this crate.

## Sharing usage statistics and tag XML

Setup asks whether `vastlint check` may send usage statistics and the full XML of checked tags to vastlint.org. We use those tags only for research and to improve VASTlint. Press Enter to accept the default Y, or type N to opt out of both. Your choice applies across projects and upgrades. After a bare Cargo or Homebrew install, the CLI asks at the first interactive file or URL check. Run `vastlint telemetry off` or `vastlint telemetry on` to change the choice later.

With sharing on, checks can send the CLI version, OS, random installation ID, and input count. Samples include checked VAST, VMAP, or DAAST XML from files, stdin, and fetched wrapper hops, plus the document version and validation counts. The server redacts known tracking identifiers before storage. [Data handling and research use](https://vastlint.org/privacy/#cli-telemetry)

CI and stdin checks skip the prompt. With no saved choice or explicit setting, they skip uploads. Set `telemetry = false` at the top of `vastlint.toml` to disable both for a project. The older `telemetry = true` setting enables statistics only and respects a saved opt-out. The explicit flags `--telemetry` and `--contribute-sample` request statistics and XML respectively for a single check.

## Explain the contents of a VAST tag

```sh
vastlint explain tag.xml --output report.html
cat tag.xml | vastlint explain - > report.html
```

Try the [annotated pause ad report](https://vastlint.org/samples/reports/static-pause.html) or follow the [guide to generating it from the sample XML and artwork](https://vastlint.org/docs/explain-vast/).

Use **QA player** to load a declared image, video, or audio asset, switch viewport shapes, compare its actual dimensions and duration with the tag, and copy playback notes. The log links playback milestones to their tracking declarations. Remote media loads on request. SIMID protocol testing is available in SIMID Studio.

Open the HTML directly in a browser or IDE preview. The annotated view pairs each XML element with its meaning for the player. It shows one row per element in source order, including a source line number for minified and multiline tags. Expand a row to see attributes and follow its source link. On narrow screens, the explanation appears directly under the XML. Use the purpose filter or search to isolate tracking, placement, and creative resources. Select **With findings** to see the affected declarations and their rule IDs. **Partner handoff** gives a copyable brief for each ad with questions about the declared placement, fallback, and player capabilities. **Trafficking** lists creative IDs, dimensions, file references, viewer destinations, and measurement for comparison with a booking. Expand a supported element for related guides and articles on vastlint.org. Reading links contain a fixed topic name. In **Trafficking**, compare declared AdCOM experience and position with the partner booking. Those selections stay in the report. Findings and raw XML have their own views. With scripting disabled all views and native row disclosures remain readable.

The report includes the input XML, including URLs, tokens and vendor payloads supplied in the tag. Opening the report runs offline. Image, video and audio declarations have a preview in their annotated XML row. **Load image**, **Load video** or **Load audio** requests that individual HTTPS media resource from the displayed host. The report leaves scripts, SIMID, frames, tracking beacons and wrapper URLs inert.

For a portable report with a thumbnail and full-size preview, bind a declared URL to a local copy of the asset:

```sh
vastlint explain tag.xml -o report.html --embed-asset https://cdn.example.com/pause.jpg ./pause.jpg
```

Repeat the option for additional assets (64 MiB combined). Each URL must match a previewable declaration. The local payload is stored once per mapped URL, even when the tag references it twice. Inline base64 media can also be previewed. Preview controls require JavaScript. Placement diagrams illustrate declared geometry. Vendor-specific elements retain their names, values and source locations.

Use one local UTF-8 XML file or stdin, up to 4 MiB. HTML goes to stdout unless you specify `--output` (`-o`). Exit 0 means the report was generated, including for malformed XML with diagnostics. I/O and usage errors exit 2. Use `vastlint check` to make validation errors fail a CI job.
