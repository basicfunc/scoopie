# Scoopie (WIP)
![Build CI](https://github.com/basicfunc/scoopie/actions/workflows/build.yml/badge.svg)

> If Scoop is vanilla, Scoopie will be tutti frutti.
>                                               - Rahul   

Scoopie is a high-performance alternative to the Scoop package manager for Windows, built from the ground up in Rust. It aims to provide a blazing-fast package management experience while minimizing dependencies and optimizing resource usage. Scoopie leverages parallel processing, bundled tools, and an efficient storage structure to deliver exceptional performance.

***NOTE: It is not meant to be one-to-one clone to Scoop, it just uses scoop buckets.***

## Key Features

- **Lightweight and Fast**: Scoopie sets a new standard for lightweight, high-speed package management. It leverages the full potential of parallel processing to execute operations simultaneously, turbocharging your tasks. With its minimalistic approach, Scoopie keeps dependencies to an absolute minimum, ensuring blazing-fast startup times and a compact footprint.

- **No External Dependencies**: Say goodbye to pre-installing Git or codecs – Scoopie comes prepared with them, simplifying your setup. It's ready to roll right out of the box, sparing you the hassle of hunting down dependencies.

- **Optimized Storage**: Scoopie redefines package storage by switching to a JSON-based bucket structure. This not only conserves precious disk space but also elevates your search experience to new heights.

- **Seamless Downloads**: No need for external download managers like `aria2`. Scoopie handles package downloads with finesse, making the installation process hassle-free and the downloading experience seamless.

- **Intuitive Command-Line Interface**: Scoopie boasts a user-friendly CLI that empowers you to effortlessly manage packages, conduct searches, and keep your installations up to date. Our CLI offers intuitive commands and informative feedback, ensuring a smooth and enjoyable experience for users of all levels.

## Installation

To install Scoopie, follow these steps:

```bash
> git clone https://github.com/rawhuul/scoopie
> cd scoopie
> cargo build --release
> scoopie init <your desired path>
> scoopie install <package>
```

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

## Contact

If you have any questions, suggestions, or feedback, feel free to open an issue. I'd love to improve this project further!
