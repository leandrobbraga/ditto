use anyhow::Result;
use clap::{Parser, Subcommand};
use ditto::{install_config_files, LinuxFilesystem, PacMan, Package, PackageManager, Shell};

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

    let packages: Vec<Package> = serde_json::from_reader(file)?;
    let shell = EchoShell {};

    match args.command {
        Commands::InstallPackages => {
            let pacman = PacMan::new(shell);
            pacman.install(&packages)
        }
        Commands::InstallConfig => {
            let filesystem = LinuxFilesystem {};
            install_config_files(&packages, &filesystem)
        }
    };

    Ok(())
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
