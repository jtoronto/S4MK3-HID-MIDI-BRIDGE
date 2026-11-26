#!/bin/bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
NAME="S4 Jog Diagnostic"
OUTPUT="$ROOT/dist/$NAME"
ARCHIVE="$ROOT/dist/$NAME-arm64.zip"
BINARY="s4mk3-jog-diagnostic"

fail() {
    printf 'build-jog-diagnostic: %s\n' "$*" >&2
    exit 1
}

[[ "$(uname -s)" == Darwin && "$(uname -m)" == arm64 ]] ||
    fail "requires an Apple Silicon Mac"
[[ ! -e "$OUTPUT" && ! -e "$ARCHIVE" ]] ||
    fail "output already exists; preserve the previous diagnostic suite first"

# Rust 1.88/Apple SDK 27 cannot load stripped proc-macro dylibs on this host.
CARGO_PROFILE_RELEASE_STRIP=none MACOSX_DEPLOYMENT_TARGET=12.0 cargo build --locked --release \
    --manifest-path "$ROOT/Cargo.toml" --bin "$BINARY"
otool -L "$ROOT/target/release/$BINARY" |
    awk 'NR > 1 && $1 ~ /^\// && $1 !~ /^\/System\/Library\// && $1 !~ /^\/usr\/lib\// {
        print "non-system dependency: " $1 > "/dev/stderr"; bad = 1
    } END { exit bad }' || fail "diagnostic links a machine-local library"

mkdir -p "$OUTPUT"
cp "$ROOT/target/release/$BINARY" "$OUTPUT/$BINARY"
cp "$ROOT/docs/research/jog/jog-tension-diagnostic.md" "$OUTPUT/README.md"
cp "$ROOT/examples/jog-diagnostic-replay.jsonl" "$OUTPUT/jog-diagnostic-replay.jsonl"
cp "$ROOT/examples/jog-diagnostic-fader-replay.jsonl" "$OUTPUT/jog-diagnostic-fader-replay.jsonl"
cp "$ROOT/examples/jog-diagnostic-tension-replay.jsonl" "$OUTPUT/jog-diagnostic-tension-replay.jsonl"
cp "$ROOT/examples/jog-diagnostic-continuity-replay.jsonl" "$OUTPUT/jog-diagnostic-continuity-replay.jsonl"
cp "$ROOT/examples/jog-diagnostic-damped-replay.jsonl" "$OUTPUT/jog-diagnostic-damped-replay.jsonl"
cp "$ROOT/examples/jog-diagnostic-assist-replay.jsonl" "$OUTPUT/jog-diagnostic-assist-replay.jsonl"
cp "$ROOT/examples/jog-diagnostic-wheels-replay.jsonl" "$OUTPUT/jog-diagnostic-wheels-replay.jsonl"
cp "$ROOT/docs/research/jog/jog-fader-braking-protocol.md" "$OUTPUT/jog-fader-braking-protocol.md"
codesign --force --sign - "$OUTPUT/$BINARY"
codesign --verify --strict "$OUTPUT/$BINARY"
ditto -c -k --sequesterRsrc --keepParent "$OUTPUT" "$ARCHIVE"
printf 'Created %s\nCreated %s\n' "$OUTPUT" "$ARCHIVE"
