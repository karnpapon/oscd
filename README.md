# `oscd`

`oscd`, a simple interactive OSC debugger for the terminal inspired by [osc-debugger](https://github.com/alexanderwallin/osc-debugger),with auto type casting and support sending multiple osc msg. 

<img src="./ss.gif">


It has two simple features:

* Monitor OSC messages (over UDP) sent to a port [WIP]
* Send OSC messages (over UDP) to a port
  - default port = 57110
  - default address = '0.0.0.0'

## Run
- `cargo run` 

## Usage
- Use the following format to send messages: `<address> <value>`
- `<address>` is osc path to communicate with.
- `<value>` is a number or a string without wrapping in double quotes (can have multiple values) 
- eg. `/s_new default -1 0 0 freq 850`, will be parsed as `("s_new", [String("default"), Int(-1), Int(0), Int(0), String("freq"), Int(850)])`)



