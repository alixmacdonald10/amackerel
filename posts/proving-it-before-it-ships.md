---
title: "Proving It Before It Ships"
date: "2026-07-05"
description: "A pipeline that scans the exact bytes it publishes. cargo audit, an OWASP ZAP gate, version-gated releases, and hands-off deploys with Watchtower."
---

<!--
DRAFT SCAFFOLD — section headers + notes only. Fill each section with prose.
Series: "Zero-Exposure" (3 of 3). Prev: Defense in the Response.
Source: .github/workflows/ci.yml, .zap/rules.tsv, cloud-init.yaml.tftpl (watchtower).
-->

# Proving It Before It Ships

<!--
HOOK: articles 1 and 2 hardened the running system. This one is about trust in the
pipeline: how do you know the thing you deployed is the thing you tested?
-->

## The pipeline at a glance

<!--
- Push to main -> version, audit, unit-tests, e2e-tests -> (gated) zap-scan -> release.
- Concurrency cancels superseded runs; default permissions contents: read.
- DIAGRAM: reuse / adapt the CI mermaid from the README.
-->

## Dependencies: cargo audit

<!--
- cargo audit against the RustSec advisory DB; fails on any advisory.
- Cheap gate that catches known-vulnerable crates before anything else runs.
-->

## Behavior: unit + e2e in parallel

<!--
- cargo test --features ssr --no-default-features.
- Playwright across chromium/firefox/webkit via cargo leptos end-to-end.
- (Aside) the Tailwind standalone install workaround for the cargo-leptos cold-cache bug.
-->

## The DAST gate: OWASP ZAP

<!--
- zap-scan builds the image, runs the container, health-polls /, then ZAP baseline scan.
- fail_action: true — a new alert fails the pipeline.
- .zap/rules.tsv encodes the intentional ignores (ties back to article 2's CSP decision).
-->

## The key idea: scanned image == published image

<!--
- Only if ZAP passes: docker save | gzip the tested image as an artifact.
- release job docker load's THAT artifact and pushes it — never a rebuild.
- Point: you publish the exact bytes that were scanned. No "works on my rebuild" gap.
-->

## Version-gated release

<!--
- version job reads semver from Cargo.toml, checks if the git tag already exists.
- Build/scan/publish only run when the version is new (should_release).
- Bumping Cargo.toml IS the release trigger; tags are bare semver (no v).
-->

## The last mile: Watchtower

<!--
- On the droplet, watchtower.service polls GHCR every 300s (--interval 300 --cleanup).
- Detects the changed :latest digest, pulls, recreates the container, prunes the old image.
- No SSH, no manual step: publishing a new :latest IS the deploy.
- Manual fallback: systemctl restart amackerel (ExecStartPre re-pulls :latest).
-->

## Takeaway

<!--
- A release pipeline is part of the security boundary, not just plumbing.
- Scan the artifact you ship, gate on it, and let a version bump drive the whole chain.
- Close the series: perimeter (1) + response (2) + pipeline (3) = zero-exposure end to end.
-->
