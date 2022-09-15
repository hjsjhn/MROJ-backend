<p align="center">
  <img src="MROJ_logo.png" alt="MROJ_logo" width="300" />
</p>

# MROJ - Making a Rust Oneline Judge

## Background

This repo was initially a homework of Tsinghua University Programing and Training Course, but I found it a good chance to release one on Github.

## Installation

This project uses [rust](https://github.com/rust-lang/rust) and [cargo](https://github.com/rust-lang/cargo). You need to have them locally installed to compile this project.

You can simply run MROJ using

``` bash
$ cargo run
```

or build a binary file in target/release/mroj-backend

``` bash
$ cargo build --release
```

## Usage

Assuming you have compiled the binaries, executing `mroj-backend` directly in the terminal will run the backend, but you can of course add the following argument:

```
-c, --config <config_path>    The config file path.
-f, --flush-data              Toggle to flush OJ data in database.
-h, --help                    Print help information
-V, --version                 Print version information
```

## APIs

Some of the APIs was given by the TAs of the course mentioned before. I must offer my thanks to them.

Given that the backend and the frontend are divided, the APIs are all HTTP requests and responses.

You can get all APIs [here](https://github.com/hjsjhn/MROJ-backend/wiki#apis).

