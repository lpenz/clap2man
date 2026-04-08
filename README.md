[![CI](https://github.com/lpenz/clap2man/actions/workflows/ci.yml/badge.svg)](https://github.com/lpenz/clap2man/actions/workflows/ci.yml)
[![coveralls](https://coveralls.io/repos/github/lpenz/clap2man/badge.svg?branch=main)](https://coveralls.io/github/lpenz/clap2man?branch=main)
[![dependency status](https://deps.rs/repo/github/lpenz/clap2man/status.svg)](https://deps.rs/repo/github/lpenz/clap2man)
[![crates.io](https://img.shields.io/crates/v/clap2man)](https://crates.io/crates/clap2man)
[![docs.rs](https://docs.rs/clap2man/badge.svg)](https://docs.rs/clap2man)

# clap2man

Converts a clap cli into a basic manpage that can be further customized

## Usage

In your `Cargo.toml`:

```toml
[build-dependencies]
clap2man = "0.1.0"
clap = "4.6.0"
```

In your `build.rs`:

```rust
use clap::{Arg, Command};
use clap2man::Manual;
use std::env;
use std::fs;
use std::path::Path;

fn main() {
    // Usually you would import your Command definition from your main crate
    // (e.g., use my_crate::cli::build_cli;) instead of defining it here.
    let cmd = Command::new("test-app")
        .version("1.2.3")
        .author("John Doe <john@doe.com>")
        .about("A test application for clap2man")
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .help("Enable verbose mode")
                .action(clap::ArgAction::SetTrue),
        );

    let manual = Manual::try_from(&cmd).unwrap();
    let manpage: man::Manual = manual.into();
    let rendered = manpage.render();

    // Find the target directory (e.g. target/debug or target/release)
    // OUT_DIR is usually target/.../build/crate-hash/out
    let out_dir = env::var_os("OUT_DIR").ok_or("OUT_DIR not set")?;
    let target_dir = Path::new(&out_dir)
        .ancestors()
        .nth(3)
        .ok_or("Could not find target directory")?;

    let out_path = target_dir.join("test-app.1");
    fs::write(out_path, rendered)?;
    Ok(())
}
```

## Features

- Converts basic information from a `clap::Command`:
  - Name
  - About
  - Description (long_about)
  - Author
  - Flags/Options (automatically adds standard help and version flags)
  - Positional Arguments
  - Subcommands
- Returns a `man::Manual` object that can be further customized.

## Customization

Since `Manual::into()` returns a `man::Manual` from the [man](https://crates.io/crates/man) crate, you can use all its methods to further customize the manpage:

```rust
let manual = Manual::try_from(&cmd)?;
let mut manpage: man::Manual = manual.into();

// Add a custom section
manpage = manpage.custom(
    man::Section::new("extra section")
        .paragraph("This is some extra information.")
);

let rendered = manpage.render();
```

## Fine-grained control

If you don't want to convert everything, you can use the functions in the `fill` module directly:

```rust
use clap2man::fill;

let mut manpage = man::Manual::new("my-app");
manpage = fill::fill_about(&cmd, manpage).unwrap();
manpage = fill::fill_flags(&cmd, manpage).unwrap();
// Only about and flags are filled; the rest is omitted or manual.
```

## Comparison with alternatives

- **[clap_mangen](https://crates.io/crates/clap_mangen)**: The official `clap` manpage generator. It is more mature and generates complete manpages directly from `clap`. However, it doesn't return a `man::Manual` object, which makes it harder to further customize the output using the `man` crate's API if you need to add complex custom sections or change the formatting in ways not supported by `clap`'s metadata.
- **clap2man**: Designed specifically to bridge `clap` and the `man` crate. Use this if you want a basic manpage generated from your CLI but also want the full flexibility of the `man` crate to add additional documentation, examples, or custom formatting that doesn't belong in your `--help` output.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
SE](LICENSE) file for details.
