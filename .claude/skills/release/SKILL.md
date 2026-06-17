---
name: release
argument-hint: v<X.Y.Z> | patch | minor | major
description: >-
  Orchestrate and monitor a full attested release of nsip end-to-end:
  version bump on develop, the develop→main Release PR, tag, attested
  binaries + completions/man-pages + MCPB bundle + SBOM + source snapshot,
  crates.io Trusted Publishing with .crate attestation, attested container
  image, automatic main→develop back-merge, Homebrew propagation, and
  independent workstation verification. Use this skill whenever the user
  invokes /release v<n.n.n> or /release patch|minor|major, or says "cut a
  release", "ship a release", "release version X", "bump and release",
  "do a patch/minor/major release", "tag a new version", or anything else
  that means publishing a new version of nsip. Do not improvise the release
  process from memory — this skill encodes nsip's develop/main flow and
  hard-won fixes for failure modes that are invisible until a release breaks.
---

# Release Orchestration (nsip)

Run a complete attested release for **nsip**. The argument is either an
explicit version (`/release v0.7.0`) or a bump type
(`/release patch|minor|major`), which computes the target from the current
`Cargo.toml` version. Every phase ends with a verification; do not proceed
past a failure — fix it or stop and report.

nsip is a **real published crate** (`publish = true`): every channel —
GitHub Release, crates.io, the container image, and Homebrew — is armed.
The crate/binary name is `nsip` and the repo is `zircote/nsip`; still
resolve them at the start (`cargo metadata --no-deps`,
`gh repo view --json nameWithOwner`) and use those values everywhere
`<bin>` / `<owner>/<repo>` appear, so the skill survives a rename.

## nsip branching model (this is the big difference from a single-branch repo)

`develop` is the integration/default branch; `main` is the release branch.
A release is a **promotion of develop into main**, then a tag on the main
merge commit. Never branch a release off `main` and never hand-roll the
develop→main PR — nsip ships a `Release PR` workflow for it.

```
bump version on develop ──> "Release PR" workflow opens develop→main PR
   ──merge──> tag main merge commit (vX.Y.Z) ──push──┬─> release.yml
                                                      ├─> publish.yml
                                                      ├─> docker.yml
                                                      └─> back-merge.yml (auto main→develop)
release.yml publishes the GitHub Release ─(release: published)─> package-homebrew.yml
```

The `release.yml` run is not just binaries: it builds the five platform
binaries, the shell-completions and man-page archives, and the MCPB
bundle; attaches SLSA provenance to every artifact; binds them all to a
CycloneDX SBOM; packs a **published source snapshot** and attests two
quality-gate verdicts over it — SCA (OSV-Scanner) and IaC/license (Trivy)
— through the central `zircote/.github` `reusable-attest-scan.yml` seam;
then **fail-closed verifies** every attestation before the GitHub Release
is created. SAST (CodeQL) and posture (Scorecard) are enforced at merge
time in `quality-gates.yml`, not re-run here.

## Help / no argument

If invoked with no argument, `--help`, or `help`, print this and stop —
do not start a release:

```
/release — attested release orchestration for nsip

USAGE
    /release v<X.Y.Z>     release an explicit version (e.g. /release v0.7.0)
    /release patch        bump X.Y.Z -> X.Y.(Z+1) and release
    /release minor        bump X.Y.Z -> X.(Y+1).0 and release
    /release major        bump X.Y.Z -> (X+1).0.0 and release

WHAT IT DOES
    bump version on develop -> "Release PR" workflow (develop->main) ->
    All Checks Pass green -> merge -> annotated tag on the main merge commit
    -> monitors: attested binaries + completions/man-pages + MCPB + SBOM +
    source snapshot + SCA/IaC-license gate attestations + fail-closed verify
    -> GitHub Release; crates.io Trusted Publishing + .crate attestation;
    attested container image; automatic main->develop back-merge; Homebrew
    auto-update -> independent workstation verification of every artifact.

NOTES
    - Publishing to crates.io is irreversible; versions are immutable.
    - Bump lands on develop; the release is a develop->main promotion.
    - Never re-runs release.yml against an existing tag (asset immutability).
```

## Phase 0 — Preflight

1. `git checkout develop && git pull --ff-only`; working tree must be clean
   of tracked changes (untracked noise is fine).
2. Resolve the target version from the argument:
   - `v<major>.<minor>.<patch>` → use as given.
   - `patch` | `minor` | `major` → read `version` from `Cargo.toml` on
     freshly-pulled develop and bump that component, zeroing the lower ones
     (`0.6.2` + `minor` → `0.7.0`; `0.6.2` + `major` → `1.0.0`).
   - Anything else → stop and ask.
   Strip the `v` for file edits; keep it for the tag. State the resolved
   version in the first progress message so a wrong bump is caught early.
