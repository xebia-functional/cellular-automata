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
generation  1 = •••••••••••••••X••••••••••••••
generation  2 = ••••••••••••••XX••••••••••••••
generation  3 = •••••••••••••XXX••••••••••••••
generation  4 = ••••••••••••XXXX••••••••••••••
generation  5 = •••••••••••XXXXX••••••••••••••
generation  6 = ••••••••••XXXXXX••••••••••••••
generation  7 = •••••••••XXXXXXX••••••••••••••
generation  8 = ••••••••XXXXXXXX••••••••••••••
generation  9 = •••••••XXXXXXXXX••••••••••••••
generation 10 = ••••••XXXXXXXXXX••••••••••••••
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

Now that we know why elementary cellular automata are interesting, let's model
them in Rust.

## Modeling Elementary Cellular Automata with Rust

That project that I developed to accompany this blog post is rooted [here](..).
It's laid out in a pretty vanilla fashion, completely standard for a simple
binary crate.

The data model for the elementary cellular automaton is in
[src/automata.rs](../src/automata.rs), so all the code excepts in this section
are sourced from that file.

Let's look at the representation of an elementary cellular automaton first.

### `Automaton`, _const_ genericity, and conditional derivation

Essentially, we keep our representational strategy super simple: we express an
elementary cellular automaton as a boolean array, albeit with a few frills.

```rust
#[derive(Copy, Clone, Debug)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub struct Automaton<const K: usize = AUTOMATON_LENGTH>([bool; K]);
```

Using the _newtype_ pattern, we define a 1-element tuple `struct` to wrap an
array of `K` booleans. Rather than hard coding the exact size, we offer some
flexibility through _const generics_. In Rust, _const_ genericity ranges over
values of primitive types rather than types or lifetimes. We default the
_const_ parameter to `AUTOMATON_LENGTH`, making the bare type `Automaton`
equivalent to `Automaton<AUTOMATON_LENGTH>`. Elsewhere, we establish
`AUTOMATON_LENGTH` itself:

```rust
pub const AUTOMATON_LENGTH: usize = 64;
```

Because `AUTOMATON_LENGTH` is `const` data, we can use it to satisfy the _const_
parameter `K`. So our default `Automaton` will comprise 64 cells, which is
plenty for an interesting visualization.

`bool` implements `Copy`, and arrays implement `Copy` if their element type
does. Extending this chain of reasoning to `structs`, tuples, and `struct`
tuples, our newtype is also eligible to implement `Copy` because its only field
implements `Copy`. Even if we changed `AUTOMATON_LENGTH` to some other number,
it would need to be small enough to support all-at-once presentation in the
application, so it's efficient enough to derive `Copy` for `Automaton`.

The application itself doesn't rely on comparison, but the test suite does. We
get the best of both worlds with `cfg_attr`: we implement `PartialEq` and `Eq`
only when compiling and linking the test suite.

Note that the _natural ordering_ of the cells within the `Automaton` tracks the
array itself, so the cell at index `0` is the _leftmost_ cell and the cell at
index `K - 1` is the _rightmost_ cell. This will matter several times, because
different contexts imply different natural orderings, and we will sometimes need
to perform coordinate transformations to account for this.

### Succession and _const fn_

The `next` method computes the next generation of an `Automaton`. There are
three cases that `next` needs to handle:

1. Computing the leading edge cell, i.e., the rightmost one, which requires
   treating the trailing edge cell, i.e., the leftmost one, as its right
   neighbor.
2. Computing the medial cells, which is trivial once we are properly oriented.
3. Computing the trailing edge cell, which requires treating the leading edge
   cell as its left neighbor.

```rust
impl<const K: usize> Automaton<K>
{
	pub fn next(&self, rule: AutomatonRule) -> Self
	{
		let mut next = [false; K];
		// Compute the leading edge cell, treating the final cell of the
		// automaton as its right neighbor.
		let ordinal = compute_ordinal(self[1], self[0], self[K - 1]);
		next[0] = rule.next_cell(ordinal);
		// Computing the medial cells is trivial.
		for i in 1 ..= K - 2
		{
			let ordinal = compute_ordinal(
				self[i + 1],
				self[i],
				self[i - 1]
			);
			next[i] = rule.next_cell(ordinal);
		}
		// Compute the trailing edge cell, treating the initial cell of the
		// automaton as its left neighbor.
		let ordinal = compute_ordinal(self[0], self[K - 1], self[K - 2]);
		next[K - 1] = rule.next_cell(ordinal);
		Automaton(next)
	}
}
```

