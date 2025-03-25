# SALTBUILD - Custom SIEM

A custom built SIEM written in Rust. To serve as apart of the SALTBUILD suite of IT/Security software & tools.
## Table of Contents

- [Installation](#installation)
- [Usage](#usage)
- [Contributing](#contributing)
- [License](#license)

## Installation

Instructions on how to install and set up the project.

```bash
# Example command
git clone https://github.com/yourusername/yourproject.git
cd yourproject
```

## Usage

Examples of how to use the project.

```bash
# Example usage
cargo build
cargo run
```

You'll be greeted with a CLI

![Application Screenshot](assets/cli_home.png)

From here, you can start the syslog listener servers (TCP & UDP) on a custom port or using the one defined within the config.toml file
![Application Screenshot](assets/start_syslog_ingestor.png)
## Contributing

Contributions are welcome! Please see the [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

This project is licensed under the [MIT License](LICENSE).