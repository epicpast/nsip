# Shell Completions

## Overview

Generate shell completions for enhanced command-line UX using [clap_complete](https://docs.rs/clap_complete).

## Setup

### Add Dependencies

```toml
[dependencies]
clap = { version = "4.5", features = ["derive"] }
clap_complete = "4.5"
```

### Implement Completions

**src/cli.rs:**

```rust
use clap::{Parser, CommandFactory};
use clap_complete::{generate, Shell};
use std::io;

#[derive(Parser, Debug)]
#[command(name = "rust-template")]
#[command(about = "Modern Rust project template")]
#[command(version)]
pub struct Cli {
    /// Configuration file path
    #[arg(short, long, value_name = "FILE")]
    pub config: Option<String>,

    /// Verbose output
    #[arg(short, long)]
    pub verbose: bool,

    /// Generate shell completions
    #[arg(long, value_name = "SHELL")]
    pub completions: Option<Shell>,
}

impl Cli {
    pub fn generate_completions(shell: Shell) {
        let mut cmd = Self::command();
        generate(shell, &mut cmd, "rust-template", &mut io::stdout());
    }
}
```

**src/main.rs:**

```rust
use clap::Parser;
use cli::Cli;

fn main() {
    let cli = Cli::parse();

    // Handle completion generation
    if let Some(shell) = cli.completions {
        Cli::generate_completions(shell);
        return;
    }

    // Normal application logic
    run(cli);
}
```

## Installation

### Bash

```bash
# Generate completions
rust-template --completions bash > ~/.local/share/bash-completion/completions/rust-template

# Or system-wide
sudo rust-template --completions bash > /etc/bash_completion.d/rust-template

# Reload
source ~/.bashrc
```

**Test:**
```bash
rust-template --<TAB>
# Shows: --config --verbose --help --version --completions
```

### Zsh

```bash
# Generate completions
rust-template --completions zsh > ~/.zsh/completions/_rust-template

# Add to .zshrc if not already
echo 'fpath=(~/.zsh/completions $fpath)' >> ~/.zshrc
echo 'autoload -Uz compinit && compinit' >> ~/.zshrc

# Reload
source ~/.zshrc
```

**Test:**
```bash
rust-template --<TAB>
# Shows completion menu with descriptions
```

### Fish

```bash
# Generate completions
rust-template --completions fish > ~/.config/fish/completions/rust-template.fish

# Reload (automatic in most cases)
fish -c 'fish_update_completions'
```

**Test:**
```bash
rust-template --<TAB>
# Shows completions with descriptions
```

### PowerShell

```powershell
# Generate completions
rust-template --completions powershell | Out-File -FilePath $PROFILE\..\rust-template.ps1

# Add to profile
Add-Content $PROFILE '. "$PSScriptRoot\rust-template.ps1"'

# Reload
. $PROFILE
```

**Test:**
```powershell
rust-template --<TAB>
# Shows completion suggestions
```

### Elvish

```bash
# Generate completions
rust-template --completions elvish > ~/.elvish/lib/rust-template.elv

# Add to rc.elv
echo 'use rust-template' >> ~/.elvish/rc.elv
```

## Package Integration

### Homebrew

**Formula includes completions:**

```ruby
def install
  system "cargo", "install", *std_cargo_args

  # Generate completions
  bash_completion.install "completions/rust-template.bash"
  zsh_completion.install "completions/_rust-template"
  fish_completion.install "completions/rust-template.fish"
end
```

**Or generate during install:**

```ruby
def install
  system "cargo", "install", *std_cargo_args

  # Generate at install time
  generate_completions_from_executable(bin/"rust-template", "--completions")
end
```

### Debian Package

**Cargo.toml:**

```toml
[package.metadata.deb]
assets = [
    ["target/release/rust-template", "usr/bin/", "755"],
    ["completions/rust-template.bash", "usr/share/bash-completion/completions/", "644"],
    ["completions/_rust-template", "usr/share/zsh/vendor-completions/", "644"],
    ["completions/rust-template.fish", "usr/share/fish/vendor_completions.d/", "644"],
]
```

### Build Script

**build.rs:**

```rust
use clap::CommandFactory;
use clap_complete::{generate_to, Shell};
use std::env;
use std::path::PathBuf;

include!("src/cli.rs");

fn main() {
    let outdir = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let mut cmd = Cli::command();

    for shell in [Shell::Bash, Shell::Zsh, Shell::Fish, Shell::PowerShell] {
        generate_to(shell, &mut cmd, "rust-template", &outdir).unwrap();
    }

    println!("cargo:rerun-if-changed=src/cli.rs");
}
```

## Advanced Features

### Subcommands

```rust
#[derive(Parser)]
enum Commands {
    /// Initialize a new project
    Init {
        /// Project name
        name: String,
    },
    /// Build the project
    Build {
        /// Release mode
        #[arg(short, long)]
        release: bool,
    },
}

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}
```

**Completions automatically include subcommands:**
```bash
rust-template <TAB>
# Shows: init, build, help
```

### Dynamic Completions

```rust
use clap::ValueHint;

#[derive(Parser)]
struct Cli {
    /// Input file
    #[arg(value_hint = ValueHint::FilePath)]
    input: String,

    /// Output directory
    #[arg(value_hint = ValueHint::DirPath)]
    output: String,

    /// Command to run
    #[arg(value_hint = ValueHint::CommandName)]
    command: String,
}
```

**Hints enable:**
- File/directory path completion
- Command name completion
- URL completion
- Username completion

### Custom Completions

```rust
use clap::builder::PossibleValue;

#[derive(Parser)]
struct Cli {
    /// Log level
    #[arg(value_parser = ["debug", "info", "warn", "error"])]
    level: String,

    /// Or with descriptions
    #[arg(value_parser = [
        PossibleValue::new("debug").help("Detailed debug information"),
        PossibleValue::new("info").help("General information"),
        PossibleValue::new("warn").help("Warning messages"),
        PossibleValue::new("error").help("Error messages only"),
    ])]
    level_detailed: String,
}
```

## Testing Completions

### Manual Testing

```bash
# Bash
complete -p rust-template

# Zsh
which _rust-template

# Fish
complete -C rust-template
```

### Automated Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use clap_complete::generate;
    use std::io;

    #[test]
    fn verify_completions() {
        let mut cmd = Cli::command();

        for shell in [Shell::Bash, Shell::Zsh, Shell::Fish] {
            let mut buf = Vec::new();
            generate(shell, &mut cmd, "rust-template", &mut buf);
            assert!(!buf.is_empty(), "Generated empty completions for {:?}", shell);
        }
    }
}
```

## Troubleshooting

### Completions Not Working

**Bash:**
```bash
# Check if bash-completion is installed
dpkg -l bash-completion  # Debian/Ubuntu
rpm -q bash-completion   # Fedora/RHEL

