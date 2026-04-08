// Copyright (C) 2026 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

#![deny(future_incompatible)]
#![deny(nonstandard_style)]
#![deny(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]
#![allow(rustdoc::private_intra_doc_links)]

//! clap2man converts a clap cli into a basic manpage that can be further customized
//!
//! # Example
//!
//! ```rust
//! use clap::{Arg, Command};
//! use clap2man::Manual;
//!
//! let cmd = Command::new("test-app")
//!     .version("1.2.3")
//!     .author("John Doe <john@doe.com>")
//!     .about("A test application for clap2man")
//!     .arg(
//!         Arg::new("verbose")
//!             .short('v')
//!             .long("verbose")
//!             .help("Enable verbose mode")
//!             .action(clap::ArgAction::SetTrue),
//!     );
//!
//! let manual = Manual::try_from(&cmd).unwrap();
//! let manpage: man::Manual = manual.into();
//! let rendered = manpage.render();
//!
//! assert!(rendered.contains("test\\-app"));
//! assert!(rendered.contains("A test application for clap2man"));
//! ```

pub mod fill;

mod wrapper;
pub use wrapper::Manual;

/// Error type for clap2man
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Command about is missing
    #[error("command about is missing")]
    MissingAbout,
    /// Command author is missing
    #[error("command author is missing")]
    MissingAuthor,
    /// Duplicate flag
    #[error("duplicate flag --{0}")]
    DuplicateFlag(String),
    /// Duplicate short flag
    #[error("duplicate short flag -{0}")]
    DuplicateShortFlag(char),
}

/// Result type for clap2man
pub type Result<T> = std::result::Result<T, Error>;
