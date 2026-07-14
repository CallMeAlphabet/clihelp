# clihelp

The colored `--help` renderer shared by the `fast*` utils, extracted after
all of them independently reinvented the same `Row`/`Section`/`paint` help page.

## Usage

```rust
use clihelp::{ColorWhen, HelpPage, Row, Section};

let page = HelpPage::new("fasthex 0.3.0 - a very fast hex dumper")
    .usage("fasthex [options] [file]...")
    .usage("fasthex -r [options] [file] [-j <offset>]")
    .blurb("Multiple files are concatenated and treated as one stream.")
    .section(Section::with_note(
        "OUTPUT FORMAT",
        "Rule: lowercase = one-byte mode, UPPERCASE = two-byte mode.",
        vec![
            Row::new("", "(default)", "canonical hex + ASCII display"),
            Row::new("-x", "--hex", "one-byte hexadecimal display"),
        ],
    ))
    .section(Section::new(
        "LAYOUT",
        vec![Row::with_value("-W", "--width", "<N>", "bytes per row (default: 16)")],
    ));

page.print(ColorWhen::Auto);
```

`ColorWhen` also resolves the `-L/--color <auto|always|never>` flag pattern
shared by these tools:

```rust
let color = clihelp::ColorWhen::parse(&flag_value)?.resolve();
```

For output that doesn't fit the flag-table model, free-form prose sections,
or coloring numbers outside the help page entirely; `clihelp::header()` and
`clihelp::paint()` expose the same palette directly.

## Install

```toml
[dependencies]
clihelp = "0.1"
```

## License

GPL-3.0-or-later, matching the tools that use it.
