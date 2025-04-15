# Usermgmt GUI

This application utilizes [egui](https://github.com/emilk/egui).

## License

The project is licensed under [MIT](./LICENSE)

## Development

You can run the GUI with: 

```bash
cargo gui
```

## GUI Settings

The GUI layout can be configured with the following two files:

- [Init file](./assets/Init.toml): Only loaded at the start of the application.
- [Settings file](./assets/Settings.toml): Loaded at the start of the application.  
  - During development, this file is loaded whenever its content changes. 

**Note:** In release mode the reload feature of the [Settings file](./assets/Settings.toml) is not active.
This program includes both files into the resulting binary at compile time. 

## Additional Information

The [GENERAL_README](./GENERAL_README.md) describes the other crates and the general structure
of the project.
