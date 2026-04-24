# Changelog

## [0.4.0](https://github.com/cognizhi/ai-heeczer/compare/v0.3.0...v0.4.0) (2026-04-24)


### Features

* **cli:** embed fixtures via include_dir, add `aih fixtures show` ([13d75f1](https://github.com/cognizhi/ai-heeczer/commit/13d75f13f9860449ba029f956e353d6b11a9fe94))
* **cli:** Phase 2 aih subcommands — score --detail, validate, bench, replay ([53b3f6e](https://github.com/cognizhi/ai-heeczer/commit/53b3f6ee8e041a2eee72bfdc7ff64673f166eea4))
* **cli:** rename binary 'aih' to 'heec' (all references updated) ([c7f4273](https://github.com/cognizhi/ai-heeczer/commit/c7f4273200de5c0b5228394b8756a4ffe5d1b053))
* **cli:** ship the aih developer CLI (ADR-0010) ([8b46a72](https://github.com/cognizhi/ai-heeczer/commit/8b46a7240341b622ce0303c20c51b025c94241a2))
* complete plans 0001-0015 — CI, docs, adapters, calibration, ingestion depth ([3f9bf97](https://github.com/cognizhi/ai-heeczer/commit/3f9bf973a2ea6a4f7f2a4270165a3b711d024cc6))
* **core-c:** expose stable C ABI for non-Rust SDKs ([4b4f979](https://github.com/cognizhi/ai-heeczer/commit/4b4f9796f387b458c140f1d34d8b78f24284b26d))
* **core:** implement deterministic Rust scoring engine with golden + schema + determinism tests ([c1689d8](https://github.com/cognizhi/ai-heeczer/commit/c1689d82e7ad3c6896688c70443250c7ab47c8d6))
* **core:** ProfileValidator + deny_unknown_fields on ScoringProfile, MSRV unify ([2d11a69](https://github.com/cognizhi/ai-heeczer/commit/2d11a69b8b091c40496ff1de16d4118e932d63c7))
* **dashboard:** scaffold Next.js 15 dashboard (plan 0010, ADR-0008) ([8176797](https://github.com/cognizhi/ai-heeczer/commit/81767975334715cd1665cf005d17c325a526d010))
* **foundation:** scoring core, C ABI, storage, and aih CLI ([1a5ca09](https://github.com/cognizhi/ai-heeczer/commit/1a5ca0943804e6207add2928b048904653476f8a))
* implement Category 3 — sync wrapper, http scaffold, govulncheck… ([#7](https://github.com/cognizhi/ai-heeczer/issues/7)) ([057d5dc](https://github.com/cognizhi/ai-heeczer/commit/057d5dca0679f3dd1ef4d0613fa269f81e56dd77))
* **ingest:** bootstrap heeczer-ingest service skeleton (axum + sqlx) ([664d4b8](https://github.com/cognizhi/ai-heeczer/commit/664d4b81c7945a95bb62f24519c4af088c81678c))
* **schema:** add canonical event.v1 schema, scoring profile/tier schemas, default profile + tiers, and golden fixtures ([8da79e4](https://github.com/cognizhi/ai-heeczer/commit/8da79e49271917c95eb7d0f4af86a8930977bb06))
* **schema:** TierSetValidator + activate aih validate tier (ADR-0010 Phase 2) ([f4f728f](https://github.com/cognizhi/ai-heeczer/commit/f4f728f40cd5adab7e11c5287012898a87246d9e))
* **sdk-go:** bootstrap heeczer-go HTTP client (plan 0007 foundation) ([5c09819](https://github.com/cognizhi/ai-heeczer/commit/5c09819a38e2cd9730c2eaaefa141ad6164f65c8))
* **sdk-java:** bootstrap heeczer-sdk Java HTTP client (plan 0009 foundation) ([0e17436](https://github.com/cognizhi/ai-heeczer/commit/0e174365e742fefcbbad6249652644ece7af60bf))
* **sdk-js:** bootstrap @heeczer/sdk HTTP client (plan 0005 foundation) ([adea0ac](https://github.com/cognizhi/ai-heeczer/commit/adea0ac100f00aac9c8015c7cb6f78dd7b12ca59))
* **sdk-py:** bootstrap heeczer Python client (plan 0006 foundation) ([9212206](https://github.com/cognizhi/ai-heeczer/commit/921220648c97d90ca0f50d6cfe03176635e2d439))
* **sdk-rust:** bootstrap heeczer crate — in-process native scoring (plan 0008) ([1a4c5d8](https://github.com/cognizhi/ai-heeczer/commit/1a4c5d858bbb6746bb5099db580b3bfb71b90ca8))
* **storage:** migration 0002 — append-only audit log + global-row unique indexes ([9fb81aa](https://github.com/cognizhi/ai-heeczer/commit/9fb81aad27f6f8c287b1b5de2680e2f4a8cbf5a6))
* **storage:** PostgreSQL migration parity + pg module (plan 0004) ([3ea94aa](https://github.com/cognizhi/ai-heeczer/commit/3ea94aac1b7738f5943f615fc754048f20a7c32f))
* **storage:** SQLite layer with sqlx migrations and append-only triggers ([8ff8a26](https://github.com/cognizhi/ai-heeczer/commit/8ff8a26ba921db997e13846180f879689c304a1f))


### Bug Fixes

* align Rust security CI with stable toolchain and local bootstrap ([3ada3d0](https://github.com/cognizhi/ai-heeczer/commit/3ada3d0b903061a76e318fe3189dfb268ddab152))
* apply multi-reviewer findings (security, code, test, arch) ([0b4b82a](https://github.com/cognizhi/ai-heeczer/commit/0b4b82acedad8e2d8aa8abcdbea7667ebd663247))
* **ci:** improve Workflow Defuser summary reporting ([38ec2cd](https://github.com/cognizhi/ai-heeczer/commit/38ec2cdcd06afd88c5c353803e66e02bc315bc26))
* **ci:** use clone instead of to_string for event_id in ingest handler ([f9d475f](https://github.com/cognizhi/ai-heeczer/commit/f9d475f6522a83a16879a594df4dc2ac95d3054d))
* correct release-please action SHA pin ([7d74495](https://github.com/cognizhi/ai-heeczer/commit/7d74495ca1cbf276b6567183a027d79bb96b2f1f))
* **foundation:** apply subagent review consensus ([6a62c8a](https://github.com/cognizhi/ai-heeczer/commit/6a62c8a1956b9a9eb6a511b24ac573f5103062c4))
* harden rust dependency checks ([e9fca72](https://github.com/cognizhi/ai-heeczer/commit/e9fca72bd8323a9d0160095c29fac57934693564))
* linting and build issue ([50d45a8](https://github.com/cognizhi/ai-heeczer/commit/50d45a86c1c7b921b153c3adccef1128bccf9acc))
* stable markdownlint config, bench-smoke split, pre-commit hook, Java CHANGELOG ([54b89c2](https://github.com/cognizhi/ai-heeczer/commit/54b89c2bf6c8a607da1b6f7d7afec5a17715843d))


### Documentation

* **adr:** write ADR-0011 (C ABI envelope) + amend ADR-0001 / ADR-0004; tick plan checkboxes ([fe952b6](https://github.com/cognizhi/ai-heeczer/commit/fe952b6bda7a3fc0d600f731e0627340c71d6f9d))
* **governance:** add ADR-0010 for the aih CLI and update PRD §12.21 + plans 0000/0003/0013 ([32cac39](https://github.com/cognizhi/ai-heeczer/commit/32cac399c1da08b53af1d6f9ad9177ac16515d93))
* **plans:** tick done-but-unticked items across plans 02-15 ([4dfdf3c](https://github.com/cognizhi/ai-heeczer/commit/4dfdf3cab17b6eb1bde3de9670be3985b9df7187))
* **prd,adr:** ADR-0012 dashboard test-orchestration view + amend ADR-0010 / PRD ([8aa35da](https://github.com/cognizhi/ai-heeczer/commit/8aa35dad7f09bbcd881a32e003e6c95fc1e36ab1))
* rewrite root README and add architecture/system-overview ([c4b2e84](https://github.com/cognizhi/ai-heeczer/commit/c4b2e84ffb4e9294f8221c37bc0fe33c84f6a6ea))

## [0.3.0](https://github.com/cognizhi/ai-heeczer/compare/v0.2.0...v0.3.0) (2026-04-24)


### Features

* complete plans 0001-0015 — CI, docs, adapters, calibration, ingestion depth ([3f9bf97](https://github.com/cognizhi/ai-heeczer/commit/3f9bf973a2ea6a4f7f2a4270165a3b711d024cc6))
* implement Category 3 — sync wrapper, http scaffold, govulncheck… ([#7](https://github.com/cognizhi/ai-heeczer/issues/7)) ([057d5dc](https://github.com/cognizhi/ai-heeczer/commit/057d5dca0679f3dd1ef4d0613fa269f81e56dd77))


### Bug Fixes

* linting and build issue ([50d45a8](https://github.com/cognizhi/ai-heeczer/commit/50d45a86c1c7b921b153c3adccef1128bccf9acc))
* stable markdownlint config, bench-smoke split, pre-commit hook, Java CHANGELOG ([54b89c2](https://github.com/cognizhi/ai-heeczer/commit/54b89c2bf6c8a607da1b6f7d7afec5a17715843d))


### Documentation

* **plans:** tick done-but-unticked items across plans 02-15 ([4dfdf3c](https://github.com/cognizhi/ai-heeczer/commit/4dfdf3cab17b6eb1bde3de9670be3985b9df7187))

## [0.2.0](https://github.com/cognizhi/ai-heeczer/compare/v0.1.0...v0.2.0) (2026-04-23)

### Features

* **cli:** embed fixtures via include_dir, add `aih fixtures show` ([13d75f1](https://github.com/cognizhi/ai-heeczer/commit/13d75f13f9860449ba029f956e353d6b11a9fe94))
* **cli:** Phase 2 aih subcommands — score --detail, validate, bench, replay ([53b3f6e](https://github.com/cognizhi/ai-heeczer/commit/53b3f6ee8e041a2eee72bfdc7ff64673f166eea4))
* **cli:** rename binary 'aih' to 'heec' (all references updated) ([c7f4273](https://github.com/cognizhi/ai-heeczer/commit/c7f4273200de5c0b5228394b8756a4ffe5d1b053))
* **cli:** ship the aih developer CLI (ADR-0010) ([8b46a72](https://github.com/cognizhi/ai-heeczer/commit/8b46a7240341b622ce0303c20c51b025c94241a2))
* **core-c:** expose stable C ABI for non-Rust SDKs ([4b4f979](https://github.com/cognizhi/ai-heeczer/commit/4b4f9796f387b458c140f1d34d8b78f24284b26d))
* **core:** implement deterministic Rust scoring engine with golden + schema + determinism tests ([c1689d8](https://github.com/cognizhi/ai-heeczer/commit/c1689d82e7ad3c6896688c70443250c7ab47c8d6))
* **core:** ProfileValidator + deny_unknown_fields on ScoringProfile, MSRV unify ([2d11a69](https://github.com/cognizhi/ai-heeczer/commit/2d11a69b8b091c40496ff1de16d4118e932d63c7))
* **dashboard:** scaffold Next.js 15 dashboard (plan 0010, ADR-0008) ([8176797](https://github.com/cognizhi/ai-heeczer/commit/81767975334715cd1665cf005d17c325a526d010))
* **foundation:** scoring core, C ABI, storage, and aih CLI ([1a5ca09](https://github.com/cognizhi/ai-heeczer/commit/1a5ca0943804e6207add2928b048904653476f8a))
* **ingest:** bootstrap heeczer-ingest service skeleton (axum + sqlx) ([664d4b8](https://github.com/cognizhi/ai-heeczer/commit/664d4b81c7945a95bb62f24519c4af088c81678c))
* **schema:** add canonical event.v1 schema, scoring profile/tier schemas, default profile + tiers, and golden fixtures ([8da79e4](https://github.com/cognizhi/ai-heeczer/commit/8da79e49271917c95eb7d0f4af86a8930977bb06))
* **schema:** TierSetValidator + activate aih validate tier (ADR-0010 Phase 2) ([f4f728f](https://github.com/cognizhi/ai-heeczer/commit/f4f728f40cd5adab7e11c5287012898a87246d9e))
* **sdk-go:** bootstrap heeczer-go HTTP client (plan 0007 foundation) ([5c09819](https://github.com/cognizhi/ai-heeczer/commit/5c09819a38e2cd9730c2eaaefa141ad6164f65c8))
* **sdk-java:** bootstrap heeczer-sdk Java HTTP client (plan 0009 foundation) ([0e17436](https://github.com/cognizhi/ai-heeczer/commit/0e174365e742fefcbbad6249652644ece7af60bf))
* **sdk-js:** bootstrap @heeczer/sdk HTTP client (plan 0005 foundation) ([adea0ac](https://github.com/cognizhi/ai-heeczer/commit/adea0ac100f00aac9c8015c7cb6f78dd7b12ca59))
* **sdk-py:** bootstrap heeczer Python client (plan 0006 foundation) ([9212206](https://github.com/cognizhi/ai-heeczer/commit/921220648c97d90ca0f50d6cfe03176635e2d439))
* **sdk-rust:** bootstrap heeczer crate — in-process native scoring (plan 0008) ([1a4c5d8](https://github.com/cognizhi/ai-heeczer/commit/1a4c5d858bbb6746bb5099db580b3bfb71b90ca8))
* **storage:** migration 0002 — append-only audit log + global-row unique indexes ([9fb81aa](https://github.com/cognizhi/ai-heeczer/commit/9fb81aad27f6f8c287b1b5de2680e2f4a8cbf5a6))
* **storage:** PostgreSQL migration parity + pg module (plan 0004) ([3ea94aa](https://github.com/cognizhi/ai-heeczer/commit/3ea94aac1b7738f5943f615fc754048f20a7c32f))
* **storage:** SQLite layer with sqlx migrations and append-only triggers ([8ff8a26](https://github.com/cognizhi/ai-heeczer/commit/8ff8a26ba921db997e13846180f879689c304a1f))

### Bug Fixes

* align Rust security CI with stable toolchain and local bootstrap ([3ada3d0](https://github.com/cognizhi/ai-heeczer/commit/3ada3d0b903061a76e318fe3189dfb268ddab152))
* apply multi-reviewer findings (security, code, test, arch) ([0b4b82a](https://github.com/cognizhi/ai-heeczer/commit/0b4b82acedad8e2d8aa8abcdbea7667ebd663247))
* **ci:** improve Workflow Defuser summary reporting ([38ec2cd](https://github.com/cognizhi/ai-heeczer/commit/38ec2cdcd06afd88c5c353803e66e02bc315bc26))
* **ci:** use clone instead of to_string for event_id in ingest handler ([f9d475f](https://github.com/cognizhi/ai-heeczer/commit/f9d475f6522a83a16879a594df4dc2ac95d3054d))
* correct release-please action SHA pin ([7d74495](https://github.com/cognizhi/ai-heeczer/commit/7d74495ca1cbf276b6567183a027d79bb96b2f1f))
* **foundation:** apply subagent review consensus ([6a62c8a](https://github.com/cognizhi/ai-heeczer/commit/6a62c8a1956b9a9eb6a511b24ac573f5103062c4))
* harden rust dependency checks ([e9fca72](https://github.com/cognizhi/ai-heeczer/commit/e9fca72bd8323a9d0160095c29fac57934693564))

### Documentation

* **adr:** write ADR-0011 (C ABI envelope) + amend ADR-0001 / ADR-0004; tick plan checkboxes ([fe952b6](https://github.com/cognizhi/ai-heeczer/commit/fe952b6bda7a3fc0d600f731e0627340c71d6f9d))
* **governance:** add ADR-0010 for the aih CLI and update PRD §12.21 + plans 0000/0003/0013 ([32cac39](https://github.com/cognizhi/ai-heeczer/commit/32cac399c1da08b53af1d6f9ad9177ac16515d93))
* **prd,adr:** ADR-0012 dashboard test-orchestration view + amend ADR-0010 / PRD ([8aa35da](https://github.com/cognizhi/ai-heeczer/commit/8aa35dad7f09bbcd881a32e003e6c95fc1e36ab1))
* rewrite root README and add architecture/system-overview ([c4b2e84](https://github.com/cognizhi/ai-heeczer/commit/c4b2e84ffb4e9294f8221c37bc0fe33c84f6a6ea))
