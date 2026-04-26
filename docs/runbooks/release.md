# Release Runbook

This runbook covers the normal release flow and partial-publish recovery
for ai-heeczer. The CI/CD design is documented in ADR-0009 and
`docs/architecture/cicd.md`.

---

## Normal release flow

1. **Merge to `main`** — `release-please.yml` creates or updates the
   Release PR automatically.
2. **Merge the Release PR** — this pushes a version tag (e.g. `v0.3.0`)
   which triggers `release.yml`.
3. **`release.yml` runs** in order:
    1. `cargo publish` — publishes `heeczer-core`, `heeczer-core-c`,
       `heeczer-storage`, `heeczer-cli`, `heeczer` to crates.io.
    2. `npm publish --provenance` — publishes `@cognizhi/heeczer-sdk` to npm.
    3. `pypa/gh-action-pypi-publish` — publishes `heeczer-py` to PyPI.
    4. `mvn deploy` — publishes `com.cognizhi:heeczer-sdk` to Maven Central via Sonatype.
    5. `git tag go/vX.Y.Z` — pushes the Go module tag.
    6. `gh release create` — creates the GitHub Release with the SBOM,
       SLSA provenance, and binary attachments.
4. All steps run with `concurrency: { group: release, cancel-in-progress: false }`.

The PyPI publish step uses a container action backed by GHCR. Pin it to an
official action release tag or to the commit behind that release tag; an
arbitrary SHA can fail before publish starts with `manifest unknown` because no
matching container image tag exists.

`release-dry-run.yml` also validates the Java publish path before a live tag by
running `mvn deploy` in `bindings/heeczer-java` with an ephemeral GPG key and
`-DskipPublishing=true`, which exercises sources/javadocs/signing and the
Central plugin wiring without attempting a live Sonatype publication.

---

## Partial-publish recovery

If `release.yml` fails mid-run (e.g. npm publish succeeds but PyPI
upload errors out), use `release-resume.yml` to retry the failed
downstream steps without re-running already-completed ones.

### Step 1: identify which registries were published

Check the failed run's logs in GitHub Actions. Each publish step prints a
`✅ Published <ecosystem> vX.Y.Z` line on success.

### Step 2: trigger `release-resume.yml`

````text
GitHub UI → Actions → "Release resume" → Run workflow
```text
Inputs:

| Input | Example | Description |
|---|---|---|
| `target` | `pypi` | Which publish target to resume: `rust`, `npm`, `pypi`, `go`, `java`, or `all`. |
| `tag` | `heeczer-py-v0.3.0` | The exact release tag to resume from, such as `v0.3.0` for the Rust release or `heeczer-py-v0.3.0` for the Python SDK release. |

The workflow checks out the requested tag and reruns only the selected target.
Use `all` only when the failed run was a root Rust release tag and multiple
downstream targets still need recovery.

### Step 3: verify

After `release-resume.yml` completes:

- **crates.io**: `cargo search heeczer` should show the new version.
- **npm**: `npm view @cognizhi/heeczer-sdk version` should show the new version.
- **PyPI**: `pip index versions heeczer-py` should list the new version.
- **Maven Central**: check [search.maven.org](https://search.maven.org/search?q=g:com.cognizhi+a:heeczer-sdk).
- **Go**: `go list -m cognizhi.com/heeczer-go@vX.Y.Z` should resolve.
- **GitHub Release**: the release page should be complete with all assets attached.

---

## Rollback / yanking

### crates.io

```bash
cargo yank --version X.Y.Z heeczer
cargo yank --version X.Y.Z heeczer-core
# repeat for each crate
```text
### npm

```bash
npm deprecate @cognizhi/heeczer-sdk@X.Y.Z "use vX.Y.Z+1 instead"
# npm does not support true yanking; use deprecation.
```text
### PyPI

```bash
# PyPI does not support yanking via CLI; use the web UI.
# https://pypi.org/manage/project/heeczer-py/releases/
```text
### Maven Central

Maven Central does not support deletion of published artifacts. Contact
Sonatype support if a critically broken artifact must be blocked.

### Go

Go module versions are immutable once tagged. Publish a patch release
(`vX.Y.Z+1`) with the fix and document the broken version in CHANGELOG.

---

## Hotfix release

1. Branch from the version tag: `git checkout -b hotfix/vX.Y.Z+1 vX.Y.Z`
2. Apply the fix and bump versions in `Cargo.toml`, `package.json`, `pyproject.toml`, `pom.xml`.
3. Open a PR against `main` (not the hotfix branch) and get it reviewed.
4. Cherry-pick the fix commit onto `main` if needed.
5. Manually push a new tag: `git tag vX.Y.Z+1 && git push origin vX.Y.Z+1`
6. `release.yml` triggers automatically.

---

## Secrets required for release

| Secret | Registry | Notes |
|---|---|---|
| `CARGO_REGISTRY_TOKEN` | crates.io | Scoped to `publish` only. |
| `NODE_AUTH_TOKEN` | npm | Automation token used by `npm publish --provenance`. |
| `CENTRAL_TOKEN_USERNAME` / `CENTRAL_TOKEN_PASSWORD` | Maven Central | Repository-level Sonatype Central credentials. |
| `GPG_PRIVATE_KEY` / `GPG_PASSPHRASE` | Maven Central | Required for artifact signing during `mvn deploy`. |
| `GITHUB_TOKEN` | GitHub | Provided automatically; needs `contents: write`, `id-token: write`. |

PyPI and npm also use OIDC trusted publishing — see ADR-0009 and
`docs/architecture/cicd.md` for the publisher configuration.
````
