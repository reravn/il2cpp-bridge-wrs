# Platform Support

## Supported Platforms

| Platform | Target Triple | Status |
|----------|--------------|--------|
| iOS | `aarch64-apple-ios` | Primary target |
| macOS | `aarch64-apple-darwin` | Supported |
| Linux | `x86_64-unknown-linux-gnu` | Supported |
| Android | `aarch64-linux-android` | Supported |
| Windows | `x86_64-pc-windows-msvc` | Supported |

## Platform-Specific APIs

### Symbol Resolution (`memory::info::symbol`)

| Platform | API | Crate |
|----------|-----|-------|
| macOS/iOS | `dlsym` | `libc` |
| Linux/Android | `dlsym` | `libc` |
| Windows | `GetProcAddress` | `windows-sys` |

### Image Base Address (`memory::info::image`)

| Platform | API | Crate |
|----------|-----|-------|
| macOS/iOS | `dyld` APIs | `mach2` |
| Linux/Android | `dl_iterate_phdr` | `libc` |
| Windows | `GetModuleHandleA` | `windows-sys` |

## Build Commands

```bash
# iOS (primary target)
cargo build --target aarch64-apple-ios
cargo build --target aarch64-apple-ios --release

# Host platform
cargo build
cargo build --release

# Type-check without building
cargo check
cargo check --target aarch64-apple-ios
```

## Platform Dependencies

Dependencies are conditionally compiled per platform:

```toml
# All unix platforms
[target.'cfg(unix)'.dependencies]
libc = "0.2"

# macOS and iOS only
[target.'cfg(any(target_os = "macos", target_os = "ios"))'.dependencies]
mach2 = "0.6.0"

# Windows only
[target.'cfg(windows)'.dependencies]
windows-sys = { version = "0.61.2", features = ["Win32_System_LibraryLoader"] }
```

## Release Profile

The release profile is configured for maximum performance:

```toml
[profile.release]
opt-level = 3           # Maximum optimization
lto = "fat"             # Full link-time optimization
codegen-units = 1       # Single codegen unit for better optimization
panic = "abort"         # No unwinding overhead
strip = "debuginfo"     # Strip debug info
overflow-checks = false # Disable overflow checks
```
