use std::collections::HashMap;

use anyhow::Result;
use clap::{Parser, Subcommand};
use ditto::{install_config_files, LinuxFilesystem, PacMan, Package, PackageManager, Shell, Snap};
use serde::Deserialize;

#[derive(Parser, Debug)]
#[clap(name = "ditto")]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    InstallPackages,
    InstallConfig,
}

fn main() -> Result<()> {
    let args = Cli::parse();

    let file = std::fs::File::open("packages.json")?;

    let packages: HashMap<PackageManagers, Vec<Package>> = serde_json::from_reader(file)?;
    let shell = EchoShell {};

    match args.command {
        Commands::InstallPackages => {
            for (package_manager, packages) in packages {
                match package_manager {
                    PackageManagers::PacMan => PacMan::new(shell.clone()).install(&packages),
                    PackageManagers::Snap => Snap::new(shell.clone()).install(&packages),
                };
            }
        }
        Commands::InstallConfig => {
            let filesystem = LinuxFilesystem {};

            let packages: Vec<Package> = packages.into_values().flatten().collect();
            install_config_files(&packages, &filesystem)
        }
    };

    Ok(())
}

#[derive(Deserialize, PartialEq, Eq, Hash, Debug)]
enum PackageManagers {
    PacMan,
    Snap,
}

#[derive(Clone)]
struct EchoShell {}

impl Shell for EchoShell {
    fn run(&self, command: &str, arguments: &[&str]) {
        println!("command {command} arguments {arguments:?}")
    }

    fn sudo_run(&self, command: &str, arguments: &[&str]) {
        println!("command {command} arguments {arguments:?}")
    }
}
