# il2cpp-bridge-rs (fork with wraith-rs usage)

Poke at Unity's IL2CPP runtime from Rust. Resolve classes, call methods, dump metadata, grab Unity objects — without the usual pointer soup.

Built for native plugins, injected libraries, and tooling that already lives inside a running Unity process. Not a mod loader.

## Install

```bash
cargo add --git https://github.com/reravn/il2cpp-bridge-wrs
```

## Quick look

```rust
use il2cpp_bridge_rs::{api, init};

init("GameAssembly", || {
    let player = api::cache::csharp()
        .class("PlayerController")
        .expect("class exists");

    let take_damage = player
        .method(("TakeDamage", ["System.Single"]))
        .expect("method exists");

    println!("{}::{} @ 0x{:X}", player.name, take_damage.name, take_damage.rva);
});
```

## Supported targets

iOS (`aarch64-apple-ios`) · macOS (`aarch64-apple-darwin`) · Linux (`x86_64-unknown-linux-gnu`) · Android (`aarch64-linux-android`) · Windows (`x86_64-pc-windows-msvc`)

## Docs

- [Getting Started](docs/getting-started.md)
- [Core Workflows](docs/core-workflows.md)
- [Platform Notes](docs/platform-support.md)
- [Architecture](docs/architecture.md)
- [API Map](docs/api-reference.md)

Or run `cargo doc --no-deps` for the real source of truth.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).

## Disclaimer

For educational and research use. Learn about IL2CPP, reverse engineering, and native interop — responsibly.

## License

[MIT](LICENSE)
