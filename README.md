# `oscd`

[![Build](https://github.com/karnpapon/oscd/actions/workflows/build.yml/badge.svg)](https://github.com/karnpapon/oscd/actions/workflows/build.yml)
[![Release](https://github.com/karnpapon/oscd/actions/workflows/release.yml/badge.svg)](https://github.com/karnpapon/oscd/actions/workflows/release.yml)

`oscd`, a simple interactive OSC debugger for the terminal inspired by [osc-debugger](https://github.com/alexanderwallin/osc-debugger), with auto type casting and support sending multiple osc msg. 

<img src="./ss3.gif">

It has two simple features:

* Monitor OSC messages (over UDP) sent to a port
* Send OSC messages (over UDP) to a port
  - default port = `57110`
  - default address = `127.0.0.1`

## Run
- easiest way is to download [released binary files](https://github.com/karnpapon/oscd/releases), unzip and put it where executable file lives based on your Operating System eg. `usr/local/bin` (for OSX)
- type `oscd` to run program

## Development
- `cargo run` 

## Usage
- Use the following format to send messages: `<address> <value>`
- `<address>` is osc path to communicate with.
- `<value>` is a number or a <ins>string without wrapping in double quotes</ins> and can have multiple values.
- eg. `/s_new default -1 0 0 freq 850`, will be parsed as `("s_new", [String("default"), Int(-1), Int(0), Int(0), String("freq"), Int(850)])`)


## Building / Release
- binary building with Github Action and supported following architectures
  - aarch64-linux
  - x86_64-linux
  - x86_64-macos
  - x86_64-windows
