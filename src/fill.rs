// Copyright (C) 2026 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

//! Functions that get use data from [`clap::Command`] to fill
//! [`man::Manual`]

use clap::Command;

/// Fills the "about" section.
pub fn fill_about(cmd: &Command, manpage: man::Manual) -> man::Manual {
    manpage.about(
        cmd.get_about()
            .map(|s| format!("{}", s))
            .unwrap_or_default(),
    )
}

/// Fills the "description" section with the long_about.
pub fn fill_description(cmd: &Command, manpage: man::Manual) -> man::Manual {
    manpage.description(
        cmd.get_long_about()
            .map(|s| format!("{}", s))
            .unwrap_or_default(),
    )
}

/// Fills the "author".
pub fn fill_author(cmd: &Command, manpage: man::Manual) -> man::Manual {
    manpage.author(man::Author::new(cmd.get_author().unwrap_or_default()))
}

/// Fills the "flags" section with all the options.
pub fn fill_flags(cmd: &Command, manpage: man::Manual) -> man::Manual {
    cmd.get_opts()
        .fold(manpage, |manpage, a| {
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
            manpage.flag(flag)
        })
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
        )
}

/// Add the positional arguments.
pub fn fill_positionals(cmd: &Command, mut manpage: man::Manual) -> man::Manual {
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

    if arguments_found {
        manpage.custom(arguments_section)
    } else {
        manpage
    }
}

/// Add the subcommands to a "SUBCOMMANDS" section.
pub fn fill_subcommands(cmd: &Command, manpage: man::Manual) -> man::Manual {
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
    if subcommands_found {
        manpage.custom(subcommands_section)
    } else {
        manpage
    }
}
