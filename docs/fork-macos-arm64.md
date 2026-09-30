# Fork macOS arm64 builds

In `polpo666/herdr-gpui`, open **Actions → Fork macOS arm64 → Run workflow**,
select the branch, and run it. This workflow is manual and independent of the
upstream release workflow.

After the run succeeds, download its `Herdr-macos-arm64-<commit>` artifact.
Extract the artifact, then extract `Herdr-<commit>-macos-arm64.app.zip` to obtain
`Herdr.app`. `SHA256SUMS` verifies the inner ZIP. The app's
`Contents/Resources/BUILD.txt` records the source repository and full commit.

The build requires Apple Silicon and macOS 15 or newer. It is ad-hoc signed,
not Developer ID signed or Apple notarized, so macOS may require approval in
Privacy & Security before opening a downloaded copy. It contains the GUI only;
install the Herdr daemon separately. The upstream automatic updater stays
disabled for this development build.

The workflow uses the repository's pinned Rust toolchain, runs the release CLI
tests, verifies the binary is arm64, and includes third-party license notices.
It needs no signing secrets, publishes no GitHub Release, and retains artifacts
for 30 days. It does not run the full workspace or native GUI test suites.

Rust registry/git dependencies, compiled dependency artifacts under `target`,
and installed Cargo tools (including `cargo-about`) are cached between runs.
The cache key accounts for the Rust toolchain, dependency manifests/lockfile,
macOS deployment target, and the pinned cargo-about version. A matching
installed cargo-about is reused instead of compiled again. The first run of
this workflow version must populate the cache; dependency or toolchain changes
can require recompilation. Application code, linking, tests, and packaging still
run. Cache restore/save logs show whether reuse actually occurred.
