---
diataxis_type: reference
---
# Man Pages Generation

## Overview

Generate Unix manual pages from CLI definitions using [clap_mangen](https://docs.rs/clap_mangen).

## Generating Man Pages

nsip generates its man pages at runtime through the built-in `man-pages`
subcommand — there is no `build.rs` step. The CLI is defined in
`crates/main.rs` (the source lives in `crates/`, not the standard `src/`
directory) and rendered with [clap_mangen](https://docs.rs/clap_mangen).

```bash
# Build the release binary
cargo build --release

# Write all man pages to a directory
./target/release/nsip man-pages --out-dir man

# Or stream the main page to stdout (omit --out-dir)
./target/release/nsip man-pages
```

This is the same invocation used by the release pipeline
(`.github/workflows/release.yml`), which runs
`./target/release/nsip man-pages --out-dir man` and packages the result as
`nsip-man-pages.tar.gz`.

The generated files land in the directory you pass to `--out-dir` (for example
`man/nsip.1`). The examples below use `man/` as that output directory.

## Installation

### System-Wide

```bash
# Build project
cargo build --release

# Copy man page
sudo cp man/nsip.1 \
     /usr/local/share/man/man1/

# Update man database
sudo mandb
```

### User Installation

```bash
# Create user man directory
mkdir -p ~/.local/share/man/man1

# Copy man page
cp man/nsip.1 \
   ~/.local/share/man/man1/

# Add to MANPATH in ~/.bashrc or ~/.zshrc
export MANPATH="$HOME/.local/share/man:$MANPATH"

# Update database
mandb ~/.local/share/man
```

### View Man Page

```bash
man nsip
```

## Package Integration

### Debian Package

**Cargo.toml:**

```toml
[package.metadata.deb]
assets = [
    ["target/release/nsip", "usr/bin/", "755"],
    ["man/nsip.1", "usr/share/man/man1/", "644"],
]
```

### RPM Package

**Cargo.toml:**

```toml
[package.metadata.generate-rpm]
assets = [
    { source = "target/release/nsip", dest = "/usr/bin/", mode = "755" },
    { source = "man/nsip.1", dest = "/usr/share/man/man1/", mode = "644" },
]
```

### Homebrew Formula

```ruby
def install
  system "cargo", "install", *std_cargo_args

  # Install man page
  man1.install "man/nsip.1"
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
    fs::write("man/nsip.1", man1.render()).unwrap();

    // Section 5: File formats (config)
    let man5 = Man::new(cmd.clone())
        .section("5")
        .title("nsip.conf");
    fs::write("man/nsip.conf.5", man5.render()).unwrap();
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
    fs::write("man/nsip.1", man.render()).unwrap();

    // Subcommands
    for subcmd in cmd.get_subcommands() {
        let name = format!("nsip-{}", subcmd.get_name());
        let man = Man::new(subcmd.clone()).title(&name);
        fs::write(format!("man/{}.1", name), man.render()).unwrap();
    }
}
```

**Results in:**
- `nsip.1` - Main command
- `nsip-init.1` - Init subcommand
- `nsip-build.1` - Build subcommand

### Custom Sections

```rust
use clap_mangen::roff::{Roff, roman};

let mut man = Man::new(cmd);

// Add EXAMPLES section
let examples = vec![
    roman("Basic usage:"),
    roman(""),
    roman("    nsip --config myconfig.toml"),
    roman(""),
    roman("Verbose mode:"),
    roman(""),
    roman("    nsip -vvv"),
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
///     nsip --config config.toml
///
/// Verbose mode:
///     nsip -vvv
///
/// # See Also
///
/// Related documentation at https://docs.rs/nsip
///
/// # Bugs
///
/// Report bugs at https://github.com/user/nsip/issues
#[derive(Parser)]
#[command(after_help = "EXAMPLES:\n    nsip --config config.toml\n\nSEE ALSO:\n    https://docs.rs/nsip")]
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
///     nsip --config config.toml
///     nsip --verbose
#[arg(long)]
pub option: bool,
```

## Testing

### Verify Generation

```bash
# Build and generate
cargo build --release
./target/release/nsip man-pages --out-dir man

# List generated man pages
ls man/

# View
man man/nsip.1
```

### Lint Man Page

```bash
# Install groff
sudo apt install groff  # Debian/Ubuntu
brew install groff      # macOS

# Check for errors
groff -man -Tutf8 nsip.1
```

### Automated Testing

```rust
#[test]
fn verify_man_page() {
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let man_file = format!("{}/man/nsip.1", out_dir);
    assert!(std::path::Path::new(&man_file).exists());
}
```

## Viewing Man Pages

### Local Development

```bash
# View directly
man man/nsip.1

# Or add to MANPATH temporarily
export MANPATH="$PWD/man:$MANPATH"
man nsip
```

### HTML Generation

```bash
# Convert to HTML
groff -man -Thtml nsip.1 > nsip.html

# Or use pandoc
pandoc nsip.1 -o nsip.html
```

### PDF Generation

```bash
# Convert to PDF
groff -man -Tpdf nsip.1 > nsip.pdf

# Or via PostScript
groff -man -Tps nsip.1 | ps2pdf - nsip.pdf
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
man -w nsip

# Verify MANPATH
echo $MANPATH

# Rebuild man database
sudo mandb
```

### Formatting Issues

```bash
# Check for groff errors
groff -man -Tutf8 -ww nsip.1

# Validate
man --warnings nsip
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
