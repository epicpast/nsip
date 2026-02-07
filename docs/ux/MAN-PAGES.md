# Man Pages Generation

## Overview

Generate Unix manual pages from CLI definitions using [clap_mangen](https://docs.rs/clap_mangen).

## Setup

### Add Dependencies

```toml
[dependencies]
clap = { version = "4.5", features = ["derive"] }

[build-dependencies]
clap = { version = "4.5", features = ["derive"] }
clap_mangen = "0.2"
```

### Build Script

**build.rs:**

```rust
use clap::CommandFactory;
use clap_mangen::Man;
use std::fs;
use std::path::PathBuf;

include!("src/cli.rs");

fn main() {
    let out_dir = PathBuf::from(std::env::var_os("OUT_DIR").unwrap());
    let man_dir = out_dir.join("man");
    fs::create_dir_all(&man_dir).unwrap();

    let cmd = Cli::command();
    let man = Man::new(cmd);
    let mut buffer = Vec::new();
    man.render(&mut buffer).unwrap();

    fs::write(man_dir.join("rust-template.1"), buffer).unwrap();

    println!("cargo:rerun-if-changed=src/cli.rs");
}
```

### CLI Definition

**src/cli.rs:**

```rust
use clap::Parser;

/// Modern Rust project template with production-ready tooling
///
/// This tool provides a comprehensive starting point for Rust projects,
/// including CI/CD workflows, security scanning, and multi-platform support.
#[derive(Parser, Debug)]
#[command(name = "rust-template")]
#[command(author = "Your Name <email@example.com>")]
#[command(version)]
#[command(about, long_about = None)]
pub struct Cli {
    /// Path to configuration file
    ///
    /// Specifies a custom configuration file location.
    /// If not provided, defaults to ./config.toml
    #[arg(short, long, value_name = "FILE")]
    pub config: Option<String>,

    /// Enable verbose output
    ///
    /// Increases verbosity of logging output.
    /// Can be specified multiple times for more verbosity.
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,

    /// Quiet mode (suppress output)
    #[arg(short, long, conflicts_with = "verbose")]
    pub quiet: bool,
}
```

## Installation

### System-Wide

```bash
# Build project
cargo build --release

# Copy man page
sudo cp target/release/build/rust-template-*/out/man/rust-template.1 \
     /usr/local/share/man/man1/

# Update man database
sudo mandb
```

### User Installation

```bash
# Create user man directory
mkdir -p ~/.local/share/man/man1

# Copy man page
cp target/release/build/rust-template-*/out/man/rust-template.1 \
   ~/.local/share/man/man1/

# Add to MANPATH in ~/.bashrc or ~/.zshrc
export MANPATH="$HOME/.local/share/man:$MANPATH"

# Update database
mandb ~/.local/share/man
```

### View Man Page

```bash
man rust-template
```

## Package Integration

### Debian Package

**Cargo.toml:**

```toml
[package.metadata.deb]
assets = [
    ["target/release/rust-template", "usr/bin/", "755"],
    ["target/release/build/rust-template-*/out/man/rust-template.1", "usr/share/man/man1/", "644"],
]
```

### RPM Package

**Cargo.toml:**

```toml
[package.metadata.generate-rpm]
assets = [
    { source = "target/release/rust-template", dest = "/usr/bin/", mode = "755" },
    { source = "target/release/build/rust-template-*/out/man/rust-template.1", dest = "/usr/share/man/man1/", mode = "644" },
]
```

### Homebrew Formula

```ruby
def install
  system "cargo", "install", *std_cargo_args

  # Install man page
  man1.install "target/release/build/rust-template-*/out/man/rust-template.1"
end
```

## Advanced Features

### Multiple Sections

```rust
// build.rs
use clap_mangen::Man;

fn main() {
    let cmd = Cli::command();

    // Section 1: User commands
    let man1 = Man::new(cmd.clone()).section("1");
    fs::write("man/rust-template.1", man1.render()).unwrap();

    // Section 5: File formats (config)
    let man5 = Man::new(cmd.clone())
        .section("5")
        .title("rust-template.conf");
    fs::write("man/rust-template.conf.5", man5.render()).unwrap();
}
```

### Subcommand Man Pages

```rust
use clap::{Parser, Subcommand};

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new project
    Init { /* ... */ },
    /// Build the project
    Build { /* ... */ },
}

// build.rs
fn main() {
    let cmd = Cli::command();

    // Main command
    let man = Man::new(cmd.clone());
    fs::write("man/rust-template.1", man.render()).unwrap();

    // Subcommands
    for subcmd in cmd.get_subcommands() {
        let name = format!("rust-template-{}", subcmd.get_name());
        let man = Man::new(subcmd.clone()).title(&name);
        fs::write(format!("man/{}.1", name), man.render()).unwrap();
    }
}
```

**Results in:**
- `rust-template.1` - Main command
- `rust-template-init.1` - Init subcommand
- `rust-template-build.1` - Build subcommand

### Custom Sections

```rust
use clap_mangen::roff::{Roff, roman};

let mut man = Man::new(cmd);

// Add EXAMPLES section
let examples = vec![
    roman("Basic usage:"),
    roman(""),
    roman("    rust-template --config myconfig.toml"),
    roman(""),
    roman("Verbose mode:"),
    roman(""),
    roman("    rust-template -vvv"),
];

man.push_examples(&examples);
```

## Man Page Sections

### Standard Sections

1. **NAME** - Command name and one-line description
2. **SYNOPSIS** - Command syntax
3. **DESCRIPTION** - Detailed description
4. **OPTIONS** - Command-line options
5. **EXAMPLES** - Usage examples
6. **AUTHORS** - Author information
7. **SEE ALSO** - Related commands
8. **BUGS** - Bug reporting information

### Customization

```rust
/// # Examples
///
/// Basic usage:
///     rust-template --config config.toml
///
/// Verbose mode:
///     rust-template -vvv
///
/// # See Also
///
/// Related documentation at https://docs.rs/rust-template
///
/// # Bugs
///
/// Report bugs at https://github.com/user/rust-template/issues
#[derive(Parser)]
#[command(after_help = "EXAMPLES:\n    rust-template --config config.toml\n\nSEE ALSO:\n    https://docs.rs/rust-template")]
pub struct Cli {
    // ...
}
```

## Formatting

### Emphasis

```rust
/// Enable **bold text** or *italic text* in descriptions
///
/// Use `code` for inline code
#[arg(long)]
pub option: bool,
```

### Lists

```rust
/// Multiple options:
///
/// - Option 1: Description
/// - Option 2: Description
/// - Option 3: Description
#[arg(long)]
pub option: String,
```

### Code Blocks

```rust
/// Example usage:
///
///     rust-template --config config.toml
///     rust-template --verbose
#[arg(long)]
pub option: bool,
```

## Testing

### Verify Generation

```bash
# Build
cargo build

# Find generated man page
find target -name "*.1"

# View
man target/release/build/rust-template-*/out/man/rust-template.1
```

### Lint Man Page

```bash
# Install groff
sudo apt install groff  # Debian/Ubuntu
brew install groff      # macOS

# Check for errors
groff -man -Tutf8 rust-template.1
```

### Automated Testing

```rust
#[test]
fn verify_man_page() {
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let man_file = format!("{}/man/rust-template.1", out_dir);
    assert!(std::path::Path::new(&man_file).exists());
}
```

## Viewing Man Pages

### Local Development

```bash
# View directly
man target/release/build/rust-template-*/out/man/rust-template.1

# Or add to MANPATH temporarily
export MANPATH="$PWD/target/release/build/rust-template-*/out/man:$MANPATH"
man rust-template
```

### HTML Generation

```bash
# Convert to HTML
groff -man -Thtml rust-template.1 > rust-template.html

# Or use pandoc
pandoc rust-template.1 -o rust-template.html
```

### PDF Generation

```bash
# Convert to PDF
groff -man -Tpdf rust-template.1 > rust-template.pdf

# Or via PostScript
groff -man -Tps rust-template.1 | ps2pdf - rust-template.pdf
```

## Best Practices

1. **Write detailed descriptions** - Users rely on man pages
2. **Include examples** - Show real usage patterns
3. **Document all options** - Every flag deserves explanation
4. **Test rendering** - View generated pages before release
5. **Update with code** - Keep docs in sync with CLI
6. **Version appropriately** - Man pages versioned with package
7. **Cross-reference** - Link related commands in SEE ALSO

## Troubleshooting

### Man Page Not Found

```bash
# Check installation
man -w rust-template

# Verify MANPATH
echo $MANPATH

# Rebuild man database
sudo mandb
```

### Formatting Issues

```bash
# Check for groff errors
groff -man -Tutf8 -ww rust-template.1

# Validate
man --warnings rust-template
```

### Build Failures

```bash
# Clean build
cargo clean
cargo build

# Check build.rs output
cargo build -vv 2>&1 | grep "build script"
```

## Links

- [clap_mangen Documentation](https://docs.rs/clap_mangen/)
- [Man Page Format](https://man7.org/linux/man-pages/man7/groff_man.7.html)
- [Linux Man Page Conventions](https://www.kernel.org/doc/man-pages/)
- [GNU Troff Manual](https://www.gnu.org/software/groff/manual/)
