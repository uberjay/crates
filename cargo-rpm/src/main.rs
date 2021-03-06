//! cargo-rpm: Cargo subcommand for creating RPM releases of Rust projects

#![crate_name = "cargo_rpm"]
#![crate_type = "bin"]
#![deny(warnings, missing_docs, unsafe_code, unused_import_braces, unused_qualifications)]
#![doc(html_root_url = "https://docs.rs/cargo-rpm/0.1.0")]

#[macro_use]
extern crate failure;
extern crate flate2;
extern crate gumdrop;
#[macro_use]
extern crate gumdrop_derive;
extern crate handlebars;
#[macro_use]
extern crate iq_cli;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate tar;
extern crate toml;

/// Support for building the release archive passed to rpmbuild
pub mod archive;

/// The `cargo rpm build` subcommand
pub mod builder;

/// Cargo.toml parser
pub mod config;

/// License format converter
pub mod license;

/// The `cargo rpm init` subcommand
pub mod init;

/// Wrapper for running the `rpmbuild` command
pub mod rpmbuild;

/// Target type autodetection for crates
pub mod target;

/// Handlebars templates (for RPM specs, etc)
pub mod templates;

/// Subdirectory of a Rust project in which we keep RPM-related configs
pub const RPM_CONFIG_DIR: &str = ".rpm";

use gumdrop::Options;
use std::env;
use std::process::exit;

use builder::BuildOpts;
use init::InitOpts;

/// Command line arguments (parsed by gumdrop)
#[derive(Debug, Options)]
enum Opts {
    #[options(help = "Build RPMs from Rust projects using Cargo")]
    Rpm(RpmOpts),
}

/// Options for the `cargo rpm` subcommand
#[derive(Debug, Options)]
enum RpmOpts {
    #[options(help = "Show help for a command")]
    Help(HelpOpts),

    #[options(help = "Initialize a Rust project with RPM support")]
    Init(InitOpts),

    #[options(help = "Build an RPM out of the current project")]
    Build(BuildOpts),
}

/// Options for the `help` command
#[derive(Debug, Default, Options)]
struct HelpOpts {
    #[options(free)]
    commands: Vec<String>,
}

/// Main entry point
fn main() {
    let args: Vec<_> = env::args().collect();

    let Opts::Rpm(rpm_opts) = Opts::parse_args_default(&args[1..]).unwrap_or_else(|e| {
        match e.to_string().as_ref() {
            // Show usage if no command name is given or if "help" is given
            "missing command name" => help(&[]),
            string => eprintln!("{}: {}", args[0], string),
        }

        exit(2);
    });

    match rpm_opts {
        RpmOpts::Help(opts) => help(opts.commands.as_slice()),
        RpmOpts::Init(init) => init.call(),
        RpmOpts::Build(build) => build.call(),
    }.unwrap_or_else(|e| {
        status_error!(e);
        exit(1)
    });

    exit(0);
}

/// Print help message
fn help(commands: &[String]) -> ! {
    if commands.len() == 1 {
        if let Some(usage) = RpmOpts::command_usage(&commands[0]) {
            println!("Usage: cargo rpm {} [OPTIONS]", commands[0]);
            println!();
            println!("{}", usage);
            exit(2);
        }
    }

    println!("Usage: cargo rpm [COMMAND] [OPTIONS]");
    println!();
    println!("Available commands:");
    println!();
    println!("{}", RpmOpts::command_list().unwrap());
    println!();

    exit(2);
}
