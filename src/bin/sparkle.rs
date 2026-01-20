//! Sparkle CLI - Browser automation toolkit
//!
//! This is the command-line interface for Sparkle, providing utilities to:
//! - Install browsers (Chrome, ChromeDriver)
//! - Manage browser versions
//! - Verify installations

use clap::{Parser, Subcommand};
use sparkle::cli::{install, list, uninstall};
use std::process;

#[derive(Parser)]
#[command(name = "sparkle")]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Install browsers and drivers
    Install {
        /// Browser to install (chromium, chrome, all)
        #[arg(default_value = "chromium")]
        browser: String,

        /// Skip ChromeDriver installation (ChromeDriver is installed by default)
        #[arg(long, default_value_t = false)]
        skip_driver: bool,

        /// Force reinstall even if already installed
        #[arg(long)]
        force: bool,
    },

    /// List installed browsers
    List,

    /// Uninstall browsers and drivers
    Uninstall {
        /// Browser to uninstall (chromium, chrome, all)
        browser: String,
    },
}

#[tokio::main]
async fn main() {
    // Initialize logging
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Install {
            browser,
            skip_driver,
            force,
        } => install::run(&browser, skip_driver, force).await,

        Commands::List => list::run().await,

        Commands::Uninstall { browser } => uninstall::run(&browser).await,
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}
