# vastlint-cli

Command line validator for IAB VAST, VMAP, and DAAST. It checks a tag, inspects a wrapper chain, and can apply the fixes the engine knows how to make.

The binary is `vastlint`. It links the published `vastlint-core` crate and does not build the engine from a sibling checkout.

```sh
cargo install --locked vastlint-cli
vastlint check tag.xml
```

Releases of this crate use tags of the form `cli-vX.Y.Z` in [vastlint-tools](https://github.com/aleksUIX/vastlint-tools). Application releases in that repository do not publish this crate.

## Explain the contents of a VAST tag

```sh
vastlint explain tag.xml --output report.html
cat tag.xml | vastlint explain - > report.html
```

Open the HTML directly in a browser or IDE preview. The annotated view pairs each XML element with its meaning for the player. It shows one row per element in source order, including a source line number for minified and multiline tags. Expand a row to see attributes and follow its source link. On narrow screens, the explanation appears directly under the XML. Use the purpose filter or search to isolate tracking, placement, and creative resources. Ad behavior, validation findings, and the original XML have their own views. With scripting disabled all views and native row disclosures remain readable.

The report includes the input XML, including URLs, tokens and vendor payloads supplied in the tag. It runs offline. Creative code, media files, tracking beacons and downstream wrappers are never loaded. Placement diagrams illustrate declared geometry. Vendor-specific elements retain their names, values and source locations.

Use one local UTF-8 XML file or stdin, up to 4 MiB. HTML goes to stdout unless you specify `--output` (`-o`). Exit 0 means the report was generated, including for malformed XML with diagnostics. I/O and usage errors exit 2. Use `vastlint check` to make validation errors fail a CI job.
