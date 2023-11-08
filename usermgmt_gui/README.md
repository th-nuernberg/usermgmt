# Readme

This crate is an application which allows simultaneous user management for LDAP and Slurm 
via GUI.

This application utilizes [egui](https://github.com/emilk/egui) as the GUI framework.

## License

The whole project is licensed under [MIT](./LICENSE)

## Development

You can run this application by running the following command 

```bash
cargo gui
```

## GUI settings

Options which determine the look of the GUI are determined by values contained 
within certain configuration files under assets.

There are two files 

- [Init file](./assets/Init.toml): Only loaded at the start of the application.
- [Settings file](./assets/Settings.toml): Loaded at the start of the application. 
During development, this file is also loaded whenever its content changes. This hot reload feature
while developing allows to see the changes in the GUI immediately. 

Note: In release the reload feature of the [Settings file](./assets/Settings.toml) is not active.
This program includes both files into the resulting binary at compile time.
So user of the application do not need to worry about having theses files on their system.

## Additional information

The [GENERAL_README](./GENERAL_README.md) describes the other crates and the general structure
of the whole project.
