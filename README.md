# Scoopie (WIP)

> If Scoop is vanilla, Scoopie will be tutti frutti.

Scoopie is a high-performance alternative to the Scoop package manager for Windows, built from the ground up in Rust. It aims to provide a blazing-fast package management experience while minimizing dependencies and optimizing resource usage. Scoopie leverages parallel processing, bundled tools, and an efficient storage structure to deliver exceptional performance.

## Key Features

- **Lightweight and Fast**: Scoopie is designed to be lightweight and lightning-fast. It minimizes dependencies, resulting in a small footprint and quick startup times.
- **Reduced Dependencies**: Scoopie comes compiled with Git, eliminating the need for users to pre-install it on their systems. This streamlines the installation process and ensures seamless integration with repositories and version control.
- **Efficient Storage**: Instead of using a flat-file structure like Scoop, Scoopie leverages the power of SQLite. This optimized storage approach reduces disk space usage and enhances the search experience.
- **Parallel Processing**: Scoopie harnesses the power of parallel processing to perform operations in parallel, greatly improving overall performance. This enables faster package installation, updates, and searches, enhancing the user experience.
- **Built-in Download Manager**: Scoopie includes a bundled efficient download manager, eliminating the reliance on external tools like `aria2`. This simplifies the installation process and provides a seamless downloading experience.
- **User-friendly CLI**: Scoopie offers a user-friendly command-line interface, allowing users to easily manage packages, perform searches, and update their installations. The CLI provides intuitive commands and helpful feedback to ensure a smooth experience.

## Installation

To install Scoopie, follow these steps:

1. Download the Scoopie installer from the official repository.
2. Run the installer, and it will guide you through the installation process.
3. Once installed, you can start using Scoopie right away.

## Usage

Scoopie provides a set of commands that allow you to manage packages efficiently. Here are some commonly used commands:

- `scoopie install <package>`: Installs the specified package.
- `scoopie install -S`: Updates all buckets to their latest versions.
- `scoopie query <keyword>`: Searches for packages matching the provided keyword.
- `scoopie rm <package>`: Uninstalls the specified package.

For a complete list of commands and their usage, please refer to the official documentation.

## Contributing

Contributions to Scoopie are welcome! If you'd like to contribute, please follow the guidelines outlined in the CONTRIBUTING.md file. You can report issues, suggest improvements, or submit pull requests on the official GitHub repository.

## License

Scoopie is released under the MIT License. See the LICENSE file for more details.

## Acknowledgements

Scoopie would not be possible without the hard work and dedication of the open-source community. We extend our gratitude to the developers of Rust, SQLite, and other libraries used in this project.

## Contact

If you have any questions, suggestions, or feedback, feel free to open an issue. I'd love to improve this project further!