3. Sanity checks, all hard stops:
   - New version is greater than `version` in `Cargo.toml`.
   - Tag does not already exist (`git tag -l v<X.Y.Z>` and check the remote
     with `git ls-remote --tags origin v<X.Y.Z>`).
   - `CHANGELOG.md` has content under `## [Unreleased]` — a release with an
     empty changelog means something is off; ask the user what this release
     contains.
   - Latest CI run on develop is green
     (`gh run list --workflow=ci.yml --branch develop --limit 1`).
4. Semver gut-check: if Unreleased contains breaking changes or an MSRV bump
   (currently 1.92) and the user asked for a patch, raise it before proceeding.

## Phase 1 — Version bump on develop

The version must land on `develop` **before** the Release PR. Branch off
develop, update **all** version locations (missing one ships inconsistent
metadata), and PR into develop:

| File | What to change |
| --- | --- |
| `Cargo.toml` | `version = "<X.Y.Z>"` (package version) |
| `Cargo.lock` | run `cargo check` after the Toml edit — never hand-edit (the `nsip` entry updates) |
| `CHANGELOG.md` | move the `## [Unreleased]` content under a new `## [<X.Y.Z>] - <today>` heading; keep an empty `## [Unreleased]`. Update the `[Unreleased]:` compare link and add the `[<X.Y.Z>]:` compare link. nsip uses git-cliff (`cliff.toml`) + `changelog.yml`; if that automation owns CHANGELOG.md in your flow, let it, but the section for this version must exist. |
| `manifest.json` | only if a version string is hand-maintained there; release.yml injects the version into the MCPB manifest at build time, so usually no edit is needed — confirm. |
| `SECURITY.md` / `CITATION.cff` | if present, any `<bin>-<version>` example or `version:` / `date-released:` fields. |

Validate locally before the PR: `cargo fmt -- --check` and `cargo check`
minimum (a broken lockfile or fmt failure must never reach the PR). The
full gauntlet runs in CI and again in release.yml's own gates.

