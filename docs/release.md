# Release Process

## 23. Full QA & Release Candidate
- Run unit tests and `cargo clippy`.
- Execute Cypress UI tests: `pnpm --dir app run test:e2e`.
- Check for memory leaks with `valgrind` on Linux and Instruments on macOS.
- Perform accessibility audit via `pnpm --dir app run a11y`.
- Sign Windows and macOS bundles using your platform certificates.
- Generate a CycloneDX SBOM and detach signature with `scripts/generate_sbom.sh`.

## 24. v1.0.0 Production Release
- Tag the release:
  ```bash
  git tag v1.0.0 && git push origin v1.0.0
  ```
- Publish artifacts `CLAppySetup.exe`, `.dmg`, and `.AppImage`.
- Provide SHA256 checksums for each artifact.
- Post the launch announcement and create `docs/roadmap.md` for v1.1.
