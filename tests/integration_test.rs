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
    assert!(rendered.contains("\\fBinput\\fR\nThe input file"));
    assert!(rendered.contains("The input file"));
    assert!(rendered.contains("SUBCOMMANDS"));
    assert!(rendered.contains(".SS sub\nA subcommand"));

    // Check if flags are correctly formatted in ROFF
    assert!(rendered.contains("\\-v"));
    assert!(rendered.contains("\\-\\-verbose"));
    assert!(rendered.contains("\\-c"));
    assert!(rendered.contains("\\-\\-config"));
    Ok(())
}

#[test]
fn test_custom_help_version_flags() -> Result<(), Box<dyn std::error::Error>> {
    let cmd = Command::new("test-app")
        .about("about")
        .author("author")
        .disable_help_flag(true)
        .disable_version_flag(true)
        .arg(Arg::new("help").short('?').long("help").help("Show help"))
        .arg(
            Arg::new("ver")
                .short('V')
                .long("version")
                .help("Show version"),
        );

    let manual = Manual::try_from(&cmd)?;
    let manpage: man::Manual = manual.into();
    let rendered = manpage.render();

    // The custom flags should appear, not duplicates
    assert!(rendered.contains("\\-?"));
    assert!(rendered.contains("Show help"));
    assert!(rendered.contains("\\-\\-version"));
    assert!(rendered.contains("Show version"));

    Ok(())
}

#[test]
fn test_subcommand_flags_and_args() -> Result<(), Box<dyn std::error::Error>> {
    let cmd = Command::new("test-app")
        .about("about")
        .author("author")
        .subcommand(
            Command::new("sub")
                .about("A subcommand")
                .arg(
                    Arg::new("output")
                        .short('o')
                        .long("output")
                        .help("Output file"),
                )
                .arg(Arg::new("target").help("Target name").index(1)),
        );

    let manual = Manual::try_from(&cmd)?;
    let manpage: man::Manual = manual.into();
    let rendered = manpage.render();

    assert!(rendered.contains("SUBCOMMANDS"));
    // Each subcommand gets its own subsection, including a synopsis
    // line for the subcommand
    assert!(rendered.contains(".SS sub\nA subcommand"));
    assert!(
        rendered.contains(
            "Usage: \\fBtest\\-app\\fR \\fBsub\\fR [\\-\\-output OUTPUT] [\\fItarget\\fR]"
        )
    );
    // Subcommand flags and args are rendered as tagged lists inside
    // the subsection
    assert!(rendered.contains("\\fB\\-o\\fR, \\fB\\-\\-output\\fR\nOutput file"));
    assert!(rendered.contains("\\fBtarget\\fR\nTarget name"));

    Ok(())
}

#[test]
fn test_synopsis_subcommands() -> Result<(), Box<dyn std::error::Error>> {
    let cmd = Command::new("test-app")
        .about("about")
        .author("author")
        .subcommand(
            Command::new("check").arg(
                Arg::new("strict")
                    .short('s')
                    .long("strict")
                    .action(clap::ArgAction::SetTrue),
            ),
        )
        .subcommand(
            Command::new("convert")
                .arg(
                    Arg::new("output")
                        .short('o')
                        .long("output")
                        .value_name("FILE"),
                )
                .arg(Arg::new("format").index(1).required(false)),
        );

    let manual = Manual::try_from(&cmd)?;
    let manpage: man::Manual = manual.into();
    let rendered = manpage.render();

    // Each subcommand gets its own line in the SYNOPSIS section, with
    // its options and arguments
    assert!(rendered.contains(".br\n\\fBtest\\-app\\fR \\fBcheck\\fR [\\-\\-strict]"));
    assert!(
        rendered.contains(
            ".br\n\\fBtest\\-app\\fR \\fBconvert\\fR [\\-\\-output FILE] [\\fIformat\\fR]"
        )
    );

    Ok(())
}

#[test]
fn test_possible_values() -> Result<(), Box<dyn std::error::Error>> {
    use clap::ValueEnum;

    #[derive(Clone, ValueEnum)]
    enum Color {
        /// Bright red
        Red,
        Green,
    }

    #[derive(Clone, ValueEnum)]
    enum Shade {
        /// Very light
        Light,
        Dark,
    }

    let cmd = Command::new("test-app")
        .about("about")
        .author("author")
        .arg(
            Arg::new("color")
                .short('C')
                .long("color")
                .help("Color to use")
                .value_parser(clap::builder::EnumValueParser::<Color>::new()),
        )
        .subcommand(
            Command::new("paint").about("Paint things").arg(
                Arg::new("shade")
                    .help("Shade of the color")
                    .index(1)
                    .value_parser(clap::builder::EnumValueParser::<Shade>::new()),
            ),
        );

    let manual = Manual::try_from(&cmd)?;
    let manpage: man::Manual = manual.into();
    let rendered = manpage.render();

    // Possible values are rendered for top-level flags...
    assert!(rendered.contains("Possible values: \\fBred\\fR (Bright red), \\fBgreen\\fR"));
    // ...and inside the subcommand's subsection
    assert!(rendered.contains(".SS paint\nPaint things"));
    assert!(rendered.contains("Possible values: \\fBlight\\fR (Very light), \\fBdark\\fR"));

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

    let cmd = Command::new("test-app")
        .about("about")
        .author("author")
        .arg(Arg::new("flag1").long("flag"))
        .arg(Arg::new("flag2").long("flag"));
    let result = Manual::try_from(&cmd);
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        clap2man::Error::DuplicateFlag(ref s) if s == "flag"
    ));
}

#[test]
fn test_hidden_subcommand_not_rendered() -> Result<(), Box<dyn std::error::Error>> {
    let cmd = Command::new("test-app")
        .about("about")
        .author("author")
        .subcommand(Command::new("visible").about("shown"))
        .subcommand(Command::new("hidden").about("not shown").hide(true));

    let manual = Manual::try_from(&cmd)?;
    let manpage: man::Manual = manual.into();
    let rendered = manpage.render();

    assert!(rendered.contains("visible"));
    assert!(!rendered.contains("hidden"));

    Ok(())
}

#[test]
fn test_fill_module_direct() -> Result<(), Box<dyn std::error::Error>> {
    use clap2man::fill;

    let cmd = Command::new("test")
        .about("my test app")
        .author("Test Author")
        .long_about("Long description")
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .help("Enable verbose mode")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(Arg::new("input").help("Input file").index(1));

    let mut manpage = man::Manual::new("test");
    manpage = fill::fill_about(&cmd, manpage)?;
    manpage = fill::fill_description(&cmd, manpage)?;
    manpage = fill::fill_author(&cmd, manpage)?;
    manpage = fill::fill_flags(&cmd, manpage)?;
    manpage = fill::fill_positionals(&cmd, manpage)?;
    let rendered = manpage.render();

    assert!(rendered.contains("my test app"));
    assert!(rendered.contains("Long description"));
    assert!(rendered.contains("Test Author"));
    assert!(rendered.contains("Enable verbose mode"));
    assert!(rendered.contains("Input file"));

    Ok(())
}
