//! `clihelp` — the `--help` renderer that fasthex, fastwc, fastcp, and
//! fastcount all independently reinvented.
//!
//! Extracted verbatim in spirit (same colors, same column alignment) from
//! fasthex's `main.rs`, just parameterized so four crates can share one copy.
//!
//! ```no_run
//! use clihelp::{ColorWhen, HelpPage, Row, Section};
//!
//! let page = HelpPage::new("fasthex 0.3.0 - a very fast hex dumper")
//!     .usage("fasthex [options] [file]...")
//!     .usage("fasthex -r [options] [file] [-j <offset>]")
//!     .blurb("Multiple files are concatenated and treated as one stream.")
//!     .section(Section::with_note(
//!         "OUTPUT FORMAT",
//!         "Rule: lowercase = one-byte mode, UPPERCASE = two-byte mode.",
//!         vec![
//!             Row::new("", "(default)", "canonical hex + ASCII display"),
//!             Row::new("-x", "--hex", "one-byte hexadecimal display"),
//!         ],
//!     ))
//!     .section(Section::new(
//!         "LAYOUT",
//!         vec![Row::with_value("-W", "--width", "<N>", "bytes per row (default: 16)")],
//!     ));
//!
//! page.print(ColorWhen::Auto);
//! ```

use std::io::IsTerminal;

/// When to emit ANSI color, mirroring the `-L/--color <WHEN>` flag shared by
/// fasthex and fastcount.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ColorWhen {
    Auto,
    Always,
    Never,
}

impl ColorWhen {
    /// Resolve against whether stdout is currently a terminal.
    pub fn resolve(self) -> bool {
        match self {
            ColorWhen::Always => true,
            ColorWhen::Never => false,
            ColorWhen::Auto => std::io::stdout().is_terminal(),
        }
    }

    /// Parse the `auto | always | never` values used by every tool's
    /// `--color` flag.
    pub fn parse(s: &str) -> Result<Self, String> {
        match s {
            "auto" => Ok(ColorWhen::Auto),
            "always" => Ok(ColorWhen::Always),
            "never" => Ok(ColorWhen::Never),
            _ => Err(format!("unknown color mode: {s}")),
        }
    }
}

/// ANSI palette for a help page. Defaults match the look shared across
/// fasthex / fastwc / fastcp / fastcount; override if a tool wants to
/// diverge.
#[derive(Clone, Copy, Debug)]
pub struct Theme {
    pub header: &'static str,
    pub flag: &'static str,
    pub placeholder: &'static str,
    pub reset: &'static str,
}

impl Default for Theme {
    fn default() -> Self {
        Theme {
            header: "\x1b[1;32m",    // bold green
            flag: "\x1b[1;36m",      // bold cyan
            placeholder: "\x1b[36m", // cyan
            reset: "\x1b[0m",
        }
    }
}

impl Theme {
    /// Color `s` as a section header under this theme.
    pub fn header_text(&self, s: &str, on: bool) -> String {
        paint_with_reset(self.header, s, self.reset, on)
    }
}

fn paint_with_reset(color: &str, s: &str, reset: &str, on: bool) -> String {
    if on {
        format!("{color}{s}{reset}")
    } else {
        s.to_string()
    }
}

/// Color a standalone section header, e.g. fastcp's hand-written `NOTES`
/// block that doesn't fit the [`Row`]/[`Section`] table model. Uses the
/// default [`Theme`]; call [`Theme::header_text`] directly if you need a
/// custom palette.
pub fn header(s: &str, on: bool) -> String {
    Theme::default().header_text(s, on)
}

/// Wrap `s` in `color` (an ANSI escape prefix, e.g. [`Theme::flag`]) when
/// `on`, resetting afterward. For coloring output outside the help page
/// itself — e.g. fastcount's colored benchmark numbers, which want the
/// same bold-cyan the flag table uses but aren't rendering a `Row`.
pub fn paint(color: &str, s: &str, on: bool) -> String {
    if on {
        format!("{color}{s}{}", Theme::default().reset)
    } else {
        s.to_string()
    }
}

/// A single flag row, e.g. `-W, --width <N>   bytes per row (default: 16)`.
pub struct Row {
    pub short: &'static str,
    pub long: &'static str,
    pub placeholder: Option<&'static str>,
    pub desc: &'static str,
}

impl Row {
    /// A flag with no value, e.g. `-h, --help`. Pass `short = ""` for
    /// long-only flags like `--debug`.
    pub fn new(short: &'static str, long: &'static str, desc: &'static str) -> Self {
        Row { short, long, placeholder: None, desc }
    }

    /// A flag that takes a value, e.g. `-W, --width <N>`.
    pub fn with_value(
        short: &'static str,
        long: &'static str,
        placeholder: &'static str,
        desc: &'static str,
    ) -> Self {
        Row { short, long, placeholder: Some(placeholder), desc }
    }

    fn plain_flags(&self) -> String {
        let base = if self.short.is_empty() {
            format!("    {}", self.long)
        } else {
            format!("{}, {}", self.short, self.long)
        };
        match self.placeholder {
            Some(ph) => format!("{base} {ph}"),
            None => base,
        }
    }

