#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
TEMP="$(mktemp -d)"
trap 'rm -rf "$TEMP"' EXIT
mkdir -p "$TEMP/inventory/notices"
touch "$TEMP/inventory/nuget-license-manifest.json"
touch "$TEMP/engine.exe" "$TEMP/mapping" "$TEMP/screen-access.json"
cat >"$TEMP/dotnet" <<'EOF'
#!/usr/bin/env bash
if [[ "${1:-}" == "--version" ]]; then
    printf '10.0.403\n'
    exit 0
fi
printf 'RESTORE_REACHED\n' >&2
exit 99
EOF
chmod +x "$TEMP/dotnet"

expect_rejected() {
    local expected="$1"
    shift
    local output
    if output="$(DOTNET="$TEMP/dotnet" bash "$ROOT/windows/build-app.sh" \
        --output "$TEMP/output-$RANDOM" \
        --inventory "$TEMP/inventory" \
        --engine "$TEMP/engine.exe" \
        --mapping "$TEMP/mapping" \
        "$@" 2>&1)"; then
        printf 'Expected rejection containing: %s\n' "$expected" >&2
        exit 1
    fi
    [[ "$output" == *"$expected"* ]] || {
        printf 'Expected "%s" in output:\n%s\n' "$expected" "$output" >&2
        exit 1
    }
}

expect_rejected '--screen-supervisor' --screen-access "$TEMP/screen-access.json"
touch "$TEMP/supervisor.exe"
expect_rejected '--screen-worker' \
    --screen-access "$TEMP/screen-access.json" \
    --screen-supervisor "$TEMP/supervisor.exe"
touch "$TEMP/worker.exe"
if output="$(S4_BRIDGE_LICENSE_INVENTORY= DOTNET="$TEMP/dotnet" bash "$ROOT/windows/build-app.sh" \
    --output "$TEMP/output-no-inventories" \
    --engine "$TEMP/engine.exe" \
    --mapping "$TEMP/mapping" \
    --screen-access "$TEMP/screen-access.json" \
    --screen-supervisor "$TEMP/supervisor.exe" \
    --screen-worker "$TEMP/worker.exe" 2>&1)"; then
    echo 'Expected the fake dotnet restore to fail' >&2
    exit 1
else
    status=$?
fi
[[ "$status" -eq 99 && "$output" == *RESTORE_REACHED* ]] || {
    printf 'Expected preflight to reach restore, got status %s:\n%s\n' "$status" "$output" >&2
    exit 1
}
expect_rejected '--inventory' \
    --inventory "$TEMP/incomplete-engine-inventory"
expect_rejected '--screen-inventory' \
    --screen-access "$TEMP/screen-access.json" \
    --screen-supervisor "$TEMP/supervisor.exe" \
    --screen-worker "$TEMP/worker.exe" \
    --screen-inventory "$TEMP/incomplete-inventory"
expect_rejected 'require --screen-access' \
    --screen-supervisor "$TEMP/supervisor.exe"
expect_rejected 'require --screen-access' \
    --screen-inventory "$TEMP/incomplete-inventory"
printf 'PASS Windows screen package argument validation\n'
