# Release impact note

Attach this note to any PR with non-trivial release impact.

## Affected packages
<!-- Tick all that apply. -->
- [ ] `core` (Rust crate `heeczer-core`)
- [ ] `core-c` (C ABI)
- [ ] `bindings/rust` (`heeczer`)
- [ ] `bindings/node` (`@ai-heeczer/sdk`)
- [ ] `bindings/python` (`ai-heeczer`)
- [ ] `bindings/go`
- [ ] `bindings/java`
- [ ] `server/ingestion`
- [ ] `dashboard`
- [ ] container images

## Version bump per package
<!-- Use semver. Justify each bump. -->

| Package | Current | Next | Reason |
| --- | --- | --- | --- |
|         |         |      |        |

## Public-surface changes
- API additions:
- API modifications:
- API removals (breaking):

## Migration / upgrade notes
<!-- What must consumers do? Required steps, env var renames, schema migrations. -->

## Schema / scoring version changes
- `spec_version`: <unchanged | new>
- `scoring_version`: <unchanged | new>
- Golden fixtures updated: <yes | no>

## Backward compatibility
<!-- For each affected package, describe back-compat guarantees and deprecation timing. -->

## Rollback plan
<!-- How does an operator roll back if the release misbehaves? -->
