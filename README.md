# Cargomon

Cargomon is a Rust implementation inspired by the popular Node.js tool nodemon. It watches your Rust project for file changes and automatically rebuilds and runs your application.

## Installation

You can install Cargomon using cargo:

```bash
cargo install cargomon
```

## Usage

Navigate to your Rust project directory and run:

```bash
cargomon
```



Cargomon will watch for file changes in your project, rebuild when changes are detected, and run the resulting executable.

## Features

- Automatic rebuilding and restarting of your Rust application
- Customizable file watching patterns
- Support for cargo workspaces
- Configurable through command-line options 



For more configuration options, run `cargomon --help`.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Acknowledgements

This project was inspired by [nodemon](https://nodemon.io/) for Node.js.