# Verify completion file
cat ~/.local/share/bash-completion/completions/rust-template
```

**Zsh:**
```bash
# Check fpath
echo $fpath

# Verify compinit loaded
which compinit

# Rebuild completion cache
rm -f ~/.zcompdump && compinit
```

**Fish:**
```bash
# Check completions directory
ls ~/.config/fish/completions/

# Reload completions
fish_update_completions
```

### Wrong Completions Shown

```bash
# Clear shell completion cache

# Bash
hash -r

# Zsh
rehash

# Fish
commandline -f repaint
```

## Best Practices

1. **Generate at install time** - Use build.rs or post-install scripts
2. **Include in packages** - Add to .deb, .rpm, Homebrew formula
3. **Document installation** - Provide clear user instructions
4. **Test all shells** - Verify bash, zsh, fish work correctly
5. **Use value hints** - Improve path/file completion UX
6. **Provide subcommand help** - Add descriptions to all commands

## Links

- [clap Documentation](https://docs.rs/clap/)
- [clap_complete Documentation](https://docs.rs/clap_complete/)
- [Bash Completion Guide](https://github.com/scop/bash-completion)
- [Zsh Completion Guide](https://github.com/zsh-users/zsh-completions)
- [Fish Completion Tutorial](https://fishshell.com/docs/current/completions.html)
