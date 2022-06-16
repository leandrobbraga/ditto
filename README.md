# Ditto

Ditto is an environment setup automation tool.

## How to use

It looks for a file called `package.json` in the root folder of the project. This file will contain
the packages that should be installed and also the configuration file, if necessary.

```json
[
    {
        "name": "fish",
        "config": ".config/fish/config.fish",
        "package_manager": "pacman"
    },
    {
        "name": "dunst",
        "config": null,
        "package_manager": "snap"
    },
    {
        "name": "alacritty",
        "config": ".config/alacritty/alacritty.yml"
    }
]
```

After running the command `cargo run -- install-packages` it will try to install it using the
package manager (currently only pacman is supported).

There is also `cargo run -- install-config` which will try to create a symlink from the
configuration file in the root folder to the destination folder.
