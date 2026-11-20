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

# Regenerate the committed app icons from packaging/icons/siir.svg (macOS only).
icons:
    #!/usr/bin/env bash
    set -euo pipefail
    if [ "$(uname)" != "Darwin" ]; then
        echo "just icons needs macOS (sips and iconutil); the generated icons are committed" >&2
        exit 1
    fi
    cd packaging/icons
    tmp="$(mktemp -d)"
    trap 'rm -rf "$tmp"' EXIT
    sips -s format png -z 1024 1024 siir.svg --out "$tmp/siir-1024.png" >/dev/null
    mkdir "$tmp/siir.iconset"
    for size in 16 32 128 256 512; do
        sips -z "$size" "$size" "$tmp/siir-1024.png" --out "$tmp/siir.iconset/icon_${size}x${size}.png" >/dev/null
        double=$((size * 2))
        sips -z "$double" "$double" "$tmp/siir-1024.png" --out "$tmp/siir.iconset/icon_${size}x${size}@2x.png" >/dev/null
    done
    iconutil -c icns "$tmp/siir.iconset" -o siir.icns
    sips -z 512 512 "$tmp/siir-1024.png" --out siir.png >/dev/null
    sips -s format ico -z 256 256 "$tmp/siir-1024.png" --out siir.ico >/dev/null

# linuxdeploy release that `just bundle-linux` downloads.
linuxdeploy_ref := "1-alpha-20251107-1"

[private]
version:
    @cargo pkgid -p siir-gui | sed 's/.*[#@]//'

# macOS: a universal SIIR.app in a .dmg under target/bundle/macos.
bundle-macos:
    #!/usr/bin/env bash
    set -euo pipefail
    if [ "$(uname)" != "Darwin" ]; then
        echo "just bundle-macos needs macOS (lipo, codesign, hdiutil)" >&2
        exit 1
    fi
    version="$(just version)"
    export MACOSX_DEPLOYMENT_TARGET=15.0
    rustup target add aarch64-apple-darwin x86_64-apple-darwin
    for target in aarch64-apple-darwin x86_64-apple-darwin; do
        cargo build --release --locked -p siir-gui --target "$target"
    done
    out=target/bundle/macos
    rm -rf "$out"
    app="$out/SIIR.app"
    mkdir -p "$app/Contents/MacOS" "$app/Contents/Resources"
    lipo -create -output "$app/Contents/MacOS/siir-gui" \
        target/aarch64-apple-darwin/release/siir-gui target/x86_64-apple-darwin/release/siir-gui
    sed "s/@VERSION@/$version/g" packaging/macos/Info.plist > "$app/Contents/Info.plist"
    cp packaging/icons/siir.icns "$app/Contents/Resources/siir.icns"
    codesign --force --deep --sign - "$app"
    mkdir "$out/dmg"
    cp -R "$app" "$out/dmg/"
    ln -s /Applications "$out/dmg/Applications"
    hdiutil create -volname SIIR -srcfolder "$out/dmg" -ov -format UDZO "$out/siir-$version-macos-universal.dmg"
    rm -rf "$out/dmg"

# Windows: x86_64 and aarch64 .exe files under target/bundle/windows.
bundle-windows:
    #!/usr/bin/env bash
    set -euo pipefail
    if [ "{{os()}}" != "windows" ]; then
        echo "just bundle-windows needs Windows: gpui-pre-windows compiles its shaders with fxc.exe" >&2
        exit 1
    fi
    version="$(just version)"
    export RUSTFLAGS="-C target-feature=+crt-static"
    rustup target add x86_64-pc-windows-msvc aarch64-pc-windows-msvc
    out=target/bundle/windows
    rm -rf "$out"
    mkdir -p "$out"
    for arch in x86_64 aarch64; do
        cargo build --release --locked -p siir-gui --target "$arch-pc-windows-msvc"
        cp "target/$arch-pc-windows-msvc/release/siir-gui.exe" "$out/siir-$version-windows-$arch.exe"
    done

# Linux: an AppImage for the host architecture under target/bundle/linux.
bundle-linux:
    #!/usr/bin/env bash
    set -euo pipefail
    if [ "{{os()}}" != "linux" ]; then
        echo "just bundle-linux needs Linux" >&2
        exit 1
    fi
    version="$(just version)"
    arch="$(uname -m)"
    cargo build --release --locked -p siir-gui
    tool="target/linuxdeploy/{{linuxdeploy_ref}}/linuxdeploy-$arch.AppImage"
    if [ ! -x "$tool" ]; then
        mkdir -p "$(dirname "$tool")"
        curl -fsSL -o "$tool" \
            "https://github.com/linuxdeploy/linuxdeploy/releases/download/{{linuxdeploy_ref}}/linuxdeploy-$arch.AppImage"
        chmod +x "$tool"
    fi
    out=target/bundle/linux
    rm -rf "$out"
    mkdir -p "$out"
    APPIMAGE_EXTRACT_AND_RUN=1 LDAI_OUTPUT="$out/siir-$version-linux-$arch.AppImage" \
        "$tool" --appdir "$out/AppDir" \
        --executable target/release/siir-gui \
        --desktop-file packaging/linux/siir.desktop \
        --icon-file packaging/icons/siir.png \
        --output appimage
    rm -rf "$out/AppDir"

# Preview the notes of the unreleased commits. Writes nothing.
changelog:
    git cliff --unreleased --strip header

# Bump the version (git-cliff picks it unless given), update CHANGELOG.md, commit, and tag. Does not push.
release version="":
    #!/usr/bin/env bash
    set -euo pipefail
    if [ "$(git branch --show-current)" != "main" ]; then
        echo "release from main" >&2
        exit 1
    fi
    if [ -n "$(git status --porcelain)" ]; then
        echo "the working tree is not clean" >&2
        exit 1
    fi
    just ci
    version="{{version}}"
    if [ -z "$version" ]; then
        version="$(git cliff --bumped-version 2>/dev/null)"
    fi
    version="${version#v}"
    tag="v$version"
    if ! [[ "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z.]+)?$ ]]; then
        echo "not a version: $version" >&2
        exit 1
    fi
    if git rev-parse -q --verify "refs/tags/$tag" >/dev/null; then
        echo "$tag already exists" >&2
        exit 1
    fi
    perl -0pi -e 's/(\[workspace\.package\]\nversion = ")[^"]*"/${1}'"$version"'"/' Cargo.toml
    if ! grep -q "^version = \"$version\"$" Cargo.toml; then
        echo "could not write the version to Cargo.toml" >&2
        exit 1
    fi
    cargo update --workspace
    git cliff --tag "$tag" -o CHANGELOG.md
    git add Cargo.toml Cargo.lock CHANGELOG.md
    git commit -m "chore(release): $tag"
    git tag -a "$tag" -m "$tag"
    echo "Tagged $tag. Push it with: git push origin main $tag"

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
        if [ ! -f "$dir/locale.scs" ]; then
            echo "missing $dir/locale.scs" >&2
            exit 1
        fi
        mise x dotnet@10.0.100 -- target/extractor/extractor "$dir/locale.scs" --partial=/locale -q -d "target/game-data/$g"
        cargo run -q -p siir-save --example catalog -- \
            "target/game-data/$g/def" "target/game-data/$g/locale" "crates/siir-save/data/$g.json" "$g"
    done
