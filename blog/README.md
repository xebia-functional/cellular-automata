# Cellular Automata

Stanisław Ulam and John von Neumann were contemporaries at Los Alamos National
Laboratory, working on different aspects of the famous (or infamous) Manhattan
Project. Many weighty things have been said on their work there, but I'm going
to concentrate on a fun little corner of their collaboration: cellular automata.

A _cellular automaton_ comprises a regular grid of _cells_, each of which must
express exactly one of a finite set of _states_. For each cell, it and its
adjacent cells constitute its _neighborhood_; to avoid edge conditions, the two
"edges" of each dimension are considered adjacent. A cellular automaton may
occupy an arbitrary, nonzero number of dimensions, and each dimension is
considered for the purpose of identifying a cell's neighborhood. A cellular
automaton _evolves_ over _time_ as governed by some _rule_ that dictates the
next state of each cell based on the current state of its neighborhood. The full
set of next states is called the next _generation_. The evolutionary rule is
typically uniform and unchanging over time, but this is not strictly required.

It's fun to watch a cellular automaton evolve, but cellular automata are more
than mere mathematical toys. In 1970, John Conway introduced the _Game of Life_,
the two-dimensional cellular automaton now famous for its blocks, beehives,
boats, blinkers, pulsars, gliders, and spaceships. In 1982, Conway published a
proof of Turing-completeness, putting the automaton on the same computational
footing as Turing machines and lambda calculus. Also in the 1980s, Stephen
Wolfram systematically studied _elementary cellular automata_ — one-dimensional
cellular automata of irreducible simplicity. In 1985, Wolfram conjectured that
one of these automata, called Rule #110, was Turing-complete, and in 2004,
Matthew Cook published the proof.

In this post, we are going to bring elementary cellular automata to life using
the Rust programming language and the Bevy game engine. We'll learn a few things
about cellular automata, Rust, entity-component-system architecture, and basic
game development.

## Elementary cellular automata

An _elementary cellular automaton_ is usually conceptualized as a single row of
cells, each of which must express one of two states: _on_, represented by a
black cell; or _off_, represented by a white cell. If you're already familiar
with the Game of Life, then you might think of `on` as _live_ and `off` as
_dead_.

Multiple generations are usually represented as a two-dimensional grid, such
that each row represents a complete generation. New generations are added at the
end, so evolution progresses downward over time. Evolution is driven by a single
fixed rule. This rule encodes the complete transition table for the eight
possible neighborhoods.

To see why there are eight possible neighborhoods, let's consider adjacency in
one-dimensional space. We'll define the _distance_ between two cells $a$ and $b$
in the natural way: as the number of cell borders that must be crossed in
transit from $a$ to $b$. Further, we'll define the _neighborhood_ of some cell
$a$ as comprising all cells whose distance from $a$ is less than or equal to
one. For any cell $a$, that gives us $a$'s left neighbor, $a$ itself, and
$a$'s right neighbor.

Now recall that each cell can express exactly two states, `on` and `off`. Taking
state into account, there are a total of $2^3 = 8$ possible neighborhoods. Using
`X` to represent `on` and `•` to represent off, the eight possible neighborhoods
and their population ordinals are:

```text
••• (0)
••X (1)
•X• (2)
•XX (3)
X•• (4)
X•X (5)
XX• (6)
XXX (7)
```

A rule dictates the outcome — `on` or `off` — for each cell as a consequence of
its previous neighborhood state. Because there are eight possible neighborhoods,
and each neighborhood can produce one of two resultant states, there are 
$2^8 = 256$ possible rules. The result state of each neighborhood can be mapped
to a single bit in an 8-bit number; this representational strategy is commonly 
called _Wolfram coding_. In a Wolfram code, each bit, numbered `0` through `7`,
corresponds to one of the eight possible neighborhoods illustrated above. The
presence of a `1` in the $n^(th)$ bit means that neighborhood $n$ produces an
`on` cell, whereas `0` at $n$ means that the neighborhood produces an `off`
cell.

Let's consider Rule #206 as a concrete case. First, we convert to binary to
obtain:

$$ 206 = 2^7 + 2^6 + 2^3 + 2^2 + 2^1 = 11001110 $$

We can visualize the transition table thus:

```text
                         Rule #206
bit index  =    7     6     5     4     3     2     1     0
binary     =    1     1     0     0     1     1     1     0
input      =   XXX   XX•   X•X   X••   •XX   •X•   ••X   •••
output     =    X     X     •     •     X     X     X     •
```

For convenience of demonstration only, we restrict the cellular automaton to 30
cells. We are free to choose an initial generation for our cellular automaton
arbitrarily, though certain choices lead to much more interesting results.
Because it will be interesting, we begin with only one cell expressing the `on`
state. Then we run Rule #206 for 9 generations.

```text
generation 0 = •••••••••••••••X••••••••••••••
generation 1 = ••••••••••••••XX••••••••••••••
generation 2 = •••••••••••••XXX••••••••••••••
generation 3 = ••••••••••••XXXX••••••••••••••
generation 4 = •••••••••••XXXXX••••••••••••••
generation 5 = ••••••••••XXXXXX••••••••••••••
generation 6 = •••••••••XXXXXXX••••••••••••••
generation 7 = ••••••••XXXXXXXX••••••••••••••
generation 8 = •••••••XXXXXXXXX••••••••••••••
generation 9 = ••••••XXXXXXXXXX••••••••••••••
```

Two cool things happened here. One is easy to see, literally: it drew a right
triangle. The other requires a bit of interpretation. We've relied on binary
codings a lot already in this post, so let's indulge a bit more. We can treat
the `on` cells as a string of binary digits, such that the rightmost `on` cell
corresponds to $2^0$. Now we can interpret the generations as the elements in a
sequence:

```text
1, 3, 7, 15, 31, 127, 511, 2047, 8191, 32767, …
```

This series corresponds to the _Mersenne numbers_, where $n >= 1$: 

$$ M_n = 2^n − 1 $$

Other rules produce evolutions with startling correspondences to other
mathematical entities, like the Jacobsthal numbers and Pascal's triangle. And
rules #110 and #124 are both capable of universal computation.

Now that we know why elementary cellular automata are interesting, let's build
an evolver using the Bevy game engine.