`compute_ordinal` is a _const_ function that determines the population ordinal
for some cell's neighborhood. To keep it independent of any particular cell's
identity, it accepts the exploded neighborhood state and answers the population
ordinal.

```rust
#[inline]
const fn compute_ordinal(left: bool, middle: bool, right: bool) -> u8
{
	let left = if left { 4u8 } else { 0 };
	let middle = if middle { 2u8 } else { 0 };
	let right = if right { 1u8 } else { 0 };
	let ordinal = left | middle | right;
	// Note that we cannot test range containment directly here because
	// `contains` is not a `const fn`.
	assert!(ordinal <= 7);
	ordinal
}
```

By the way, a _const_ function can be used at compile time to initialize _const_
data. _Const_ Rust is a foldable subset of Rust that operates only on literals,
_const_ data, and values produced by _const_ functions. The compiler evaluates
and folds _const_ expressions down into a single result. Using _const_ functions
lets your initialization logic focus on semantics rather than magic numbers.
_Const_ Rust is still limited in scope — it can't handle loops yet, for example
— but it has steadily gained more features over many releases. A good rule of
thumb is to make data and functions _const_ wherever possible, as it expands
your compile-time vocabulary and thus improves the expressiveness of your
_const_ and _static_ initializers.

Right, back to `next`. Armed with the population ordinal, we can call the
`next_cell` method to ask the supplied rule to produce the appropriate successor
for the corresponding neighborhood. After that, it's simply a matter of
clobbering the slots of the eponymous `next` array, then wrapping an `Automaton`
around it before returning.

### Rules

`AutomataRule` represents an evolutionary rule, once again using the newtype
pattern. This newtype wraps a Wolfram code, expressed as a `u8`.

```rust
#[derive(Copy, Clone, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Resource)]
pub struct AutomatonRule(u8);
```

Now we look at `next_cell`, which is completely straightforward:

```rust
impl AutomatonRule
{
	#[inline]
	const fn next_cell(self, ordinal: u8) -> bool
	{
		self.0 & (1 << ordinal) != 0
	}
}
```

The Wolfram code fully encapsulates the transition table for an elementary
cellular automaton, so it's a simple matter of extracting the bit associated
with a population ordinal. As expected, it's just a bit shift, a bitwise _and_,
and a test against zero.

### Instantiating an `Automaton<K>`

We need an ergonomic way to populate an `Automaton`:

```rust
impl<const K: usize> From<u64> for Automaton<K>
{
	fn from(value: u64) -> Self
	{
		assert!(K <= 0u64.count_zeros() as usize);
		let mut next = [false; K];
		for i in 0 ..= K - 1
		{
			next[i] = value & (1 << i) != 0;
		}
		Automaton(next)
	}
}
```

Pretty straightforward. There are only a couple tricks here.

1. We need to traverse both the array and the `u64` in the same direction, i.e.,
   from least-significant cell to most-significant cell. For the array, this
   means indexing up from zero; for the `u64`, this means masking up from $2^0$
   to $2^63$.
2. The `assert!` macro call is, _prima facie_, a dynamic guard on the value of
   the _const_ parameter `K`. `count_zeros` is a _const fn_, which we use to
   obtain the number of bits in a `u64`. We could instead insert the literal
   `64`, of course, but this technique clearly preserves the correlation between
   the type and its bit length. Since `K` is a _const_ parameter and
   `count_zeros` is a _const fn_ with a literal receiver (`0u64`), the whole
   predicate is a _const_ expression, meaning that the compiler can optimize
   away the runtime check whenever the _const_ expression evaluates to `true`.
   In release mode, this ends up being a static guard after all!

Worth noting, _const_ implementation bounds, i.e., _const_ expressions in
_const_ parameter positions, are available in nightly Rust, but not in stable
Rust. With the nightly toolchain, we could use a conditional trait
implementation on an unrelated helper type to statically guard against
out-of-range values of `K` through emission of a compiler error.

