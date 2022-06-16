#![allow(dead_code)]
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct Package {
    pub name: String,
    pub config: Option<PathBuf>,
}

pub struct Fish;

pub struct LinuxFilesystem;

pub struct PacMan<S: Shell> {
    shell: S,
}

pub trait PackageManager {
    fn install(&self, packages: &[Package]);
}

pub trait Shell {
    fn run(&self, command: &str, arguments: &[&str]);
    fn sudo_run(&self, command: &str, arguments: &[&str]);
}

pub trait Filesystem {
    fn symlink(&self, original: impl AsRef<Path>, link: impl AsRef<Path>);
}

impl<S: Shell> PacMan<S> {
    pub fn new(shell: S) -> Self {
        PacMan { shell }
    }
}

impl<S: Shell> PackageManager for PacMan<S> {
    fn install(&self, packages: &[Package]) {
        let mut arguments = vec!["-Syyu"];
        arguments.extend(packages.iter().map(|package| &package.name[..]));

        self.shell.sudo_run("pacman", &arguments)
    }
}

impl Shell for Fish {
    fn run(&self, command: &str, arguments: &[&str]) {
        std::process::Command::new(command)
            .args(arguments)
            .status()
            .unwrap();
    }
    fn sudo_run(&self, command: &str, arguments: &[&str]) {
        std::process::Command::new("sudo")
            .arg(command)
            .args(arguments)
            .status()
            .unwrap();
    }
}

impl Filesystem for LinuxFilesystem {
    fn symlink(&self, original: impl AsRef<Path>, link: impl AsRef<Path>) {
        std::os::unix::fs::symlink(original, link).unwrap()
    }
}

pub fn install_config_files(packages: &[Package], filesystem: &impl Filesystem) {
    // I'm allowing deprecated code here because I've looked into the library recommendation
    // (home_dir) and - for linux - it does use `std::env::home_dir` as well.
    #[allow(deprecated)]
    let home = std::env::home_dir().unwrap();
    let home = home.to_str().unwrap();

    for package in packages {
        if let Some(path) = &package.config {
            let src = path.file_name().unwrap();
            let dst = Path::new(home).join(path.to_str().unwrap());

            filesystem.symlink(src, dst)
        }
    }
}

#[cfg(test)]
mod test {
    use std::cell::RefCell;

    use crate::{install_config_files, Filesystem, PacMan, Package, PackageManager, Shell};

    #[derive(Debug)]
    struct MockShell {
        calls: RefCell<Vec<MockShellCall>>,
    }

    impl MockShell {
        fn new() -> Self {
            MockShell {
                calls: RefCell::new(vec![]),
            }
        }

        fn assert_called_once_with(&self, sudo: bool, command: &str, arguments: &[&str]) {
            let calls = &*self.calls.borrow_mut();

            assert!(calls.len() == 1);

            assert_eq!(calls[0].sudo, Some(sudo));
            assert_eq!(calls[0].command, Some(command.to_string()));
            assert_eq!(calls[0].arguments, arguments);
        }

        #[allow(dead_code)]
        fn assert_called_with(&self, sudo: bool, command: &str, arguments: &[&str]) {
            let calls = &*self.calls.borrow_mut();

            let mut result = false;

            for call in calls {
                if call.sudo == Some(sudo)
                    && call.command == Some(command.to_string())
                    && call.arguments == arguments
                {
                    result = true;
                    break;
                }
            }

            assert!(
                result,
                "could not find call: sudo={sudo}, command={command}, arguments={arguments:?} in calls {calls:?}"
            )
        }
    }

    #[derive(Debug)]
    struct MockShellCall {
        pub sudo: Option<bool>,
        pub command: Option<String>,
        pub arguments: Vec<String>,
    }

    impl Shell for MockShell {
        fn run(&self, command: &str, arguments: &[&str]) {
            let call = MockShellCall {
                sudo: Some(false),
                command: Some(command.to_string()),
                arguments: arguments
                    .iter()
                    .map(|argument| argument.to_string())
                    .collect(),
            };

            self.calls.borrow_mut().push(call);
        }

        fn sudo_run(&self, command: &str, arguments: &[&str]) {
            let call = MockShellCall {
                sudo: Some(true),
                command: Some(command.to_string()),
                arguments: arguments
                    .iter()
                    .map(|argument| argument.to_string())
                    .collect(),
            };

            self.calls.borrow_mut().push(call);
        }
    }

    fn setup_packages() -> Vec<Package> {
        vec![
            Package {
                name: "fish".to_string(),
                config: Some(".config/fish/config.fish".into()),
            },
            Package {
                name: "dunst".to_string(),
                config: None,
            },
            Package {
                name: "alacritty".to_string(),
                config: Some(".config/alacritty/alacritty.yml".into()),
            },
        ]
    }

    #[test]
    fn test_pacman_install() {
        let packages = setup_packages();

        let shell = MockShell::new();
        let package_manager = PacMan { shell };

        package_manager.install(&packages);

        package_manager.shell.assert_called_once_with(
            true,
            "pacman",
            &["-Syyu", "fish", "dunst", "alacritty"],
        )
    }

    struct MockFilesystem {
        calls: RefCell<Vec<FilesystemCall>>,
    }

    impl MockFilesystem {
        fn new() -> Self {
            MockFilesystem {
                calls: RefCell::new(vec![]),
            }
        }

        fn assert_called_with(&self, original: String, link: String) {
            let calls = self.calls.borrow();
            let calls: &Vec<FilesystemCall> = calls.as_ref();

            let mut result = false;

            for call in calls {
                if call.original == original && call.link == link {
                    result = true;
                    break;
                }
            }

            assert!(
                result,
                "could not find call: original={original}, link={link} in calls {calls:?}"
            )
        }
    }

    impl Filesystem for MockFilesystem {
        fn symlink(
            &self,
            original: impl AsRef<std::path::Path>,
            link: impl AsRef<std::path::Path>,
        ) {
            let call = FilesystemCall {
                original: original.as_ref().to_str().unwrap().to_string(),
                link: link.as_ref().to_str().unwrap().to_string(),
            };

            self.calls.borrow_mut().push(call);
        }
    }

    #[derive(Debug)]
    struct FilesystemCall {
        original: String,
        link: String,
    }

    #[test]
    fn test_install_config_files() {
        let packages = setup_packages();
        let filesystem = MockFilesystem::new();
        install_config_files(&packages, &filesystem);

        filesystem.assert_called_with(
            "alacritty.yml".to_string(),
            "/home/leandro/.config/alacritty/alacritty.yml".to_string(),
        );
        filesystem.assert_called_with(
            "config.fish".to_string(),
            "/home/leandro/.config/fish/config.fish".to_string(),
        );
    }
}
