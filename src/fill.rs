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

/// Return the help text of the given argument, followed by its
/// possible values line when present.
fn help_with_possible_values(a: &clap::Arg) -> String {
    let mut help = a.get_help().map(|s| format!("{}", s)).unwrap_or_default();
    let pv_line = possible_values_line(a);
    if !pv_line.is_empty() {
        if !help.is_empty() {
            help.push('\n');
        }
        help.push_str(&pv_line);
    }
    help
}

/// Return a roff tagged-list (`.TP`) entry describing a single
/// option of the given command, including its help and possible
/// values.
///
/// Returns `None` when the argument has no short or long name.
fn flag_entry(a: &clap::Arg) -> Option<String> {
    let mut names = Vec::new();
    if let Some(short) = a.get_short() {
        names.push(bold(&format!("-{}", short)));
    }
    if let Some(long) = a.get_long() {
        names.push(bold(&format!("--{}", long)));
    }
    if names.is_empty() {
        return None;
    }
    let header = names.join(", ");
    let help = help_with_possible_values(a);
    Some(if help.is_empty() {
        format!(".TP\n{}", header)
    } else {
        format!(".TP\n{}\n{}", header, help)
    })
}

/// Return a roff tagged-list (`.TP`) entry describing a single
/// positional argument of the given command, including its help and
/// possible values.
///
/// Returns `None` when the argument has no help and no possible
/// values.
fn positional_entry(a: &clap::Arg) -> Option<String> {
    let help = help_with_possible_values(a);
    if help.is_empty() {
        return None;
    }
    let id = format!("{}", a.get_id());
    Some(format!(".TP\n{}\n{}", bold(&id), help))
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

/// Return the value placeholder of the given argument, used in
/// synopsis entries: either its value name or its id uppercased.
fn value_placeholder(a: &clap::Arg) -> String {
    a.get_value_names()
        .and_then(|names| names.first())
        .map(|s| s.to_string())
        .unwrap_or_else(|| a.get_id().to_string().to_uppercase())
}

/// Return the SYNOPSIS roff line for the given subcommand, listing
/// its visible options and positional arguments.
fn synopsis_line(name: &str, sub: &Command) -> String {
    let mut line = format!("\n.br\n{} {}", bold(name), bold(sub.get_name()));
    for a in sub.get_opts().filter(|a| !a.is_hide_set()) {
        let flag = match (a.get_long(), a.get_short()) {
            (Some(long), _) => format!("--{}", long),
            (_, Some(short)) => format!("-{}", short),
            (None, None) => continue,
        };
        if matches!(
            a.get_action(),
            clap::ArgAction::Set | clap::ArgAction::Append
        ) {
            line.push_str(&format!(" [{} {}]", flag, value_placeholder(a)));
        } else {
            line.push_str(&format!(" [{}]", flag));
        }
    }
    for a in sub.get_positionals().filter(|a| !a.is_hide_set()) {
        let placeholder = italic(&format!("{}", a.get_id()));
        if a.is_required_set() {
            line.push(' ');
            line.push_str(&placeholder);
        } else {
            line.push_str(&format!(" [{}]", placeholder));
        }
    }
    line
}

/// Add a SYNOPSIS line for each visible subcommand, containing its
/// options and positional arguments.
///
/// The extra lines are added as synthetic argument entries, which the
/// [man] crate renders inside the SYNOPSIS section.
///
/// # Example
///
/// ```rust
/// use clap::{Arg, Command};
/// use clap2man::fill;
///
/// let cmd = Command::new("test")
///     .about("about")
///     .subcommand(Command::new("run").arg(Arg::new("target").index(1)));
/// let mut manpage = man::Manual::new("test");
/// manpage = fill::fill_synopsis(&cmd, manpage)?;
/// let rendered = manpage.render();
/// assert!(rendered.contains(".br\n\\fBtest\\fR \\fBrun\\fR [\\fItarget\\fR]"));
/// # Ok::<(), clap2man::Error>(())
/// ```
pub fn fill_synopsis(cmd: &Command, mut manpage: man::Manual) -> Result<man::Manual> {
    let name = cmd
        .get_display_name()
        .unwrap_or_else(|| cmd.get_name())
        .to_owned();
    for sub in cmd.get_subcommands().filter(|s| !s.is_hide_set()) {
        manpage = manpage.arg(man::Arg::new(&synopsis_line(&name, sub)));
    }
    Ok(manpage)
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
            let help = help_with_possible_values(a);
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

        if let Some(entry) = positional_entry(a) {
            arguments_found = true;
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
/// Each visible subcommand is rendered as its own subsection (roff
/// `.SS`) containing the about text, followed by the subcommand's
/// visible options and positional arguments as tagged lists.
///
/// # Example
///
/// ```rust
/// use clap::{Arg, Command};
/// use clap2man::fill;
///
/// let cmd = Command::new("test").subcommand(
///     Command::new("run")
///         .about("Run things")
///         .arg(Arg::new("target").help("Target name").index(1)),
/// );
/// let mut manpage = man::Manual::new("test");
/// manpage = fill::fill_subcommands(&cmd, manpage).unwrap();
/// let rendered = manpage.render();
/// assert!(rendered.contains(".SS run\nRun things"));
/// assert!(rendered.contains("\\fBtarget\\fR\nTarget name"));
/// ```
pub fn fill_subcommands(cmd: &Command, manpage: man::Manual) -> Result<man::Manual> {
    let mut subcommands_section = man::Section::new("subcommands");
    let mut subcommands_found = false;
    for sub in cmd.get_subcommands() {
        if sub.is_hide_set() {
            continue;
        }
        subcommands_found = true;

        let mut heading = format!(".SS {}", sub.get_name());
        if let Some(about) = sub.get_about() {
            heading.push('\n');
            heading.push_str(&format!("{}", about));
        }
        subcommands_section = subcommands_section.paragraph(&heading);

        for a in sub.get_opts().filter(|a| !a.is_hide_set()) {
            if let Some(entry) = flag_entry(a) {
                subcommands_section = subcommands_section.paragraph(&entry);
            }
        }

        for a in sub.get_positionals().filter(|a| !a.is_hide_set()) {
            if let Some(entry) = positional_entry(a) {
                subcommands_section = subcommands_section.paragraph(&entry);
            }
        }
    }
    Ok(if subcommands_found {
        manpage.custom(subcommands_section)
    } else {
        manpage
    })
}
