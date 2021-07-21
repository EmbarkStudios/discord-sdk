//! Functionality for registering an application with Discord so that Discord can
//! start it in the future eg. when the user accpets an invite to play the game
//! by another user

#[cfg_attr(target_os = "linux", path = "registration/linux.rs")]
#[cfg_attr(target_os = "windows", path = "registration/windows.rs")]
#[cfg_attr(target_os = "macos", path = "registration/mac.rs")]
mod registrar;

use crate::Error;
pub use registrar::register_app;
pub use url::Url;

#[derive(PartialEq)]
pub enum BinArg {
    /// A placeholder token that will be filled with the url that was opened
    Url,
    /// Generic argument
    Arg(String),
}

impl From<String> for BinArg {
    fn from(s: String) -> Self {
        Self::Arg(s)
    }
}

pub enum LaunchCommand {
    /// A URL
    Url(Url),
    /// A binary with optional args
    Bin {
        /// A full path or a name of a binary in PATH
        path: std::path::PathBuf,
        /// The arguments to pass
        args: Vec<BinArg>,
    },
    /// A Steam game identifier
    Steam(u32),
}

impl LaunchCommand {
    pub fn current_exe(args: Vec<BinArg>) -> Result<Self, Error> {
        let path = std::env::current_exe()
            .map_err(|e| Error::io("retrieving current executable path", e))?;

        if args.iter().filter(|a| **a == BinArg::Url).count() > 1 {
            return Err(Error::TooManyUrls);
        }

        Ok(Self::Bin { path, args })
    }
}

pub struct Application {
    /// The application's unique Discord identifier
    pub id: crate::AppId,
    /// The application name, defaults to the id if not specified
    pub name: Option<String>,
    /// The command to launch the application itself.
    pub command: LaunchCommand,
}

#[allow(unused)]
pub(crate) fn create_command(path: std::path::PathBuf, args: Vec<BinArg>, url_str: &str) -> String {
    use std::fmt::Write;

    let mut cmd = format!("\"{}\"", path.display());

    for arg in args {
        match arg {
            BinArg::Url => write!(&mut cmd, " {}", url_str),
            BinArg::Arg(s) => {
                // Only handle spaces, if there are other whitespace characters
                // well...
                if s.contains(' ') {
                    write!(&mut cmd, " \"{}\"", s)
                } else {
                    write!(&mut cmd, " {}", s)
                }
            }
        }
        .unwrap();
    }

    cmd
}