```rust
// Available in nightly only.
trait IsTrue {}
struct Guard<const C: bool>;
impl IsTrue for Guard<true> {}

impl<const K: usize> From<u64> for Automaton<K> where Guard<{K <= 64}>: IsTrue
{
	fn from(value: u64) -> Self
	{
		// …
	}
}
```

In this scenario, out-of-range `K` disables the implementation of
`From<u64>` for `Automaton<K>`, so attempting `from` with, e.g., an
`Automaton<90>`, causes the compiler to announce that the trait isn't
implemented.

But the `assert!` technique is the best we can do with stable Rust, and we use
stable Rust through this project in order to maximize stability and
availability.

### Testing the evolutionary mechanism

Before moving on to Bevy, we should test whether all this works. We pick an
arbitrary initial generation and rule, mechanically perform an evolution by
hand, then rely on structural induction to conclude that the implementation is
correct. Because we're free to pick anything we want, we pick `0x34244103` and
rule #30. This scenario is
[illustrated](https://en.wikipedia.org/wiki/Elementary_cellular_automaton#/media/File:One-d-cellular-automate-rule-30.gif)
on Wikipedia, so we can treat it as a community-checked test vector.

```rust
#[test]
fn rule_30()
{
	//     XX•X••••X••X•••X•••••X••••••XX
	// 0b00110100001001000100000100000011
	// 0x   3   4   2   4   4   1   0   3
	let automaton = Automaton::<30>::from(0x34244103);
	//     •••XX••XXXXXX•XXX•••XXX••••XX•
	// 0b00000110011111101110001110000110
	// 0x   0   6   7   E   E   3   8   6
	let expected = Automaton::<30>::from(0x067EE386);
	let actual = automaton.next(30.into());
	assert_eq!(expected, actual);
}
```

Our test suite includes a similar test for rule #110, just for safety, but I've
omitted it here for brevity.

## Entity-component-system (ECS) architecture

We start with theory, moved to practice, and now it's time for some more theory.
Before we dive into using Bevy, let's first make a pit stop to learn about
_entity-component-system_ (ECS) architecture.

In ECS architecture, a discrete event simulator subjects numerous _entities_ to
_systems_ that govern their lifecycles and mediate their interactions through
operations on their stateful _components_.

* An _entity_ is an opaque atom of identity. Typically devoid of any intrinsic
  properties, it can usually be represented with a simple integer.
* A _component_ ascribes a role to an entity and encapsulates any data
  necessary to model that role. Components may be affixed to entities
  permanently or transiently, and are usually maintained extrinsically, i.e.,
  mapped onto an entity through an external data structure. In a physics
  simulation, a "rigid body" component might adorn every entity that represents
  a physical object; the "rigid body" component could include state to model
  mass, linear drag, angular drag, and so forth.
* A _system_ embodies a process that acts only on entities that instantaneously
  possess some target combination of components. Systems can: inject entities
  into the simulation; delete entities from the simulation; attach components to
  entities; detach components from entities; modify the state inside components;
  manage global resources; and interface with other application modules.

ECS is common in video games and simulations, but works well whenever
applications are founded upon data-oriented design principles. It fits snugly
alongside other paradigms, like object-oriented or functional programming,
taking an orthogonal approach to solving related problems of structure and
composition.

### Bevy

Bevy is a data-driven game engine with a fast, flexible ECS. It's relatively
new, but it's also powerful and cross-platform, with support for 2D and 3D
render pipelines, scene persistence, cascading style sheets (CSS), and hot
reloading. Its build system permits fast recompilation, so you spend more time
testing than waiting. It also integrates smoothly with numerous popular crates,
like [Serde](https://crates.io/crates/serde) and
[egui](https://crates.io/crates/egui). We're barely going to scratch the surface
of what Bevy can do in this project.

Bevy's entities are
[generational&#32;indices](https://lucassardois.medium.com/generational-indices-guide-8e3c5f7fd594).
Its components are `struct`s and `enum`s: ordinary data types for which you can
implement the `Component` trait, which you typically do just by deriving
`Component`. Its systems are ordinary functions whose signatures are built-in up
from types that implement the `SystemParam` trait; these types are provided by
the Bevy framework, and many of them are generic over (your own) component
types.

If this is too abstract, don't worry. We'll put it together one piece at a time
with concrete examples. It's time to animate some elementary cellular automata!