Commit as `chore: bump version to <X.Y.Z>` (nsip's established style),
push the branch, open a PR **into develop**, and merge it once
`All Checks Pass` is green. Then `git checkout develop && git pull` so the
bump is local. (If the user explicitly wants the bump committed directly to
develop, that is also acceptable — develop is unprotected — but a PR is the
default.)

## Phase 2 — Release PR (develop → main)

Do **not** hand-roll this PR. Trigger nsip's workflow:

```bash
gh workflow run "Release PR" -f version=<X.Y.Z>
```

It opens (or updates) the `develop → main` promotion PR. Find it
(`gh pr list --base main --head develop --state open`) and monitor with the
Monitor tool. main's **only** required status check is `All Checks Pass`
(branch protection is non-strict — no up-to-date requirement). Use the
aggregate-gate guard, not a naive all-non-pending check:

```bash
# Terminal only when the aggregate gate itself has reported. Right after a
# push there is a window where only 1-2 checks are registered; "zero
# pending" alone declares victory in that window.
gate=$(jq -r '[.[] | select(.name=="All Checks Pass")][0].bucket // "absent"' <<<"$checks")
```

When `All Checks Pass` is green, merge the promotion PR with a **merge
commit** (`gh pr merge <n> --merge`) — *not* squash: develop→main is a
promotion, and squashing would diverge the two branches and break the
back-merge. Do **not** delete the `develop` branch. Then
`git checkout main && git pull` and confirm HEAD is the promotion merge commit.

## Phase 3 — Tag the main merge commit

Annotated tag on the merge commit, then push:

```bash
git tag -a v<X.Y.Z> -m "Release v<X.Y.Z>

<one-paragraph summary from the changelog>" <merge-sha>
git push origin v<X.Y.Z>
```

The tag push is the release trigger. Facts that matter:

- Tag pushes bypass branch protection — release.yml carries its own Test
  and Cargo Audit gates precisely because the tag is untrusted input.
- **Never re-dispatch release.yml against an existing tag.** Builds are not
  reproducible; it would overwrite published release assets with different
  bytes, violating the immutability the attestations exist to protect.
- Tag immediately after merging so the changelog/PR diffs stay clean.

## Phase 4 — Monitor the chains

Five things run off the tag; watch them with the Monitor tool (report each
as it lands):

1. **Release run** (`release.yml`). Expect all of these jobs to succeed:
   - `Resolve Version` (`version`)
   - `Test`, `Cargo Audit` (untrusted-tag gates; audit runs
     `--ignore RUSTSEC-2023-0071`)
   - `Build (<platform>)` × 5
   - `Completions & man pages` (`extras`), `Package MCPB Bundle` (`mcpb`)
   - `Source Snapshot` (`source`) — packs and attests the published
     `nsip-<X.Y.Z>-source.tar.gz`, the verifiable subject for the gates
   - `Gate — SCA (OSV)`, `Gate — Trivy (IaC/license)` — the two
     quality-gate scans (OSV uses `--config=osv-scanner.toml`)
   - `Attest — SCA`, `Attest — IaC/license` — gate verdicts signed over
     the source snapshot via the central `reusable-attest-scan.yml` seam
   - `SBOM (generate + attest)` (`sbom`)
   - `Verify Attestations` (`verify`) — fail-closed; asserts **10**
     bound subjects (5 binaries + 2 extras archives + MCPB + SBOM doc +
     source) and the source's two gate verdicts. A count mismatch is a
     hard stop, not a flake.
   - `Create Release` (`release`) — environment `copilot`; notes generated
     by git-cliff.
2. **Publish run** (`publish.yml`, the `Publish to crates.io` job,
   environment `copilot`). Report these step conclusions explicitly:
   "Run pre-publish checks", "Authenticate with crates.io" (Trusted
   Publishing, OIDC — no stored token), "Publish to crates.io"
   (`cargo publish`), then the crate-attestation steps "Download published
   crate from registry" (sha256 match against the local package) and
   "Attest crate provenance".
3. **Docker run** (`docker.yml`, container chain on the tag). Expect:
   `Build and Push Docker Image`, `Sign and Attest Image` (central
   `sign-and-attest.yml`, cosign), `Verify Image Attestations`
   (`verify-attestation.yml`, fail-closed), and the container gate pair
   `Gate — Trivy (image)` + `Attest — Container scan`. All success.
4. **Back-merge run** (`back-merge.yml`). Automatic on the tag push: opens
   and merges a `main → develop` PR so develop stays in sync. Confirm it
   ran and develop received the merge — if it failed, back-merge manually
   (`gh workflow run back-merge.yml`) and report.
5. **Homebrew run** (`package-homebrew.yml`) must appear **on its own**
   after the Release is *published*, via the `release: published` trigger,
   and update `zircote/homebrew-tap`. If no run appears within a few
   minutes of the Release, the trigger regressed — fall back to manual
   dispatch (`gh workflow run package-homebrew.yml -f version=<X.Y.Z>`)
   and investigate.

### Failure playbook

| Symptom | Cause | Action |
| --- | --- | --- |
| `Gate — SCA (OSV)` fails on RUSTSEC-2023-0071 (rsa) | OSV-Scanner flags the rsa Marvin advisory | nsip vets it non-applicable (HMAC-only JWT) in `deny.toml`, `cargo-audit --ignore`, **and** `osv-scanner.toml`. If it fires, the `--config=osv-scanner.toml` arg or the ignore entry was dropped — restore it. A *new* advisory must be added to all three. |
| `sast / analyze` fails: "advanced configurations cannot be processed when the default setup is enabled" | CodeQL default setup got re-enabled | Disable it again: `gh api -X PATCH repos/<owner>/<repo>/code-scanning/default-setup -f state=not-configured`. The gh-attested advanced CodeQL supersedes default setup. |
| `Gate — *` / `Verify Attestations` fails on the source snapshot | Gate verdict attestation missing or signer mismatch | Source is verified against `https://zircote.github.io/attestations/sca/v1` and `.../iac-license/v1` with `--signer-workflow zircote/.github/.github/workflows/reusable-attest-scan.yml`. Check the seam pin in `release.yml`. The downstream `Attest — *` jobs cannot run without a clean gate; fix the finding via a normal PR and restart at Phase 0. |
| `Verify Attestations` count mismatch (expected 10) | An artifact job failed/partial, or a gate SARIF leaked into the bound set | Downloads are scoped to `nsip-<version>-*`; a count ≠ 10 means a build/extras/mcpb/source/sbom artifact is missing. Find the failed upstream job; do not bypass the gate. |
| Publish auth fails: "No Trusted Publishing config found" | crates.io TP not configured | One-time on crates.io: crate `nsip` → Settings → Trusted Publishing → repo `<owner>/<repo>`, workflow `publish.yml`, environment `copilot`. Then `gh workflow run publish.yml --ref v<X.Y.Z>`. |
| Publish fails: "crate nsip@X.Y.Z already exists" | Duplicate publish raced a success | Benign. Verify the version is live (`cargo search nsip`), report, move on — crates.io versions are immutable. |
| Crate download step exhausts retries | static.crates.io CDN propagation | Re-run the failed `publish.yml` job; the publish itself succeeded. The step already retries 6× with backoff. |
| Crate sha256 mismatch (registry vs local) | Should never happen — packaging is deterministic per commit | Hard stop. Do not attest. Investigate first. |
| `Cargo Audit` fails | Real advisory in `Cargo.lock` | Fix via a normal PR into develop (often `cargo update <crate>`), then restart at Phase 0. cargo-deny may not flag it — audit scans the raw lockfile, deny the feature graph; keep both gates. |
| A build leg fails | Platform/toolchain issue | Legs: linux-amd64 (`ubuntu-latest`), linux-arm64 (`ubuntu-24.04-arm`), macos-amd64 (`x86_64-apple-darwin`), macos-arm64 (`aarch64-apple-darwin`), windows-amd64.exe (`windows-2022`, `x86_64-pc-windows-msvc`). Binaries build with default features (`--release --locked`). |
| Image verify fails on the tag run | Central signer/verify regression | Check the `zircote/.github` pins in `docker.yml` (`sign-and-attest.yml`, `verify-attestation.yml`) before anything else. |
| Back-merge PR conflicts | develop advanced during the release | Resolve the `main → develop` PR manually; never force the back-merge. |

## Phase 5 — Independent workstation verification

In-pipeline success is necessary; this is the acceptance test. Run from the
local machine, in a scratch dir:

```bash
gh release download v<X.Y.Z> --repo <owner>/<repo>
# Expect ~11 assets: 5 binaries, nsip-<X.Y.Z>-completions.tar.gz,
# nsip-<X.Y.Z>-man-pages.tar.gz, the MCPB bundle (nsip-<X.Y.Z>.mcpb),
# nsip-<X.Y.Z>-sbom.cdx.json, nsip-<X.Y.Z>-source.tar.gz, and
# nsip-<X.Y.Z>-checksums.txt

# Every shipped artifact (binaries, completions, man-pages, MCPB, source)
# carries provenance + an SBOM binding. Verify each:
for f in nsip-<X.Y.Z>-{linux-amd64,linux-arm64,macos-arm64,macos-amd64,windows-amd64.exe} \
         nsip-<X.Y.Z>-completions.tar.gz nsip-<X.Y.Z>-man-pages.tar.gz \
         nsip-<X.Y.Z>.mcpb nsip-<X.Y.Z>-source.tar.gz; do
  gh attestation verify "$f" --repo <owner>/<repo>                   # provenance
  gh attestation verify "$f" --repo <owner>/<repo> \
    --predicate-type https://cyclonedx.org/bom                       # SBOM binding
done

# Source snapshot also carries the two seam-signed gate verdicts.
for pt in sca iac-license; do
  gh attestation verify nsip-<X.Y.Z>-source.tar.gz --owner <owner> \
    --signer-workflow zircote/.github/.github/workflows/reusable-attest-scan.yml \
    --predicate-type "https://zircote.github.io/attestations/${pt}/v1"
done

shasum -a 256 -c nsip-<X.Y.Z>-checksums.txt

# crates.io: needs a User-Agent or the CDN may reject
curl -fsSL -A 'release-check' \
  -O https://static.crates.io/crates/nsip/nsip-<X.Y.Z>.crate
gh attestation verify nsip-<X.Y.Z>.crate --repo <owner>/<repo>

# Container image (digest from the docker.yml run's build-and-push output):
gh attestation verify "oci://ghcr.io/<owner>/<repo>@<digest>" \
  --repo <owner>/<repo> \
  --signer-workflow <owner>/.github/.github/workflows/sign-and-attest.yml \
  --predicate-type https://slsa.dev/provenance/v1
```

Check **exit codes**, not grepped output — a filtered pipe that matches
nothing looks identical to success; silence is not success. No
`--signer-workflow` for binary/extras/MCPB *provenance/SBOM* or the crate:
those are signed by this repo's own workflows. Two things DO need it: the
source-snapshot *gate verdicts* (central `reusable-attest-scan.yml`) and
the container image (central `sign-and-attest.yml`).

Confirm crates.io shows the version:
`curl -s -A 'release-check' https://crates.io/api/v1/crates/nsip | jq .crate.max_version`

## Final report

Summarize for the user: version, promotion merge commit, tag; per-channel
status (GitHub Release / crates.io / container image / Homebrew); whether
the automatic main→develop back-merge landed; workstation verification
results (binaries, completions, man-pages, MCPB, source snapshot + gate
verdicts, crate, image); and anything from the failure playbook that fired.
If any channel is incomplete, say exactly what is pending and what unblocks it.
