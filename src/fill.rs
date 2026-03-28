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
pub fn fill_positionals(cmd: &Command, manpage: man::Manual) -> man::Manual {
    cmd.get_positionals().fold(manpage, |manpage, a| {
        let id = format!("{}", a.get_id());
        let arg = man::Arg::new(&id);
        let mut flag = man::Flag::new();
        flag = flag.long(&id);
        if let Some(help) = a.get_help() {
            flag = flag.help(&format!("{}", help));
        }
        manpage.flag(flag).arg(arg)
    })
}
