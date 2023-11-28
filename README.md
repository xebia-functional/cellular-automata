Cellular Automata
----------------------------

Herein is a simulator for
[elementary&#32;cellular&#32;automata](https://en.wikipedia.org/wiki/Elementary_cellular_automaton),
written in [Rust](https://www.rust-lang.org/) using the
[Bevy](https://bevyengine.org/) game engine.

The simulator begins paused, using a random seed and rule (unless overridden
with command line options).

* Press the space bar to unpause; when the simulator is running, press the space
  bar to pause.
* When paused, click any cell in the bottom row to toggle its state, i.e.,
  alive -> dead, dead -> alive.
* At any time, type a new rule, specified as a
  [Wolfram&#32;code](https://en.wikipedia.org/wiki/Wolfram_code) in
  `[0,255]`, to altar the evolution of the automaton.
* Hold the right shift key to display the frames per second (FPS).

To run the WASM build on GitHub Pages, go
[here](https://47degrees.github.io/cellular-automata). Note that this is not
expected to work on mobile, so use a desktop browser instead. Also, you will
need to click the WASM canvas to transfer focus; key presses won't be detected
before this.

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

When running the application natively from the command line, the following
parameters are available:

```text
$ cargo run -- --help
Fun with cellular automata! Set the first generation with a known seed and/or
rule, or let the program choose randomly. Watch the automaton evolve, and
influence its evolution with the keyboard and mouse

Usage: cellular-automata [OPTIONS]

Options:
  -r, --rule <RULE>  The rule, specified as a Wolfram code between 0 and 255,
                     inclusive. If unspecified, the rule will be chosen randomly
  -s, --seed <SEED>  The first generation, specified as a 64-bit integer that
                     represents the complete population. Lower numbered bits
                     correspond to cells on the right of the visualization. If
                     unspecified, the first generation will be chosen randomly
  -h, --help         Print help
```

If `rule` is unspecified, then a rule will be chosen randomly. Likewise, if
`seed` is unspecified, then a seed will be chosen randomly.

Query Parameters
----------------

When running the application from a web browser, the following query parameters
are available:

```text
rule=<RULE>          The rule, specified as a Wolfram code between 0 and 255,
                     inclusive. If unspecified, the rule will be chosen randomly
seed=<SEED>          The first generation, specified as a 64-bit integer that
                     represents the complete population. Lower numbered bits
                     correspond to cells on the right of the visualization. If
                     unspecified, the first generation will be chosen randomly
```

For example, the following URL illustrates running rule #206 on an initial
population of one (at index 12, counting up from 0 on the right):
https://xebia-functional.github.io/cellular-automata/?rule=206&seed=4096
