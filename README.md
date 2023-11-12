Cellular Automata
----------------------------

Herein is a simulator for
[elementary&#32;cellular&#32;automata](https://en.wikipedia.org/wiki/Elementary_cellular_automaton),
written in [Rust](https://www.rust-lang.org/) using the
[Bevy](https://bevyengine.org/) game engine.

The simulator begins paused. Press the space bar to unpause; when the simulator
is running, press the space bar to pause. At any time, type a new rule,
specified as a [Wolfram&#32;code](https://en.wikipedia.org/wiki/Wolfram_code) in
`[0,255]`, to altar the evolution of the automaton. Hold the right shift key to
display the frames per second (FPS).

To run the WASM build on GitHub Pages, go
[here](https://47degrees.github.io/cellular-automata).

Building
--------

```shell
$ cargo build --release
```

Running
-------

To run natively:

```shell
$ cargo run
```

Command Line Arguments
----------------------

```shell
$ cargo run -- --help
Usage: cellular-automata [OPTIONS]

Options:
  -r, --rule <RULE>  The [rule](AutomatonRule], specified as a Wolfram code in
                     `[0,255]`
  -s, --seed <SEED>  The [seed](Automaton), specified as a `u64` that represents
                     the state of the initial generation
  -h, --help         Print help
```

If `rule` is unspecified, then a rule will be chosen randomly. Likewise, if
`seed` is unspecified, then a seed will be chosen randomly.