    fn styled_flags(&self, theme: &Theme, on: bool) -> String {
        let lit = |s: &str| paint_with_reset(theme.flag, s, theme.reset, on);
        let base = if self.short.is_empty() {
            format!("    {}", lit(self.long))
        } else {
            format!("{}, {}", lit(self.short), lit(self.long))
        };
        match self.placeholder {
            Some(ph) => format!("{base} {}", paint_with_reset(theme.placeholder, ph, theme.reset, on)),
            None => base,
        }
    }

    fn render(&self, theme: &Theme, desc_column: usize, on: bool) -> String {
        let plain_len = self.plain_flags().len();
        let pad = desc_column.saturating_sub(plain_len);
        format!("  {}{}{}", self.styled_flags(theme, on), " ".repeat(pad), self.desc)
    }
}

/// A titled group of [`Row`]s, e.g. `LAYOUT` or `OFFSET & NAVIGATION`.
pub struct Section {
    pub title: &'static str,
    pub note: Option<&'static str>,
    pub rows: Vec<Row>,
}

impl Section {
    pub fn new(title: &'static str, rows: Vec<Row>) -> Self {
        Section { title, note: None, rows }
    }

    /// A section with an explanatory note under the header, e.g. fasthex's
    /// "Rule: lowercase = one-byte mode, UPPERCASE = two-byte mode."
    pub fn with_note(title: &'static str, note: &'static str, rows: Vec<Row>) -> Self {
        Section { title, note: Some(note), rows }
    }
}

/// A full `--help` page: a name/version line, usage line(s), an optional
/// blurb, a list of [`Section`]s, and an optional footer (e.g. fasthex's
/// size-suffix legend).
pub struct HelpPage {
    name_line: String,
    usage: Vec<String>,
    blurb: Option<String>,
    sections: Vec<Section>,
    footer: Option<String>,
    desc_column: usize,
    theme: Theme,
}

impl HelpPage {
    pub fn new(name_line: impl Into<String>) -> Self {
        HelpPage {
            name_line: name_line.into(),
            usage: Vec::new(),
            blurb: None,
            sections: Vec::new(),
            footer: None,
            desc_column: 28,
            theme: Theme::default(),
        }
    }

    pub fn usage(mut self, line: impl Into<String>) -> Self {
        self.usage.push(line.into());
        self
    }

    pub fn blurb(mut self, text: impl Into<String>) -> Self {
        self.blurb = Some(text.into());
        self
    }

    pub fn section(mut self, section: Section) -> Self {
        self.sections.push(section);
        self
    }

    pub fn footer(mut self, text: impl Into<String>) -> Self {
        self.footer = Some(text.into());
        self
    }

    /// Override the description column (default 28, matching all four
    /// tools today).
    pub fn desc_column(mut self, col: usize) -> Self {
        self.desc_column = col;
        self
    }

    pub fn theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    /// Render the page to a `String`. `color` is typically the resolved
    /// bool from [`ColorWhen::resolve`].
    pub fn render(&self, color: bool) -> String {
        let header = |s: &str| paint_with_reset(self.theme.header, s, self.theme.reset, color);
        let mut out = String::new();

        out.push_str(&self.name_line);
        out.push_str("\n\n");

        if !self.usage.is_empty() {
            out.push_str(&format!("{}\n", header("Usage:")));
            for line in &self.usage {
                out.push_str("  ");
                out.push_str(line);
                out.push('\n');
            }
            out.push('\n');
        }

        if let Some(blurb) = &self.blurb {
            out.push_str(blurb);
            out.push_str("\n\n");
        }

        for section in &self.sections {
            out.push_str(&format!("{}\n", header(section.title)));
            if let Some(note) = section.note {
                out.push_str(&format!("  {note}\n\n"));
            }
            for r in &section.rows {
                out.push_str(&r.render(&self.theme, self.desc_column, color));
                out.push('\n');
            }
            out.push('\n');
        }

        if let Some(footer) = &self.footer {
            out.push_str(footer);
            out.push('\n');
        }

        out
    }

    /// Render straight to stdout, resolving color from `when`.
    pub fn print(&self, when: ColorWhen) {
        print!("{}", self.render(when.resolve()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_render_has_no_escape_codes() {
        let page = HelpPage::new("demo 1.0")
            .usage("demo [options]")
            .section(Section::new(
                "MISC",
                vec![Row::new("-h", "--help", "show this help")],
            ));
        let out = page.render(false);
        assert!(!out.contains('\x1b'));
        assert!(out.contains("-h, --help"));
        assert!(out.contains("show this help"));
    }

    #[test]
    fn colored_render_wraps_flags() {
        let page = HelpPage::new("demo 1.0").section(Section::new(
            "MISC",
            vec![Row::new("-h", "--help", "show this help")],
        ));
        let out = page.render(true);
        assert!(out.contains('\x1b'));
    }

    #[test]
    fn color_when_parses_known_values() {
        assert_eq!(ColorWhen::parse("auto"), Ok(ColorWhen::Auto));
        assert_eq!(ColorWhen::parse("always"), Ok(ColorWhen::Always));
        assert_eq!(ColorWhen::parse("never"), Ok(ColorWhen::Never));
        assert!(ColorWhen::parse("sometimes").is_err());
    }

    #[test]
    fn long_only_row_pads_like_short_plus_long() {
        let row = Row::new("", "--debug", "indicate what acceleration is used");
        assert_eq!(row.plain_flags(), "    --debug");
    }
}
