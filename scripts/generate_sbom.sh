#!/bin/sh
set -e
cargo install --locked cargo-cyclonedx --root ./target/cyclonedx >/dev/null 2>&1 || true
PATH=./target/cyclonedx/bin:$PATH cargo cyclonedx --all --format xml --output sbom.xml
if command -v gpg >/dev/null 2>&1; then
  gpg --batch --yes --armor --detach-sign sbom.xml
fi
