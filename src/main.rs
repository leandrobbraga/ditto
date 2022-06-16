use std::collections::HashMap;

use anyhow::Result;
use clap::{Parser, Subcommand};
use ditto::{
    install_config_files, Filesystem, LinuxFilesystem, PacMan, Package, PackageManager, Shell, Snap,
};
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
                    PackageManagers::PacMan => PacMan::new(&shell).install(&packages),
                    PackageManagers::Snap => Snap::new(&shell).install(&packages),
                };
            }
        }
        Commands::InstallConfig => {
            let filesystem = EchoFilesystem {};

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

struct EchoShell {}

impl Shell for EchoShell {
    fn run(&self, command: &str, arguments: &[&str]) {
        println!("command {command} arguments {arguments:?}")
    }

    fn sudo_run(&self, command: &str, arguments: &[&str]) {
        println!("command {command} arguments {arguments:?}")
    }
}

struct EchoFilesystem {}

impl Filesystem for EchoFilesystem {
    fn symlink(&self, original: impl AsRef<std::path::Path>, link: impl AsRef<std::path::Path>) {
        let original = original.as_ref().as_os_str();
        let link = link.as_ref().as_os_str();

        println!("create symlink from {original:?} to {link:?}")
    }
}
