# List commands.
[default]
help:
    @just --list

# Extractor commit used to generate the catalog snapshots.
extractor_ref := "1a318e6c57e6f703ba86ced8006587ce853109c9"

# Install pinned tools and this clone's git hooks.
setup:
    mise install
    hk install --mise

# Format Rust sources.
fmt:
    cargo fmt --all

# Fail if rustfmt would change a file. Does not write.
fmt-check:
    cargo fmt --all -- --check

# Clippy on every target and feature. A warning is an error.
lint:
    cargo clippy --all-targets --all-features -- -D warnings

# Unit, integration, and doc tests.
test:
    cargo test --all-targets --all-features

# Build every target and feature.
build:
    cargo build --all-targets --all-features

# Run the desktop front end.
gui *args:
    cargo run -p siir-gui -- {{args}}

# API docs, without dependencies. A warning is an error.
[env("RUSTDOCFLAGS", "-D warnings")]
doc:
    cargo doc --no-deps --all-features

# RustSec advisories and dependency policy (license, ban, source).
audit:
    cargo audit
    cargo deny check

# Fast pre-commit checks. Does not run tests or the audit.
pre-commit: fmt-check lint

# Required gate. pre-push runs this recipe and nothing else.
ci: fmt-check lint test build doc audit

# Build the archive extractor that `just catalog` runs.
extractor:
    #!/usr/bin/env bash
    set -euo pipefail
    stamp=target/extractor/.extractor-ref
    if [ -x target/extractor/extractor ] \
        && [ "$(cat "$stamp" 2>/dev/null)" = "{{extractor_ref}}" ]; then
        exit 0
    fi
    if ! mise x dotnet@10.0.100 -- dotnet --version >/dev/null 2>&1; then
        echo "dotnet 10 is required; install it with mise" >&2
        exit 1
    fi
    if [ ! -d target/extractor-src/.git ]; then
        git clone https://github.com/sk-zk/Extractor target/extractor-src
    fi
    git -C target/extractor-src fetch --all
    git -C target/extractor-src checkout --detach {{extractor_ref}}
    mise x dotnet@10.0.100 -- dotnet publish target/extractor-src/Extractor/Extractor.csproj -c Release -o target/extractor
    mkdir -p target/extractor
    printf '%s\n' "{{extractor_ref}}" > "$stamp"

# Regenerate the committed catalog snapshots from the installed games.
catalog game="all":
    #!/usr/bin/env bash
    set -euo pipefail
    just extractor
    case "{{game}}" in
        ets2|ats) games=("{{game}}") ;;
        all) games=(ets2 ats) ;;
        *) echo "usage: just catalog [ets2|ats|all]" >&2; exit 1 ;;
    esac
    shopt -s nullglob
    for g in "${games[@]}"; do
        if [ "$g" = "ets2" ]; then
            dir="${ETS2_DIR:-$HOME/Library/Application Support/Steam/steamapps/common/Euro Truck Simulator 2}"
        else
            dir="${ATS_DIR:-$HOME/Library/Application Support/Steam/steamapps/common/American Truck Simulator}"
        fi
        if [ ! -f "$dir/def.scs" ]; then
            echo "missing $dir/def.scs; set ETS2_DIR or ATS_DIR" >&2
            exit 1
        fi
        rm -rf "target/game-data/$g"
        archives=("$dir/def.scs" "$dir"/dlc_*.scs)
        mise x dotnet@10.0.100 -- target/extractor/extractor "${archives[@]}" --partial=/def -q -d "target/game-data/$g"
        cargo run -q -p siir-save --example catalog -- \
            "target/game-data/$g/def" "crates/siir-save/data/$g.json" "$g"
    done
