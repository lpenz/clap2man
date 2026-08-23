// Copyright (C) 2026 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

//! Functions that get use data from [`clap::Command`] to fill
//! [`man::Manual`]

use clap::Command;

use crate::{Error, Result};

/// Return the given text formatted as bold troff.
fn bold(input: &str) -> String {
    format!(r"\fB{input}\fR")
}

/// Return the given text formatted as italic troff.
fn italic(input: &str) -> String {
    format!(r"\fI{input}\fR")
}

/// Return a roff line listing the visible possible values of the
/// given argument.
///
/// Each value is rendered in bold, followed by its help in
/// parentheses when present.  Returns an empty string when there is
/// nothing to display.
fn possible_values_line(a: &clap::Arg) -> String {
    let values: Vec<String> = a
        .get_possible_values()
        .iter()
        .filter(|v| !v.is_hide_set())
        .map(|v| match v.get_help() {
            Some(help) => format!("{} ({})", bold(v.get_name()), help),
            None => bold(v.get_name()),
        })
        .collect();
    if values.is_empty() {
        return String::new();
    }
    format!(".br\nPossible values: {}", values.join(", "))
}

/// Fills the "about" section.
///
/// # Example
///
/// ```rust
/// use clap::Command;
/// use clap2man::fill;
///
/// let cmd = Command::new("test").about("my test app");
/// let mut manpage = man::Manual::new("test");
/// manpage = fill::fill_about(&cmd, manpage).unwrap();
/// assert!(manpage.render().contains("my test app"));
/// ```
pub fn fill_about(cmd: &Command, manpage: man::Manual) -> Result<man::Manual> {
    Ok(manpage.about(
        cmd.get_about()
            .map(|s| s.to_string())
            .ok_or(Error::MissingAbout)?,
    ))
}

/// Fills the "description" section with the long_about.
///
/// Unlike [`fill_about`] and [`fill_author`], this intentionally
/// returns an empty string when `long_about` is not set — a missing
/// description is valid and common for short CLIs.
pub fn fill_description(cmd: &Command, manpage: man::Manual) -> Result<man::Manual> {
    Ok(manpage.description(
        cmd.get_long_about()
            .map(|s| s.to_string())
            .unwrap_or_default(),
    ))
}

/// Fills the "author".
pub fn fill_author(cmd: &Command, manpage: man::Manual) -> Result<man::Manual> {
    let author = cmd
        .get_author()
        .map(|s| s.to_string())
        .ok_or(Error::MissingAuthor)?;
    Ok(manpage.author(man::Author::new(&author)))
}

/// Fills the "flags" section with all the options from the given [`Command`].
///
/// If the command doesn't already define `-h`/`--help` or `-V`/`--version`,
/// the standard defaults are added.
///
/// # Example
///
/// ```rust
/// use clap::{Arg, Command};
/// use clap2man::fill;
///
/// let cmd = Command::new("test")
///     .arg(Arg::new("verbose")
///         .short('v')
///         .long("verbose")
///         .help("Enable verbose mode")
///         .action(clap::ArgAction::SetTrue));
/// let mut manpage = man::Manual::new("test");
/// manpage = fill::fill_flags(&cmd, manpage)?;
/// let rendered = manpage.render();
/// assert!(rendered.contains("Enable verbose mode"));
/// assert!(rendered.contains("\\-v"));
/// assert!(rendered.contains("\\-\\-verbose"));
/// # Ok::<(), clap2man::Error>(())
/// ```
pub fn fill_flags(cmd: &Command, mut manpage: man::Manual) -> Result<man::Manual> {
    let mut longs = std::collections::HashSet::new();
    let mut shorts = std::collections::HashSet::new();
    let mut has_help = false;
    let mut has_version = false;

    for a in cmd.get_arguments() {
        if let Some(long) = a.get_long()
            && !longs.insert(long.to_string())
        {
            return Err(Error::DuplicateFlag(long.to_string()));
        }
        if let Some(short) = a.get_short()
            && !shorts.insert(short)
        {
            return Err(Error::DuplicateShortFlag(short));
        }

        match (a.get_short(), a.get_long()) {
            (Some('h'), _) | (_, Some("help")) => has_help = true,
            (Some('V'), _) | (_, Some("version")) => has_version = true,
            _ => {}
        }

        if cmd.get_opts().any(|o| o.get_id() == a.get_id()) {
            let mut flag = man::Flag::new();
            if let Some(short) = a.get_short() {
                flag = flag.short(&format!("-{}", short));
            }
            if let Some(long) = a.get_long() {
                flag = flag.long(&format!("--{}", long));
            }
            let mut help = a.get_help().map(|s| format!("{}", s)).unwrap_or_default();
            let pv_line = possible_values_line(a);
            if !pv_line.is_empty() {
                help.push('\n');
                help.push_str(&pv_line);
            }
            if !help.is_empty() {
                flag = flag.help(&help);
            }
            manpage = manpage.flag(flag);
        }
    }

    if !has_help {
        manpage = manpage.flag(
            man::Flag::new()
                .short("-h")
                .long("--help")
                .help("Print help (see a summary with '-h')"),
        );
    }
    if !has_version {
        manpage = manpage.flag(
            man::Flag::new()
                .short("-V")
                .long("--version")
                .help("Print version"),
        );
    }

    Ok(manpage)
}

