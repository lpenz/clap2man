// Copyright (C) 2026 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

//! Functions that get use data from [`clap::Command`] to fill
//! [`man::Manual`]

use clap::Command;

use crate::{Error, Result};

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
/// This function also adds the default `-h`, `--help`, and `-V`, `--version` flags.
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

    for a in cmd.get_opts() {
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

        let mut flag = man::Flag::new();
        if let Some(short) = a.get_short() {
            flag = flag.short(&format!("-{}", short));
        }
        if let Some(long) = a.get_long() {
            flag = flag.long(&format!("--{}", long));
        }
        if let Some(help) = a.get_help() {
            flag = flag.help(&format!("{}", help));
        }
        manpage = manpage.flag(flag);
    }

    Ok(manpage
        .flag(
            man::Flag::new()
                .short("-h")
                .long("--help")
                .help("Print help (see a summary with '-h')"),
        )
        .flag(
            man::Flag::new()
                .short("-V")
                .long("--version")
                .help("Print version"),
        ))
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
        if !help.is_empty() {
            arguments_found = true;
            arguments_section = arguments_section.paragraph(&format!("**{}**: {}", id, help));
        }
    }

    Ok(if arguments_found {
        manpage.custom(arguments_section)
    } else {
        manpage
    })
}

/// Add the subcommands to a "SUBCOMMANDS" section.
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
        subcommands_section = subcommands_section.paragraph(&format!("**{}**: {}", name, about));
    }
    Ok(if subcommands_found {
        manpage.custom(subcommands_section)
    } else {
        manpage
    })
}
