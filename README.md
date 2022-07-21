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

## Run
- easiest way is to download [released binary files](https://github.com/karnpapon/oscd/releases), unzip and put it where executable file lives based on your Operating System eg. `usr/local/bin` (for OSX)
- type `oscd` to run program

## Development
- `cargo run` 

## Usage
- Use the following format to send messages: `<address> <argument>`
- `<address>` is osc path to communicate with.
- `<argument>` is a number or a string (double quotes can be omitted) and can have multiple arguments.
- eg. `/s_new "default" -1 0 0 freq 850`, will be parsed as `("s_new", [String("default"), Int(-1), Int(0), Int(0), String("freq"), Int(850)])`)
- by default `oscd` automatically casting type for you, but it also support [Rust implicit conversion](https://doc.rust-lang.org/rust-by-example/types/cast.html) eg. `65.4321_f64` is equivalent to `65.4321 as f64` (in Rust language) and will be parsed osc as `Double(65.4321)` see support types below.

## Types [WIP]
`oscd` follows [OscType](https://docs.rs/rosc/latest/rosc/enum.OscType.html) from [rosc](https://github.com/klingtnet/rosc) library
- [x] Int(i32)  
- [x] Float(f32)  
- [x] Double(f64)
- [x] String(String)
- [x] Bool(bool)
- [x] Char(char)
- [x] Long(i64)
- [ ] Blob(Vec<u8>)
- [ ] Time(OscTime)
- [ ] Color(OscColor)
- [ ] Midi(OscMidiMessage)
- [ ] Array(OscArray)
- [x] Nil
- [x] Inf

## Building / Release
- binary building with Github Action and supported following architectures
  - aarch64-linux
  - x86_64-linux
  - x86_64-macos
  - x86_64-windows


## Resources
- https://ccrma.stanford.edu/groups/osc/index.html
- https://ccrma.stanford.edu/groups/osc/spec-1_0.html
- https://ccrma.stanford.edu/groups/osc/files/2009-NIME-OSC-1.1.pdf