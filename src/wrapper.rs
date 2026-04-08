// Copyright (C) 2026 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

//! [`man::Manual`] wrapper

use clap::Command;

use crate::Result;
use crate::fill;

/// A wrapper over [`man::Manual`] to keep proper hygiene.
///
/// Use `Manual::try_from(&cmd)` to convert a [`clap::Command`] into a [`Manual`],
/// then use `.into()` to convert it to a [`man::Manual`].
///
/// # Example
///
/// ```rust
/// use clap::Command;
/// use clap2man::Manual;
///
/// let cmd = Command::new("test-app")
///     .about("about")
///     .author("author");
/// let manual = Manual::try_from(&cmd)?;
/// let manpage: man::Manual = manual.into();
/// let rendered = manpage.render();
/// # Ok::<(), clap2man::Error>(())
/// ```
#[derive(Debug)]
pub struct Manual(man::Manual);

/// Create a [`Manual`] based on the information contained in the
/// given [`clap::Command`]
impl TryFrom<&Command> for Manual {
    type Error = crate::Error;

    fn try_from(cmd: &Command) -> Result<Self> {
        let name = cmd
            .get_display_name()
            .unwrap_or_else(|| cmd.get_name())
            .to_owned();
        let mut manpage = man::Manual::new(&name);
        manpage = fill::fill_about(cmd, manpage)?;
        manpage = fill::fill_description(cmd, manpage)?;
        manpage = fill::fill_author(cmd, manpage)?;
        manpage = fill::fill_flags(cmd, manpage)?;
        manpage = fill::fill_positionals(cmd, manpage)?;
        manpage = fill::fill_subcommands(cmd, manpage)?;
        Ok(Manual(manpage))
    }
}

/// Extranct the [`man::Manual`] from within this crate's [`Manual`]
/// wrapper
impl From<Manual> for man::Manual {
    fn from(man: Manual) -> Self {
        man.0
    }
}
