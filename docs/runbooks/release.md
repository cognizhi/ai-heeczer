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
    4. `mvn deploy` — publishes `heeczer-java` to Maven Central via Sonatype.
    5. `git tag go/vX.Y.Z` — pushes the Go module tag.
    6. `gh release create` — creates the GitHub Release with the SBOM,
       SLSA provenance, and binary attachments.
4. All steps run with `concurrency: { group: release, cancel-in-progress: false }`.

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
| `tag` | `v0.3.0` | The version tag that was partially released. |
| `skip_crates` | `true` | Set `true` if `cargo publish` already succeeded. |
| `skip_npm` | `true` | Set `true` if `npm publish` already succeeded. |
| `skip_pypi` | `true` | Set `true` if PyPI publish already succeeded. |
| `skip_maven` | `false` | Set `false` to re-run Maven Central deploy. |
| `skip_go_tag` | `false` | Set `false` to re-push the Go tag. |

The workflow checkouts the tag, skips already-published steps, and
completes the remainder.

### Step 3: verify

After `release-resume.yml` completes:

- **crates.io**: `cargo search heeczer` should show the new version.
- **npm**: `npm view @cognizhi/heeczer-sdk version` should show the new version.
- **PyPI**: `pip index versions heeczer-py` should list the new version.
- **Maven Central**: check [search.maven.org](https://search.maven.org/search?q=g:com.cognizhi+a:heeczer-java).
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
| `CRATES_IO_TOKEN` | crates.io | Scoped to `publish` only. |
| `NPM_TOKEN` | npm | OIDC provenance enabled; token is an automation token. |
| `SONATYPE_USERNAME` / `SONATYPE_PASSWORD` | Maven Central | Repository-level credentials. |
| `GITHUB_TOKEN` | GitHub | Provided automatically; needs `contents: write`, `id-token: write`. |

PyPI and npm also use OIDC trusted publishing — see ADR-0009 and
`docs/architecture/cicd.md` for the publisher configuration.
````
