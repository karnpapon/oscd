# `oscd`

[![Build](https://github.com/karnpapon/oscd/actions/workflows/build.yml/badge.svg)](https://github.com/karnpapon/oscd/actions/workflows/build.yml)
[![Release](https://github.com/karnpapon/oscd/actions/workflows/release.yml/badge.svg)](https://github.com/karnpapon/oscd/actions/workflows/release.yml)

`oscd`, a simple interactive [OSC](https://en.wikipedia.org/wiki/Open_Sound_Control) debugger for the terminal inspired by [osc-debugger](https://github.com/alexanderwallin/osc-debugger), with auto type casting and support sending multiple osc arguments. 

<img src="./ss3.gif">

It has two simple features:

* Monitor OSC messages (over UDP) sent to a port
* Send OSC messages (over UDP) to a port
  - default port = `57110`
  - default address = `127.0.0.1`

## Install / Run
- easiest way is to download [released binary files](https://github.com/karnpapon/oscd/releases), unzip and put it where executable file lives based on your Operating System eg. `usr/local/bin` (for OSX)
- type `oscd` to run program

## Usage
- Use the following format to send messages: `<address> <argument>`
- `<address>` is osc path to communicate with.
- `<argument>` is a number or a string (double quotes can be omitted) and can have multiple arguments.
- eg. `/s_new "default" -1 0 0 freq 850`, will be parsed as `("s_new", [String("default"), Int(-1), Int(0), Int(0), String("freq"), Int(850)])`)
- by default `oscd` automatically casting type for you, ~~and it also support [numeric literals type conversion](https://doc.rust-lang.org/rust-by-example/types/cast.html)~~
  - ~~eg. `65.4321_f64` is equivalent to `65.4321 as f64` (`Explicit conversion`)~~
  - ~~it will be parsed osc as `Double(65.4321)`, otherwise `osc` will parsed it based on the input (eg. `65.4321` = `f32`).~~
- see supported types below.

## Types [WIP]
`oscd` follows [OscType](https://docs.rs/rosc/latest/rosc/enum.OscType.html) from [rosc](https://github.com/klingtnet/rosc) library

| status  | types                | example                            | notes                                                         |
|---------|----------------------|------------------------------------|---------------------------------------------------------------|
| &#9745; | Int(i32)             | `1234`                             |                                                               |
| &#9744; | Long(i64)            |                                    |                                                               |
| &#9745; | Float(f32)           | `1234.32`                          |                                                               |
| &#9744; | Double(f64)          |                                    |                                                               |
| &#9745; | String(String)       | `str_no_space`, `"str with space"` |                                                               |
| &#9745; | Bool(bool)           | `true` or `false`                  |                                                               |
| &#9744; | Char(char)           |                                    |                                                               |
| &#9744; | Blob(Vec&#60;u8>)    |                                    |                                                               |
| &#9744; | Time(OscTime)        |                                    |                                                               |
| &#9745; | Color(OscColor)      | `#2F14DF2A`                        | use hexadecimal pattern `#<red><green><blue><alpha>`          |
| &#9744; | Midi(OscMidiMessage) |                                    |                                                               |
| &#9745; | Array(OscArray)      | `[10,20,true]`                     |                                                               |
| &#9745; | Nil                  | `Nil`                              |                                                               |
| &#9745; | Inf                  | `Inf`                              |                                                               |

## Development
- `cargo run` 

## Building / Release
- binary building with Github Action and supported following architectures
  - aarch64-linux
  - x86_64-linux
  - x86_64-macos
  - x86_64-windows


## ⚠️ Bypass security settings. (OSX)

With each iteration of OS X from Mountain Lion onwards, Apple have made it progressively harder for users to access un-certificated downloaded applications/binary, such as those coming from the Open Source/Free Software community.

The problem typically manifests when trying to launch a newly downloaded application/binary whether directly or via the Dock. At the point of downloading a new app, the OS places it on a “quarantine list”. An alarming error message is displayed indicating the application is “damaged”, or from an unidentified developer, and has been prevented from running.

A standard workaround for a single application/binary is to launch using “Open” from the menu that pops up using Right-Click (or Ctrl-Click) on the application’s/binary's icon.

## Resources
- https://ccrma.stanford.edu/groups/osc/index.html
- https://ccrma.stanford.edu/groups/osc/spec-1_0.html
- https://ccrma.stanford.edu/groups/osc/files/2009-NIME-OSC-1.1.pdf