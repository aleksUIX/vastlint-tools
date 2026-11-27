# vastlint-cli

Command line validator for IAB VAST, VMAP, and DAAST. It checks a tag, inspects a wrapper chain, and can apply the fixes the engine knows how to make.

The binary is `vastlint`. It links the published `vastlint-core` crate and does not build the engine from a sibling checkout.

```sh
cargo install --locked vastlint-cli
vastlint check tag.xml
```

Releases of this crate use tags of the form `cli-vX.Y.Z` in [vastlint-tools](https://github.com/aleksUIX/vastlint-tools). Application releases in that repository do not publish this crate.
