// Copyright (C) 2026 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

use clap::{Arg, Command};
use clap2man::Manual;

#[test]
fn test_integration() -> Result<(), Box<dyn std::error::Error>> {
    let cmd = Command::new("test-app")
        .version("1.2.3")
        .author("John Doe <john@doe.com>")
        .about("A test application for clap2man")
        .long_about("This is a longer description of the test application. It should show up in the description section.")
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .help("Enable verbose mode")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("config")
                .short('c')
                .long("config")
                .help("The configuration file to use")
                .num_args(1),
        )
        .arg(
            Arg::new("input")
                .help("The input file")
                .required(true)
                .index(1),
        )
        .subcommand(
            Command::new("sub")
                .about("A subcommand")
                .arg(Arg::new("sub-flag").short('s').long("sub-flag")),
        );

    let manual = Manual::try_from(&cmd)?;
    let manpage: man::Manual = manual.into();
    let rendered = manpage.render();
    println!("{}", rendered);

    assert!(rendered.contains("test\\-app"));
    assert!(rendered.contains("A test application for clap2man"));
    assert!(rendered.contains("John Doe <john@doe.com>"));
    assert!(rendered.contains("Enable verbose mode"));
    assert!(rendered.contains("The configuration file to use"));
    assert!(rendered.contains("input"));
    assert!(rendered.contains("ARGUMENTS"));
    assert!(rendered.contains("The input file"));
    assert!(rendered.contains("SUBCOMMANDS"));
    assert!(rendered.contains("sub"));
    assert!(rendered.contains("A subcommand"));

    // Check if flags are correctly formatted in ROFF
    assert!(rendered.contains("\\-v"));
    assert!(rendered.contains("\\-\\-verbose"));
    assert!(rendered.contains("\\-c"));
    assert!(rendered.contains("\\-\\-config"));
    Ok(())
}

#[test]
fn test_errors() {
    let cmd = Command::new("test-app");
    let result = Manual::try_from(&cmd);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), clap2man::Error::MissingAbout));

    let cmd = Command::new("test-app").about("about");
    let result = Manual::try_from(&cmd);
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        clap2man::Error::MissingAuthor
    ));

    let cmd = Command::new("test-app")
        .about("about")
        .author("author")
        .arg(Arg::new("flag1").short('f'))
        .arg(Arg::new("flag2").short('f'));
    let result = Manual::try_from(&cmd);
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        clap2man::Error::DuplicateShortFlag('f')
    ));
}
