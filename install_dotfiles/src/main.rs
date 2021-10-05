#[macro_use]
extern crate maplit;

#[macro_use]
extern crate lazy_static;

mod add;
mod doctor;
mod git;
mod init;
mod log_utils;
mod manifest;
mod pack;
mod package;
mod progress;
mod prompt;
mod pull;
mod rm;
mod script;
mod status;
mod tag;
mod utils;

use isahc::ReadResponseExt;
use clap::{App, AppSettings, Arg};
use env_logger::{Builder, Target};
use log::LevelFilter;
use log::{warn, error, info};
use manifest::Manifest;
use nix::unistd::Uid;
use std::io::Write;
use std::path::PathBuf;

/// Main
fn main() {
    // Initialize logging
    let mut builder = Builder::from_default_env();
    builder.target(Target::Stdout);
    builder.format(|buf, record| {
        writeln!(
            buf,
            "{}{}\x1b[0m",
            log_utils::get_logging_prefix(record),
            record.args()
        )
    });
    #[cfg(debug_assertions)]
    builder.filter_level(LevelFilter::Trace);
    #[cfg(not(debug_assertions))]
    builder.filter_level(LevelFilter::Info);

    // CLI Interface (RAS Syndrome)
    let mut default_greatness_dir = std::path::PathBuf::new();
    default_greatness_dir.push(home::home_dir().unwrap());
    default_greatness_dir.push(".greatness");

    let matches = App::new("Achieve Greatness!")
        .version(env!("CARGO_PKG_VERSION"))
        .author("Milo Banks (Isacc Barker) <milobanks@zincsoft.dev>")
        .about("Helps you to achieve greatness!")
        .arg(
            Arg::from("<verbose> -v, --verbose 'Enable verbose mode.'")
                .required(false)
                .takes_value(false)
        )
        .subcommand(
            App::new("init")
                .about("Initializes greatness!")
                .arg(
                    Arg::from("--force 'Force to reinitialize greatness.'")
                        .required(false)
                        .takes_value(false),
                ),
        )
        // TODO: purge (removed all traces of greatness; in short, make your computer not great)
        .subcommand(
            App::new("doctor")
                .about("Finds errors. Not that there are any, this software is great after all!")
        )
        .subcommand(
            App::new("status")
                .about("Prints the status of the configuration.")
        )
        .subcommand(
            App::new("add")
                .about("Adds (a) file(s) to the state.")
                .setting(AppSettings::TrailingVarArg)
                .arg(Arg::from("<files>... 'File(s) to add.'").required(true)),
        )
        .subcommand(
            App::new("rm")
                .about("Removes a file from the state. Does not remove the file itself.")
                .setting(AppSettings::TrailingVarArg)
                .arg(Arg::from("<files>... 'File(s) to remove.'").required(true))
        )
        .subcommand(
            App::new("pull")
                .about("Fetches and merges external states.")
                .subcommand(
                    App::new("add")
                        .about("Fetches and merges an external state.")
                        .arg(
                            Arg::from("<from> 'Where to fetch the external state.'")
                                .required(true)
                                .index(1),
                        )
                        .arg(
                            Arg::from("<only-with-tag> -t, --only-with-tag 'Only merge files with a specific tag.'")
                                .required(false)
                        )
                        .arg(
                            Arg::from("<allow-mods> -d, --allow-mods 'Allow scripts and package installation. Please do not use this argument without trusting the source.'")
                                .required(false)
                                .takes_value(false)
                       )
                        .arg(
                            Arg::from("<as-main> -m, --as-main 'Install the file, overwriting the main configuration.'")
                                .required(false)
                                .takes_value(false)
                        )
                    )
                .subcommand(
                    App::new("rm")
                        .about("Removes an external state.")
                        .arg(
                            Arg::from("<name> 'The name of the external state to remove.'")
                                .required(false)
                                .index(1)
                        )
                )
        )
        .subcommand(
            App::new("tag")
                .about("Tag(s) (a) file(s).")
                .setting(AppSettings::TrailingVarArg)
                .arg(
                    Arg::from("<tag> 'What to tag the file(s) as.'")
                        .required(true)
                        .index(1),
                )
                .arg(
                    Arg::from("<files>... 'File(s) to add.'")
                        .required(true)
                        .index(2),
                ),
        )
        .subcommand(
            App::new("git")
                .about("Git utilities. For more indepth commands, use `prompt`.")
                .setting(AppSettings::SubcommandRequired)
                .subcommand(
                    App::new("remote")
                        .about("Set a remote.")
                        .arg(
                            Arg::from("<url> 'The URL of the remote.'")
                                .index(1)
                                .required(true)
                        )
                        .arg(
                            Arg::from("<remote> 'The name of the remote.'")
                                .default_value("origin")
                                .index(2)
                                .required(false)
                        )
                )
                .subcommand(
                    App::new("add")
                        .about("Adds all files to the git repository.")
                )
                .subcommand(
                    App::new("pull")
                        .about("Pull the latest from the git repository.")
                        .arg(
                            Arg::from("<remote> -r, --remote 'Remote to push to.'")
                                .default_value("origin")
                                .required(false)
                        )
                        .arg(
                            Arg::from("<branch> -b, --branch 'Branch to push to.'")
                                .default_value("main")
                                .required(false)
                        )
                        .arg(
                            Arg::from("<allow-mods> -d, --allow-mods 'Allow scripts and package installation. Please do not use this argument without trusting the source.'")
                                .required(false)
                        )

                )
                .subcommand(
                    App::new("push")
                        .about("Push your location configuration to the remote git repository.")
                        .arg(
                            Arg::from("<remote> -r, --remote 'Remote to push to.'")
                                .default_value("origin")
                                .required(false)
                        )
                        .arg(
                            Arg::from("<branch> -b, --branch 'Branch to push to.'")
                                .default_value("main")
                                .required(false)
                        )
                        .arg(
                            Arg::from("<dont-add> -a, --dont-add 'Do not add files before pushing.'")
                                .required(false)
                                .takes_value(false)
                        )

                )
        )
        .subcommand(
            App::new("pack")
                .about("Pack all your dotfiles into a git repository.")
        )
        .subcommand(
            App::new("package")
                .about("Package utilities.")
                .subcommand(
                    App::new("jog")
                        .about("Install all packages.")
                )
                .subcommand(
                    App::new("add")
                        .about("Add a package to install.")
                        .arg(
                            Arg::from("<packages>... 'Packages to mark to install.'")
                                .required(true)
                                .index(1)
                        )
                )
                .subcommand(
                    App::new("rm")
                        .about("Removes a package from what to install.")
                        .arg(
                            Arg::from("<packages>... 'Packages to unmark to install.'")
                                .required(true)
                                .index(1)
                        )
                )
                .subcommand(
                    App::new("overload")
                        .about("Deal with package overloading.")
                        .subcommand(
                            App::new("add")
                                .about("Add an overload to a added package.")
                                .arg(
                                    Arg::from("<manager> 'The manager to overload'")
                                        .index(1)
                                        .required(true)
                                )
                                .arg(
                                    Arg::from("<original> 'The original package to overload'")
                                        .index(2)
                                        .required(true)
                                )
                                .arg(
                                    Arg::from("<overload> 'The overload itself'")
                                        .index(3)
                                        .required(true)
                                )
                        )
                        .subcommand(
                            App::new("rm")
                                .about("Remove a overload from an added package.")
                                .arg(
                                    Arg::from("<overloads>.... 'The overload(s) to remove'")
                                        .index(1)
                                        .required(true)
                                )
                        )
                )
        )
        .subcommand(
            App::new("prompt")
                .about("Change directory into the git repository, with special environment variables set for git.")
                .arg(
                    Arg::from("<no-overwite-ps1> -c, --no-overwrite-ps1 'Dont overwrite the ps1 of your shell.'")
                        .required(false)
                )
        )
        .subcommand(
            App::new("script")
                .setting(AppSettings::SubcommandRequired)
                .about("Deal with Lua scripts.")
                .subcommand(
                    App::new("assign")
                        .about("Assign a Lua script to a added file.")
                        .arg(
                            Arg::from("<file> 'The file to operate on.'")
                        )
                        .arg(
                            Arg::from("<script> 'The script to operate on.'")
                        )
                )
                .subcommand(
                    App::new("rm")
                        .setting(AppSettings::TrailingVarArg)
                        .about("Remove a scripts from a file.")
                        .arg(
                            Arg::from("<file> 'The file to operate on.'")
                                .index(1)
                                .required(true)
                        )
                        .arg(
                            Arg::from("<script> 'The script to operate on.'")
                                .index(2)
                                .required(true)
                        )
                )
                .subcommand(
                    App::new("jog")
                        .about("Run script associated with a file.")
                )
        )
        .get_matches(); // TODO: Push and pull commands?

    if matches.is_present("verbose") {
        builder.filter_level(LevelFilter::Debug);
    }

    builder.init();

    if Uid::effective().is_root() {
        eprintln!(
            "You should not be great as root, or it might track files for the
root user. The feeling might also go to your head, and being root
may just tip you over into a State of catatonia.
If you got a permision error previously, please just change the permisions
on the directory."
        );
        std::process::exit(1);
    
}
    