/// Add the positional arguments.
pub fn fill_positionals(cmd: &Command, mut manpage: man::Manual) -> Result<man::Manual> {
    let mut arguments_section = man::Section::new("arguments");
    let mut arguments_found = false;

    for a in cmd.get_positionals() {
        let id = format!("{}", a.get_id());
        let arg = man::Arg::new(&id);
        manpage = manpage.arg(arg);

        let help = a.get_help().map(|s| format!("{}", s)).unwrap_or_default();
        let pv_line = possible_values_line(a);
        if !help.is_empty() || !pv_line.is_empty() {
            arguments_found = true;
            let mut entry = format!(".TP\n{}", bold(&id));
            if !help.is_empty() {
                entry.push('\n');
                entry.push_str(&help);
            }
            if !pv_line.is_empty() {
                entry.push('\n');
                entry.push_str(&pv_line);
            }
            arguments_section = arguments_section.paragraph(&entry);
        }
    }

    Ok(if arguments_found {
        manpage.custom(arguments_section)
    } else {
        manpage
    })
}

/// Add the subcommands to a "SUBCOMMANDS" section.
///
/// Each visible subcommand is rendered as a tagged list (`.TP`) entry
/// with the name in bold, followed by the about text and, when
/// present, the subcommand's visible flags and positional arguments.
///
/// # Example
///
/// ```rust
/// use clap::Command;
/// use clap2man::fill;
///
/// let cmd = Command::new("test").subcommand(Command::new("run").about("Run things"));
/// let mut manpage = man::Manual::new("test");
/// manpage = fill::fill_subcommands(&cmd, manpage).unwrap();
/// let rendered = manpage.render();
/// assert!(rendered.contains(".TP\n\\fBrun\\fR\nRun things"));
/// ```
pub fn fill_subcommands(cmd: &Command, manpage: man::Manual) -> Result<man::Manual> {
    let mut subcommands_section = man::Section::new("subcommands");
    let mut subcommands_found = false;
    for sub in cmd.get_subcommands() {
        if sub.is_hide_set() {
            continue;
        }
        subcommands_found = true;
        let name = sub.get_name();
        let about = sub
            .get_about()
            .map(|s| format!("{}", s))
            .unwrap_or_default();
        let mut entry = format!(".TP\n{}", bold(name));
        if !about.is_empty() {
            entry.push('\n');
            entry.push_str(&about);
        }

        let flags: Vec<String> = sub
            .get_opts()
            .filter(|a| !a.is_hide_set())
            .flat_map(|a| {
                let mut parts = Vec::new();
                if let Some(short) = a.get_short() {
                    parts.push(bold(&format!("-{}", short)));
                }
                if let Some(long) = a.get_long() {
                    parts.push(bold(&format!("--{}", long)));
                }
                parts
            })
            .collect();
        if !flags.is_empty() {
            entry.push_str("\n.br\nFlags: ");
            entry.push_str(&flags.join(", "));
        }

        let args: Vec<String> = sub
            .get_positionals()
            .filter(|a| !a.is_hide_set())
            .map(|a| italic(&format!("{}", a.get_id())))
            .collect();
        if !args.is_empty() {
            entry.push_str("\n.br\nArguments: ");
            entry.push_str(&args.join(", "));
        }

        for a in sub.get_positionals().filter(|a| !a.is_hide_set()) {
            let pv_line = possible_values_line(a);
            if !pv_line.is_empty() {
                entry.push('\n');
                entry.push_str(&pv_line);
            }
        }

        subcommands_section = subcommands_section.paragraph(&entry);
    }
    Ok(if subcommands_found {
        manpage.custom(subcommands_section)
    } else {
        manpage
    })
}
