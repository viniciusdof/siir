## Downloads

| File | Platform |
|---|---|
| `siir-@VERSION@-macos-universal.dmg` | macOS 15 or later, Apple silicon and Intel |
| `siir-@VERSION@-windows-x86_64.exe` | Windows, x86_64 |
| `siir-@VERSION@-windows-aarch64.exe` | Windows on ARM |
| `siir-@VERSION@-linux-x86_64.AppImage` | Linux, x86_64 |
| `siir-@VERSION@-linux-aarch64.AppImage` | Linux, ARM64 |

`SHA256SUMS` holds the checksum of every file.

## First launch

The builds are not code-signed or notarized, so each system warns once:

- **macOS**: right-click `SIIR.app`, choose **Open**, then **Open** again. If the
  download was quarantined, `xattr -d com.apple.quarantine /Applications/SIIR.app`
  clears it.
- **Windows**: SmartScreen says "Windows protected your PC"; choose **More info**
  and **Run anyway**.
- **Linux**: `chmod +x siir-@VERSION@-linux-x86_64.AppImage` and run it. Without
  FUSE, run it with `--appimage-extract-and-run`.
