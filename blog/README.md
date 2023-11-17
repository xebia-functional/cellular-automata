# Cellular Automata

In this post, we are going to bring elementary cellular automata to life using
the Rust programming language and the [Bevy](https://bevyengine.org/) game
engine. We'll learn a few things about cellular automata, Rust,
entity-component-system architecture, and basic game development.

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
than mere mathematical toys. Stanisław Ulam and John von Neumann discovered
cellular automate in the 1940s during their time together at Los Alamos National
Laboratory. In 1970, John Conway introduced the _Game of Life_, the
two-dimensional cellular automaton now famous for its blocks, beehives, boats,
blinkers, pulsars, gliders, and spaceships. In 1982, Conway published a proof of
Turing-completeness, putting the automaton on the same computational footing as
Turing machines and lambda calculus. Also in the 1980s, Stephen Wolfram
published _A New Kind of Science_, wherein he systematically studied _elementary
cellular automata_ — one-dimensional cellular automata of irreducible
simplicity. In 1985, Wolfram conjectured that one of these automata, called Rule
#110, was Turing-complete, and in 2004, Matthew Cook published the proof.

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
`X` to represent `on` and `•` to represent `off`, the eight possible
neighborhoods and their population ordinals are:

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
binary crate. When I present a code excerpt, I typically strip out any comments,
but you can see all the original comments intact in the project on GitHub.

The data model for the elementary cellular automaton is in
[`src/automata.rs`](../src/automata.rs), so all the code excerpts in this
section are sourced from that file.

Let's look at the representation of an elementary cellular automaton first.

### `Automaton`, _const_ genericity, and conditional derivation

Essentially, we keep our representational strategy super simple: we express an
elementary cellular automaton as a boolean array, albeit with a few frills.

```rust
#[derive(Copy, Clone, Debug)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub struct Automaton<const K: usize = AUTOMATON_LENGTH>([bool; K]);
```

Using the _newtype_ pattern, we define a 1-element tuple struct to wrap an
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
does. Extending this chain of reasoning to structs, tuples, and struct
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
correct. Because we're free to pick anything we want, we pick initial generation
`0x34244103` and rule #110. We also choose a shorter length automaton — 30 cells
instead of 64 — for convenience. This scenario is
[illustrated](https://commons.wikimedia.org/wiki/File:One-d-cellular-automaton-rule-110.gif#/media/File:One-d-cellular-automaton-rule-110.gif)
on Wikipedia, so we can treat it as a community-checked test vector.

```rust
#[test]
fn rule_110()
{
	//     XX•X••••X••X•••X•••••X••••••XX
	// 0b00110100001001000100000100000011
	// 0x   3   4   2   4   4   1   0   3
	let automaton = Automaton::<30>::from(0x34244103);
	//     •XXX•••XX•XX••XX••••XX•••••XX•
	// 0b00011100011011001100001100000110
	// 0x   1   C   6   C   C   3   0   6
	let expected = Automaton::<30>::from(0x1C6CC306);
	let actual = automaton.next(110.into());
	assert_eq!(expected, actual);
}
```

Our test suite includes a similar test for rule #30, just for safety, but I've
omitted it here for brevity.

### Ring buffers and `impl Trait` syntax

We want to visualize more than one generation, however, because otherwise we
can't experience any cool series or structures, like Sierpiński triangles. We
want to see how a cellular automaton evolves over time, at least over the course
of, say, fifty generations.

```rust
pub const AUTOMATON_HISTORY: usize = 50;
```

To avoid excessive memory retention or having to accommodate a scrollable
viewport, we bound the evolution to `AUTOMATON_HISTORY` generations. We should
keep only the most recent generations, of course, with the current generation at
the frontier. A ring buffer seems like the natural choice to satisfy these
goals.

The crate [`ringbuffer`](https://crates.io/crates/ringbuffer) provides a solid
implementation with a convenient API, so let's bring it into our project. We add
the following to our [`Cargo.toml`](../Cargo.toml):

```toml
[dependencies]
ringbuffer = "0.15.0"
```

Back in [`src/automata.rs`](../src/automata.rs), we introduce `History` as
another _const_-generic newtype, this one wrapping a `ConstGenericRingBuffer`:

```rust
#[derive(Debug, Resource)]
pub struct History<
	const K: usize = AUTOMATON_LENGTH,
	const N: usize = AUTOMATON_HISTORY
>(
	ConstGenericRingBuffer<Automaton<K>, N>
);
```

As strongly implied by its name, `ConstGenericRingBuffer` also uses _const_
genericity, so it fits perfectly with our strategy. Thinking ahead to our UI, we
need to present an entire history, with perhaps a single active generation at
the tail. We could special case the UI setup logic, but since the memory for the
ring buffer is already committed, it's cleaner to pre-populate `History` with
empty automata.

```rust
impl<const K: usize, const N: usize> History<K, N>
{
	pub fn new() -> Self
	{
		let mut ring = ConstGenericRingBuffer::new();
		for _ in 0 .. N
		{
			ring.push(Automaton::default());
		}
		assert!(ring.is_full());
		Self(ring)
	}
}
```

`ConstGenericRingBuffer::new` builds an empty ring buffer, and our loop fills it
with empty automata. Fullness of the ring buffer is an essential postcondition
for our use case, so we `assert!` it for sincerity. Now we can iterate through
the history, from `oldest` to `newest`, and know that fifty generations will occur.

```rust
impl<const K: usize, const N: usize> History<K, N>
{
	pub fn newest(&self) -> &Automaton<K> { self.0.back().unwrap() }
	pub fn oldest(&self) -> &Automaton<K> { self.0.front().unwrap() }
	pub fn iter(&self) -> impl Iterator<Item=&Automaton<K>> { self.0.iter() }
}
```

We can safely `unwrap` in `newest` and `oldest` because we have ensured
fullness, which is a stronger precondition than nonemptiness, which is all that
`unwrap` requires here.

But the most interesting thing here is the return type of `iter`, which employs
what Rust calls `impl Trait` syntax. The concrete iterator defined in the
`ringbuffer` crate is called `RingBufferIterator`, and it is private to the
crate. We can't name this type at all in our own code, meaning that we can
neither directly mention it in a method signature nor encapsulate it inside
another type. `impl Iterator` declares that we return some concrete type that
implements the trait `Iterator`, but keeps the identity of that type vague. It
allows the private type to remain hidden, even to us, but still allows us to
re-export that type anonymously from our own API. This kind of _ad hoc_
genericity is very useful at API boundaries, and more ergonomic than introducing
a named type parameter.

### Evolving the automaton

Evolving the cellular automaton is just a simple matter of using the data and
logic that we've already seen:

```rust
impl<const K: usize, const N: usize> History<K, N>
{
	pub fn evolve(&mut self, rule: AutomatonRule)
	{
		self.0.push(self.newest().next(rule));
	}
}
```

And that's more or less everything we need from the model, so it's time to leave
[`src/automata.rs`](../src/automata.rs) behind and turn our attention to the UI.

## Entity-component-system (ECS) architecture

We started with theory, moved to practice, and now it's time for some more
theory. Before we dive into using Bevy, let's first make a pit stop to learn
about _entity-component-system_ (ECS) architecture.

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

[Bevy](https://bevyengine.org/) is a data-driven game engine with a fast,
flexible ECS. It's relatively new, but it's also powerful and cross-platform,
with support for 2D and 3D render pipelines, scene persistence, cascading style
sheets (CSS), and hot reloading. Its build system permits fast recompilation, so
you spend more time testing than waiting. It also integrates smoothly with
numerous popular crates, like [Serde](https://crates.io/crates/serde) and
[egui](https://crates.io/crates/egui). We're barely going to scratch the surface
of what Bevy can do in this project.

Bevy's entities are
[generational&#32;indices](https://lucassardois.medium.com/generational-indices-guide-8e3c5f7fd594).
Its components are structs and enums: ordinary data types for which you can
implement the `Component` trait, which you typically do just by deriving
`Component`. Its systems are ordinary functions whose signatures are built-in up
from types that implement the `SystemParam` trait; these types are provided by
the Bevy framework, and many of them are generic over (your own) component
types.

If this is too abstract, don't worry. We'll put it together one piece at a time
with concrete examples.

### Setting up cross-platform Bevy

Let's get Bevy wired up for both native and web development and deployment. In
[`Cargo.toml`](../Cargo.toml), we add not one, but _two_ dependencies for Bevy.

```toml
[dependencies.bevy]
version = "0.12.0"

[target.'cfg(not(target_family = "wasm"))'.dependencies.bevy]
version = "0.12.0"
features = ["dynamic_linking"]
```

The first section brings Bevy into the project using the default set of
features, so long as there isn't an override for a more specific configuration.
Naturally, the second section is such an override; in particular, this override
enables dynamic linking of the Bevy crate, which speeds up your application
development cycle. Dynamic linking is only available for native targets, not
WebAssembly (WASM), hence the conditionality. 

Now we need to instruct Cargo to benefit from dynamic linking. In
[`.cargo/config.toml`](../.cargo/config.toml), we provide the various
platform-specific configurations.

```toml
[target.x86_64-unknown-linux-gnu]
linker = "clang"
rustflags = ["-Clink-arg=-fuse-ld=lld"]

[target.x86_64-apple-darwin]
rustflags = [
	"-C",
	"link-arg=-fuse-ld=/usr/local/opt/llvm/bin/ld64.lld"
]

[target.aarch64-apple-darwin]
rustflags = [
	"-C",
	"link-arg=-fuse-ld=/opt/homebrew/opt/llvm/bin/ld64.lld"
]

[target.x86_64-pc-windows-msvc]
linker = "rust-lld.exe"

[target.wasm32-unknown-unknown]
runner = "wasm-server-runner"
```

We support Linux, macOS, and Windows on x86; macOS on AArch64; and web on WASM.
As recommended by the folks behind Bevy, we use [`lld`](https://lld.llvm.org/),
the linker supplied with [LLVM](https://llvm.org/). You don't have to get `lld`,
but it's recommended for fastest link-time performance, which translates
directly into less time waiting for builds to complete. If you don't have `lld`
already and don't want to install it, you can just replace the paths to your
preferred linker.

The `runner` key in the WASM section at the end specifies a Cargo plugin,
[`wasm-server-runner`](https://github.com/jakobhellermann/wasm-server-runner),
that enables you to use `cargo run --target wasm32-unknown-unknown` to test a
WASM build. You can install it with `cargo install wasm-server-runner`.

It took a little bit of work, but Bevy is ready to go — on five platforms.

### Cross-platform program arguments

It would be nice to let the user set the initial conditions for the evolution.
There are two interesting configuration parameters:

1. The first generation of the cellular automaton.
2. The rule by which the cellular automaton will evolve from one generation to
   the next.

As we saw above, we can describe a cellular automaton with a `u64` and a rule
with a `u8`, and use `from` to obtain our model types. But there's enough
complexity to parsing command-line arguments that we want to delegate that
responsibility to a mature third-party crate:
[Clap](https://crates.io/crates/clap). Let's bring it into the project by adding
this to [`Cargo.toml`](../Cargo.toml):

```toml
[target.'cfg(not(target_family = "wasm"))'.dependencies.clap]
version = "4.4.8"
features = ["derive"]
```

Back in [`src/main.rs`](../src/main.rs), we bundle our configuration parameters
into a struct with a declarative strategy, letting Clap do the hard work for us:

```rust
/// Fun with cellular automata! Set the first generation with a known seed
/// and/or rule, or let the program choose randomly. Watch the automaton evolve,
/// and influence its evolution with the keyboard and mouse.
#[derive(Debug, Default)]
#[cfg_attr(not(target_family = "wasm"), derive(Parser))]
struct Arguments
{
	/// The rule, specified as a Wolfram code between 0 and 255, inclusive. If
	/// unspecified, the rule will be chosen randomly.
	#[cfg_attr(not(target_family = "wasm"), arg(short, long))]
	rule: Option<u8>,

	/// The first generation, specified as a 64-bit integer that represents the
	/// complete population. Lower numbered bits correspond to cells on the
	/// right of the visualization. If unspecified, the first generation will be
	/// chosen randomly.
	#[cfg_attr(not(target_family = "wasm"), arg(short, long))]
	seed: Option<u64>
}
```

There's some apparent complexity, so let's unpack:

* We derive `Debug` and `Default`, because both are handy.
* When the target isn't WASM, we derive `clap::Parser`, which will generate all
  the necessary boilerplate to parse our arguments from the command line. 
* When the target isn't WASM, we supply the `arg` attribute from the `Clap`
  crate. This primes the generated parser with a short form, long form, and
  description of the argument; the description is taken directly from the doc
  comment, which is why I included it in the excerpt. You want to avoid relying
  on any fancy Rustdoc formatting, because Clap will dump that formatting
  directly to standard output when the program is run with `--help`.
* `rule` and `seed` are both optional. We randomize whatever the user doesn't
  specify.
* Clap also emits the doc comment for the struct itself as the summary of the
  program, so the same caveats apply as above; keep it simple and address the
  user directly.

That handles the native case, but the web doesn't have command line arguments.
It does, however, have a query string comprising _search parameters_, which play
an analogous role to command-line arguments. We pop over to
[`Cargo.tml`](../Cargo.toml) one more time to register a conditional dependency
on [web-sys](https://crates.io/crates/web-sys):

```toml
[target.'cfg(target_family = "wasm")'.dependencies.web-sys]
version = "0.3.65"
features = ["Location", "Url", "UrlSearchParams"]
```

`web-sys` partitions the enormous web API using crate features. We need to
access the `Location`, `Url`, and `UrlSearchParams` types in order to build our
own simple search parameter parser, so we specify the eponymous features.

Oh, while we're still looking at the build file, we might as well do one more
thing. We promised randomization, so let's bring in the `rand` crate to handle
that. We'll insert it right before `ringbuffer`, to keep things alphabetized.

```toml
[dependencies]
rand = "0.8.5"
ringbuffer = "0.15.0"
```

We can implement the two cases now. Back over to [`src/main.rs`](../src/main.rs)
now! For native, we just wrap the result of Clap-generated `parse` in a `Some`:

```rust
#[cfg(not(target_family = "wasm"))]
fn arguments() -> Option<Arguments>
{
	Some(Arguments::parse())
}
```

For web, we do a little bit more work, but it closely tracks the web APIs:

```rust
#[cfg(target_family = "wasm")]
fn arguments() -> Option<Arguments>
{
	let href = web_sys::window()?.location().href().ok()?;
	let url = web_sys::Url::new(&href).ok()?;
	let params = url.search_params();
	let rule = params.get("rule").and_then(|rule| rule.parse().ok());
	let seed = params.get("seed").and_then(|seed| seed.parse().ok());
	Some(Arguments { rule, seed })
}
```

We called the function `arguments` in both cases, and exercised care to give it
the same signature, so we can use the same name and conventions to call it on
native and web.

### Giving control to Bevy

We've done bottom-up examination of the code so far, but now it's time to go top
down. Let's see how `main` initializes Bevy and hands over control to its engine
loop.

```rust
fn main()
{
	let args = arguments().unwrap_or(Arguments::default());
	let rule = args.rule
        .and_then(|rule| Some(AutomatonRule::from(rule)))
        .unwrap_or_else(|| random::<u8>().into());
    let seed = args.seed
		.and_then(|seed| Some(Automaton::<AUTOMATON_LENGTH>::from(seed)))
		.unwrap_or_else(|| random::<u64>().into());
    App::new()
		.insert_resource(
			History::<AUTOMATON_LENGTH, AUTOMATON_HISTORY>::from(seed)
		)
		.insert_resource(rule)
		.add_plugins(AutomataPlugin)
		.run();
}
```

Here we see the call to `arguments`, which binds to the correct implementation
based on compilation target. As promised, there's nothing special about the call
— it's a perfectly ordinary function call. If it fails for any reason, we plug
in the default `Arguments`, which will cause both the rule and first generation
to be randomized.

`App` is our gateway into the Bevy framework. We won't be referring to it
directly after initialization, but it holds onto the world, the runner, and the
plugins. The _world_ is the complete collection of system elements that compose
the application model. The _runner_ is the main loop that processes user input,
evolves the world over time, and controls rendering. And _plugins_ are
pre-packaged mini-worlds: collections of resources and systems that can be
reused across many projects.

A _resource_ is a global singleton with a unique type. Systems access resources
via dependency injection. We use `insert_resource` to register both the rule and
the history, making them available for dependency injection into our systems.
Anything that derives `Resource` can be used as a resource. If you were paying
close attention, you may have noticed that `AutomatonRule` and `History` both
derived `Resource` — and now you know why!

`AutomataPlugin` is the plugin that bundles together all of our other resources
and systems. We attach it via `add_plugins`. Finally, we call `run` to hand
control over to Bevy. From here on, the engine's main loop is responsible for
all execution.

### Modular composition with plugins

Perhaps surprisingly, our plugin is entirely stateless. Over in
[`src/ecs.rs`](../src/ecs.rs), where we're going to spend the rest of our time,
we see:

```rust
pub struct AutomataPlugin;
```

Stateless is fine, because we only care about the plugin's behavior, which is to
finish initializing the application. For that, we implement the `Plugin` trait:

```rust
impl Plugin for AutomataPlugin
{
    fn build(&self, app: &mut App)
    {
        let _seed = app.world.get_resource::<History>()
                .expect("History resource to be inserted already");
        let rule = app.world.get_resource::<AutomatonRule>()
                .expect("AutomatonRule resource to be inserted already");
        let mut window = Window {
            resolution: [1024.0, 768.0].into(),
            title: rule.to_string(),
            ..default()
        };
        set_title(&mut window, *rule);
        app
                .add_plugins(DefaultPlugins.set(WindowPlugin {
                    primary_window: Some(window),
                    ..default()
                }))
                .add_plugins(FrameTimeDiagnosticsPlugin)
                .insert_resource(EvolutionTimer::default())
                .insert_resource(AutomatonRuleBuilder::default())
                .add_systems(Startup, add_camera)
                .add_systems(Startup, build_ui)
                .add_systems(Update, maybe_toggle_instructions)
                .add_systems(Update, accept_digit)
                .add_systems(Update, maybe_show_fps)
                .add_systems(Update, maybe_toggle_cells)
                .add_systems(Update, update_next_rule)
                .add_systems(Update, maybe_change_rule)
                .add_systems(Update, evolve)
                .add_systems(Update, update_fps);
    }
}
```

There's no requirement for a plugin to be stateless, so `build` borrows both the
plugin and the `App`. We use the statically polymorphic `get_resource` to
extract the seed and the rule that we registered in `main`. Note that we pull
these resources using their static types only, which is why every resource needs
a unique static type. This is not a problem, because if we want to register,
say, 20 strings, we can wrap each in a disparate newtype first; newtypes have
zero runtime cost, and also provide better semantics, so this restriction guides
us toward better modeling decisions. We don't use the seed at all, but its
availability is an assertable precondition for installing our plugin, so we
extract it anyway.

We use the rule to set the title for the `Window`. On native systems, this
affects the title bar of the window. But in WASM, `Window` maps onto a
[canvas](https://html.spec.whatwg.org/#the-canvas-element), which doesn't have a
title bar. We'll need a cross-platform mechanism to handle this properly, so
we'll revisit this below.

`DefaultPlugins` aggregates the standard plugins that are widely useful across
most projects:

* `LogPlugin`, a logging plugin built on top of the popular
  [`tracing-subscriber`](https://crates.io/crates/tracing-subscriber) crate.
* `TaskPoolPlugin`, for managing task pools
* `TypeRegistrationPlugin`, which provides low-level support for the type-based
  resource registration that we saw above
* `FrameCountPlugin`, for counting frames
* `TimePlugin`, which adds support for discrete time and timers
* `TransformPlugin`, to enable entity placement and transformation
* `HierarchyPlugin`, for building component hierarchies
* `DiagnosticsPlugin`, for collecting various execution and performance metrics
* `InputPlugin`, which provides access to keyboard, mouse, and gamepad input
* `WindowPlugin`, for cross-platform windowing support
* `AccessibilityPlugin`, a plugin to manage and coordinate integrations with
  accessibility APIs

The `set` method on `DefaultPlugins` lets us replace one of these plugins. We
manually supply `window`, which we already created and customized, to serve as
the primary window for the application.

After adding the basic plugins, we insert two more resources, one to manage the
evolution rate and the other to buffer user input when entering a new rule.
Finally, we pour in all the systems that together define the behavior of our
application. Bevy groups systems together into predefined _schedules_. The
`Startup` schedule runs exactly one time, during initialization of the
application, so systems may be recorded here to perform some nonrecurring setup
logic. The `Update` schedule runs once per iteration of the engine loop.
`add_systems` associates a system with a schedule, and incorporates that
association into the world.

### Setting the window title

Before diving into the various systems, let's take a short detour to reach
catharsis about window titles. We abstracted the logic out to `set_title`, so
that we could specialize behavior differently for native and web.

The native implementation is quite trivial. We already have a `Window`, and the
`Window` has a title bar, so it's a simple matter of updating a field:

```rust
#[cfg(not(target_family = "wasm"))]
fn set_title(window: &mut Window, rule: AutomatonRule)
{
	window.title = rule.to_string();
}
```

The web implementation isn't too much harder, but it does require remembering
how web works. At the root of the namespace is `window`, which holds onto a
complete browser window. A window has a `document`, which is the root for the
page nodes, organized according to the web's Document Object Model (DOM). A
document has a title, which is displayed in the document tab or window title bar
(for rare non-tabbed browsers). `web-sys` models the web APIs closely, so we can
follow this chain of custody directly:

```rust
#[cfg(target_family = "wasm")]
fn set_title(_window: &mut Window, rule: AutomatonRule)
{
	web_sys::window().unwrap().document().unwrap().set_title(&rule.to_string());
}
```

`unwrap` is safe here because our host application is a web browser. Either call
would fail only for a headless host, like Node.js, where a graphical cellular
automaton simulator wouldn't even make sense.

### Camera

Bevy is a game engine that supports multiple scenes, so it serves a broader,
more general purpose than a run-of-the-mill UI toolkit. Entities don't just
appear in our window because we place them, we need to watch them through a
camera. If you don't add a camera, then you'll be staring at a pitch black
window.

```rust
fn add_camera(mut commands: Commands)
{
	commands.spawn(Camera2dBundle::default());
}
```

`add_camera` is a system that we added to the `Startup` schedule. Its argument,
`Commands`, is our interface to the Bevy command queue, which allows us to spawn
entities, add components to entities, remove components from entities, and
manage resources.

`spawn` creates a new entity with the specified component attached. The argument
can be anything, so long as it represents a bundle. A _bundle_ is just a batch
of components, and any component can be construed as a batch of one. In terms of
traits, `spawn` expects an implementation of the trait `Bundle`, and every type
that implements the trait `Component` also automatically implements the trait
`Bundle`. Bevy implements `Bundle` for tuples of components, so this makes it
handy to spawn an entity with multiple components attached.

`Camera2dBundle` aggregates the many components that together provide a view
onto some scene. The default instance provides an orthographic projection, so
lines and parallelism are preserved at the expense of distances and angles. For
our purposes, this ensures that all cells will appear congruent regardless of
their distance to the lens.

### The user interface

There are essentially four user interface elements in our application:

![All UI Elements Visible](All%20Ui%20Elements%20Visible.png)

* The most obvious is the grid of cells that represents the history of the
  cellular automaton. As mentioned before, each automaton comprises 64 cells,
  and we retain 50 generations. Each cell has a black border, and is filled
  black if it's "on" and white if it's "off". The bottom row represents the
  newest generation, so the generations scroll bottom-to-top over time as the
  evolution runs. We want to let the user toggle the newest generation between
  "on" and "off", so we use clickable buttons for the last row, and we fill the
  button with yellow when hovered, as a discoverable indication of
  interactivity. None of the other cells are interactive, so mere rectangles
  suffice.
* The semitransparent banner near the top of the grid contains abbreviated
  instructions to guide the user toward supported keyboard interactions. This
  banner is shown only when the simulator is paused. Naturally, the simulator
  begins paused so that the user can see the banner a learn a bit about what
  behaviors are supported.
* The semitransparent banner in the lower left shows the next rule that will
  run. This banner appears when the user presses a digit, either on the number
  row or on the number pad, remains on the screen while the user is typing
  additional digits, and disappears when the user is finished. So if the user
  types "121", the banner will first show "1", then "12", and finally "121". If
  the user types an invalid rule number, like "500", then the banner will show
  "Error".
* The semitransparent banner in the lower right shows the instantaneous _frames
  per second_ (FPS), which is the rendering rate for the graphical pipeline,
  i.e., how often the view redraws. The iteration rate for the engine loop is
  measured in _ticks per second_ (TPS), where a tick is a single iteration. Some
  game engines separate these two concepts, but Bevy ties them together
  directly, so $FPS = TPS$. FPS therefore gives us a coarse performance metric.
  This banner only appears while the user holds down the right shift key.

The `build_ui` system belongs to the `Startup` schedule. We are going to create
the UI elements only once, then mutate them in place from our systems. Only the
call graph rooted at `build_ui` will spawn entities, and these entities survive
until the application terminates.

```rust
fn build_ui(history: Res<History>, mut commands: Commands)
{
    commands
            .spawn(NodeBundle {
                style: Style {
                    height: Val::Percent(100.0),
                    width: Val::Percent(100.0),
                    ..default()
                },
                background_color: BackgroundColor(Color::DARK_GRAY),
                ..default()
            })
            .with_children(|builder| {
                build_history(builder, &history);
                build_instruction_banner(builder);
                build_next_rule_banner(builder);
                build_fps_banner(builder);
            });
}
```

This system also receives access to the command queue, but something new is
happening. `Res<History>` is the injection point for the `History` that we
registered as a resource. `Res` acts like an immutable borrow, here providing us
read access to the whole `History`. Just by including this parameter, Bevy knows
statically to inject the `History` that we registered. Of course, it's possible
to forget to register a resource; in this case, Bevy will panic at runtime prior
to invoking a system that requires the missing resource. You generally find such
problems immediately when you run the application, so it's not a big deal that
the registration check happens at runtime.

`build_ui` spawns an entity to represent the whole user interface. That entity
serves as the root of the containment hierarchy that includes the four major
elements mentioned above, each of which encompasses its own constituent
sub-elements. `NodeBundle` is the component type that serves as the basic UI
element. `Style` supports a sizable subset of the features of Cascading Style
Sheets (CSS), including [Flexbox](https://www.w3.org/TR/css-flexbox-1/) and
[Grid](https://www.w3.org/TR/css-grid-2/). Here we ensure that the element will
occupy all available space in the window.

Bevy passes a `ChildBuilder` to `with_children`, which permits hierarchical
composition of entities. We pass it into each of our subordinate UI element
builders.

#### History view

In build history, we lay out the grid that visualizes the evolution of our
cellular automaton over the last fifty generations:

```rust
fn build_history(builder: &mut ChildBuilder, history: &History)
{
	builder
		.spawn(NodeBundle {
			style: Style {
				display: Display::Grid,
				height: Val::Percent(100.0),
				width: Val::Percent(100.0),
				aspect_ratio: Some(1.0),
				padding: UiRect::all(Val::Px(24.0)),
				column_gap: Val::Px(1.0),
				row_gap: Val::Px(1.0),
				grid_template_columns: RepeatedGridTrack::flex(
					AUTOMATON_LENGTH as u16, 1.0),
				grid_template_rows: RepeatedGridTrack::flex(
					AUTOMATON_HISTORY as u16, 1.0),
				..default()
			},
			background_color: BackgroundColor(Color::DARK_GRAY),
			..default()
		})
		.with_children(|builder| {
			for (row, automaton) in history.iter().enumerate()
			{
				for (column, is_live) in automaton.iter().enumerate()
				{
					cell(builder, CellPosition { row, column }, *is_live);
				}
			}
		});
}
```

We use CSS Grid to ensure that the cells are uniformly sized. In the closure
passed to `with_children`, we iterate through the complete history to emit the
cells. `CellPosition` is a custom component:

```rust
#[derive(Copy, Clone, Debug, Component)]
struct CellPosition
{
	row: usize,
	column: usize
}
```

Just as deriving `Resource` is sufficient to permit a type's use as a resource,
deriving `Component` is sufficient to permit a type's use as a component. As the
placement loop illustrates, `row` increases top-to-bottom, while `column`
increases left-to-right.

```rust
fn cell(builder: &mut ChildBuilder, position: CellPosition, live: bool)
{
	builder
		.spawn(NodeBundle {
			style: Style {
				display: Display::Grid,
				padding: UiRect::all(Val::Px(2.0)),
				..default()
			},
			background_color: liveness_color(true),
			..default()
		})
		.with_children(|builder| {
			if position.is_active_automaton()
			{
				builder.spawn(
					(
						ButtonBundle {
							background_color: liveness_color(live),
							..default()
						},
						position
					)
				);
			} else {
				builder.spawn(
					(
						NodeBundle {
							background_color: liveness_color(live),
							..default()
						},
						position
					)
				);
			}
		});
}
```

We emit a visual cell with the eponymous `cell` function. We indulge in some
CSS Grid chicanery to surround our cell with a 2px border. `is_active_automaton`
answers `true` if and only if the row corresponds to the newest generation, so
we use it choose whether to attach a clickable `ButtonBundle` component or an
inactive `NodeBundle` component. We set the cell color with `liveness_color`,
which produces black for "on" cells and white for "off" cells.

If you look carefully, you'll see that `spawn` is receiving 2-tuples — the UI
bundle and our `CellPosition`. The resultant entity will have both components
attached. This will end up being important when we run the `evolve` system.

#### Instruction banner

Building the instruction banner is very similar, but contains a few new pieces:

```rust
fn build_instruction_banner(builder: &mut ChildBuilder)
{
	builder
		.spawn(
			(
				NodeBundle {
					style: Style {
						display: Display::Flex,
						position_type: PositionType::Absolute,
						height: Val::Px(50.0),
						width: Val::Percent(100.0),
						padding: UiRect::all(Val::Px(8.0)),
						top: Val::Px(50.0),
						justify_content: JustifyContent::Center,
						..default()
					},
					background_color: BackgroundColor(
						Color::rgba(0.0, 0.0, 0.0, 0.8)
					),
					..default()
				},
				Instructions
			)
		)
		.with_children(|builder| {
			builder.spawn(
				TextBundle::from_section(
					"[space] to resume/pause, [right shift] to \
						show FPS, or type a new rule",
					TextStyle {
						font_size: 28.0,
						color: LABEL_COLOR,
						..default()
					}
				)
					.with_style(Style {
						align_self: AlignSelf::Center,
						..default()
					})
			);
		});
}
```

Since we are creating an overlay, we use absolute positioning. We make the
background mostly opaque to provide enough contrast to read the instructional
text label. We attach a custom `Instructions` component to the overlay. This is
a stateless marker component that tags the overlay for easy access later.

```rust
#[derive(Component)]
struct Instructions;
```

Inside the overlay, we place a `TextBundle` that holds and styles the desired
text. A `TextBundle` comprises multiple sections, each of which sports different
text. This supports easy piecemeal substitution — your label can have static and
dynamic portions, and you just swap out the dynamic portions whenever they
change. Nothing needs to change in this label, however, so we employ but a
single section.

While there are several centering strategies that ought to have worked, there
are shortcomings in the CSS implementation, and I only found one strategy that
worked reliably in all cases:

* In the parent entity's `Style`, set `display` to `Display::Flex`.
* In the parent entity's `Style`, set `justify_content` to
  `JustifyContent::Center`.
* In the child entity's `TextBundle`'s `Style`, set `align_self` to
  `AlignSelf::Center`.

Save yourself some time and follow those steps if you want to center text in
Bevy!

#### Next-rule banner

The next-rule banner presents the buffered user input that contributes toward
ingestion of the next rule. It's so similar to `build_instruction_banner` that
we can ignore most of the code, focusing just on what's different:

```rust
fn build_next_rule_banner(builder: &mut ChildBuilder)
{
	builder
		.spawn(
			(
				NodeBundle {
					style: Style {
						display: Display::None,
						position_type: PositionType::Absolute,
						height: Val::Px(50.0),
						width: Val::Px(300.0),
						padding: UiRect::all(Val::Px(8.0)),
						bottom: Val::Px(50.0),
						left: Val::Px(50.0),
						..default()
					},
					background_color: BackgroundColor(
						Color::rgba(0.0, 0.0, 0.0, 0.8)
					),
					..default()
				},
				NextRule
			)
		)
		.with_children(|builder| {
			builder
				.spawn(
					(
						TextBundle::from_sections([
							TextSection::new(
								"Next up: ",
								TextStyle {
									font_size: 32.0,
									color: LABEL_COLOR,
									..default()
								},
							),
							TextSection::from_style(TextStyle {
								font_size: 32.0,
								color: LABEL_COLOR,
								..default()
							})
						]),
						NextRuleLabel
					)
				);
		});
}
```

We attach a custom `NextRule` component instead of an `Instructions` component,
but it serves the same purpose — to give this entity a systemic identity.

```rust
#[derive(Component)]
struct NextRuleLabel;
```

This time we hand an array of `TextSection`s to `TextBundle::from_sections`.
The first we treat as static text, and the second as dynamic. In particular, we
update the second section to show the currently buffered next rule. We attach
another custom marker component, `NextRuleLabel`, to the `TextBundle`.

```rust
#[derive(Component)]
struct NextRuleLabel;
```

#### FPS banner

The FPS banner is identical to the next-rule banner except for position,
specific text, and marker components. We substitute `"FPS: "` for `"Next up:"`,
the `Fps` component for the `NextRule` component, and the `FpsLabel` component
for the `NextRuleLabel` component.

```rust
#[derive(Component)]
struct Fps;

#[derive(Component)]
struct FpsLabel;

fn build_fps_banner(builder: &mut ChildBuilder)
{
	builder
		.spawn(
			(
				NodeBundle {
					style: Style {
						display: Display::None,
						position_type: PositionType::Absolute,
						height: Val::Px(50.0),
						width: Val::Px(200.0),
						padding: UiRect::all(Val::Px(8.0)),
						bottom: Val::Px(50.0),
						right: Val::Px(50.0),
						..default()
					},
					background_color: BackgroundColor(
						Color::rgba(0.0, 0.0, 0.0, 0.8)
					),
					..default()
				},
				Fps
			)
		)
		.with_children(|builder| {
			builder
				.spawn(
					(
						TextBundle::from_sections([
							TextSection::new(
								"FPS: ",
								TextStyle {
									font_size: 32.0,
									color: LABEL_COLOR,
									..default()
								},
							),
							TextSection::from_style(TextStyle {
								font_size: 32.0,
								color: LABEL_COLOR,
								..default()
							})
						]),
						FpsLabel
					)
				);
		});
}
```

### The evolver

Okay, we're done with setup! Now it's time for the exciting part: evolving the
cellular automaton!

```rust
fn evolve(
	time: Res<Time>,
	rule: Res<AutomatonRule>,
	mut timer: ResMut<EvolutionTimer>,
	mut history: ResMut<History>,
	mut cells: Query<(&CellPosition, &mut BackgroundColor)>
) {
	if timer.is_running()
	{
		timer.tick(time.delta(), || {
			history.evolve(*rule);
			for (position, mut color) in &mut cells
			{
				*color = liveness_color(history[*position]);
			}
		});
	}
}
```

We've seen `Res` already, so it's not hard to guess what `ResMut` is: a mutable
borrow of a resource, provided to our `evolve` system through dependency
injection.

`Time` is a clock resource supplied by Bevy. It tracks how much time has elapsed
since its creation or previous update, which is queryable via the `delta`
method. Bevy updates this value every frame, before running any systems in the
`Update` schedule.

Now for the magical part! `Query` is effectively an iterator over all entities
that possess the specified combination of components. So Bevy will pass every
entity that currently has both a `CellPosition` and a `BackgroundColor`. This
just so happens, by construction, to be the cells of our visual history.

So let's break the logic down into a narrative before we dive into the
individual pieces:

1. Check whether the `EvolutionTimer` is running. This is one of our project's
   resources. We saw its registration back in our plugin setup code, but we
   haven't investigated yet. We're going to kick that can a few more paragraphs
   down the post, but for the moment we note that the application starts paused
   and toggles between paused and running when the user presses the space bar.
2. Assume that the `EvolutionTimer` is running. Now manually tick it by the time
   elapsed since the last frame, which may cause the timer to expire.
3. Assume that the `EvolutionTimer` is expired. Now evolve the cellular
   automaton in accordance with the injected rule, then update the background
   color of every cell to agree with its new model state. Our `Query` iterator
   here just answers 2-tuples, so we destructure them to simplify the update
   process.

#### `EvolutionTimer` and manual ticking

You probably won't be surprised to discover that `EvolutionTimer` is just
another newtype around an existing Bevy struct:

```rust
#[derive(Resource)]
struct EvolutionTimer(Timer);
```

Bevy timers support a duration, a repeat mode (either `Once` or `Repeating`),
and a run mode (either running or paused). Like our own wrapper, a `Timer` must
be manually "ticked" via the `tick` method. This gives us total freedom about
how we couple any particular timer to the engine loop or wall clock. We
introduce a `new` method to configure an `EvolutionTimer` for our particular
circumstances.

```rust
const HEARTBEAT: Duration = Duration::from_millis(250);

impl EvolutionTimer
{
	fn new() -> Self
	{
		Self({
			let mut timer = Timer::new(HEARTBEAT, TimerMode::Repeating);
			timer.pause();
			timer
		})
	}
}
```

We use an inline block here to create a `Timer`, pause it, and embed it inside
an `EvolutionTimer`.

```rust
impl EvolutionTimer
{
	fn is_running(&self) -> bool
	{
		!self.0.paused()
	}

	#[inline]
	fn tick(&mut self, delta: Duration, on_expired: impl FnOnce())
	{
		self.0.tick(delta);
		if self.0.finished()
		{
			on_expired();
		}
	}
}
```

Now we see that `is_running` just tests whether the internal Bevy timer is
paused. Calling `tick` just delegates to the eponymous method of the internal
timer and conditions execution of the supplied closure on the internal timer
having exhausted its allotted duration.

### Toggling the `EvolutionTimer`

Toggling the `EvolutionTimer` is pretty straightforward, so there's really
nothing interesting to say about it:

```rust
impl EvolutionTimer
{
	fn toggle(&mut self)
	{
		match self.0.paused()
		{
			true => self.0.unpause(),
			false => self.0.pause()
		}
	}
}
```

Enabling the user to toggle the `EvolutionTimer` is much more interesting:

```rust
fn maybe_toggle_instructions(
	keys: Res<Input<KeyCode>>,
	mut instructions: Query<&mut Style, With<Instructions>>,
	mut timer: ResMut<EvolutionTimer>
) {
	if keys.just_pressed(KeyCode::Space)
	{
		timer.toggle();
		let style = &mut instructions.single_mut();
		style.display = match style.display
		{
			Display::Flex => Display::None,
			Display::None => Display::Flex,
			Display::Grid => unreachable!()
		};
	}
}
```

There are two new things in the signature:

1. `Input` is a system resource that provides access to "press-able" input, like
   a keyboard, mouse button, or gamepad. The type parameter, `KeyCode`,
   corresponds to cooked keyboard input. I say "cooked" because these are not
   the raw keycodes obtained from the operating system, but rather Bevy's
   cross-platform abstractions of key events. We can ask an `Input` whether it
   is "just pressed" (since the last frame), "pressed" (over the course of
   multiple frames), or "just released" (since the last frame).
2. `With` acts a positive filter for a `Query`: we want every entity that has
   both a `Style` component and an `Instructions` component, but we don't need
   access to the `Instructions` component itself. Recall that `Instructions` was
   an empty marker struct, so there wouldn't be any point to binding it locally,
   because there's nothing to read and nothing to write.

The code itself is simple. When the user presses the space bar, toggle the
`EvolutionTimer`, extract the only `Style` from the singleton query, and toggle
the `display` property between `Flex` and `None`. How do we know that there's
only one `Style`? Construction: we only branded one entity with the
`Instructions` label. `single_mut` will panic if you get this wrong, so it has
the side benefit of acting as an assertion. Speaking of assertions, we call
`unreachable!` if the old `display` is `Grid`, because this should be impossible
from the structure of our code.

### Toggling cell state by mouse

When the simulation is paused, we let the user toggle the cells at the bottom of
the grid — the ones that represent the newest generation — by clicking on them.

```rust
const PRESSED_COLOR: Color = Color::YELLOW;

fn maybe_toggle_cells(
	timer: ResMut<EvolutionTimer>,
	mut history: ResMut<History>,
	mut interaction: Query<
		(&Interaction, &CellPosition, &mut BackgroundColor),
		(Changed<Interaction>, With<Button>)
	>
) {
	if !timer.is_running()
	{
		for (interaction, position, mut color) in &mut interaction
		{
			match *interaction
			{
				Interaction::Pressed =>
				{
					let cell = &mut history[*position];
					*cell = !*cell;
					*color = liveness_color(*cell);
				},
				Interaction::Hovered =>
				{
					*color = BackgroundColor(PRESSED_COLOR);
				},
				Interaction::None =>
				{
					*color = liveness_color(history[*position]);
				}
			}
		}
	}
}
```

We have a much more complex `Query` here, so let's analyze it. There are two
type parameters, each a tuple. The first aggregates the target components; the
second aggregates additional filters. `Interaction` abstracts the kind of
interaction that occurred on some UI node. `Changed` excludes entities for which
the `Interaction` did not change since the last time that this system ran.
`With<Button>` ensures that only entities with the `Button` component will
appear during iteration.

Now for the logic. We ignore mouse input when the timer is running; when the
timer is running, _the simulation is running_, and we don't want to inflict upon
the user the frustrating experience of racing against the evolver to click the
cells. We loop over all the interactions, taking appropriate action when the UI
element is pressed, hovered, or vacated (`Interaction::None`). Respectively,
this amounts to: toggling the cell between "on" and "off"; showing the bright
yellow interactivity indicator; and restoring the cell to its usual state
indicator upon un-hovering.

Note that we never explicitly mention the specific device here, because
`Interaction` is from the UI element's point of view, not the user's or the
device's. That technically makes this mechanism device-agnostic, even though the
device will usually be a pointing device like a mouse or trackpad.

### Buffering a new rule

In most mathematical exercises, a cellular automaton evolves according to a
single fixed rule established at the onset. But it makes for a much more
appealing visual simulation to allow the user to change the rule midstream.

Before we look at the systems involved, let's look at the data structure that
supports them. It's called `AutomatonRuleBuilder`. We added it as a resource
from our plugin, but glossed over it completely. It's time to put it front and
center.

```rust
#[derive(Default, Resource)]
struct AutomatonRuleBuilder
{
	builder: Option<String>,
	timer: Option<Timer>
}
```

`builder` is the compositional buffer for the textual version of a user-typed
rule. It transitions from `None` to `Some` when the user types the first digit
of the new rule, either through the number row or the number pad. It transitions
back to `None` when either the paired `timer` expires or an invalid rule is
detected.

`timer` controls the pacing of data entry. The timer also transitions from
`None` to `Some` when the user types the first digit. The timer is one-shot, but
resets whenever the user enters another digit. When the timer expires, the
content of the buffer is treated as complete and parsed as an `AutomatonRule`.

Following the same pattern that we've seen several times now, we also manually
tick the data entry timer:

```rust
impl AutomatonRuleBuilder
{
	/// Update the [timer](Self::timer) by the specified [duration](Duration).
	#[inline]
	fn tick(&mut self, delta: Duration)
	{
		if let Some(ref mut timer) = self.timer
		{
			timer.tick(delta);
		}
	}
}
```

We only allow digits to be accumulated into the buffer:

```rust
const RULE_ENTRY_GRACE: Duration = Duration::from_millis(600);

impl AutomatonRuleBuilder
{
	fn push_digit(&mut self, c: char)
	{
		assert!(c.is_digit(10));
		match self.builder
		{
			None =>
			{
				self.builder = Some(c.into());
				self.timer = Some(
					Timer::new(RULE_ENTRY_GRACE, TimerMode::Once)
				);
			},
			Some(ref mut builder) if builder.len() < 3 =>
			{
				builder.push(c);
				self.timer.as_mut().unwrap().reset();
			},
			Some(_) =>
			{
				self.builder = None;
				self.timer = None;
			}
		}
	}
}
```

The caller is responsible for giving `push_digit` an actual digit, so we assert
the precondition. We install a buffer and timer if they didn't exist already,
meaning that this is the first digit that we've seen ever or since the last
proposed new rule was accepted or rejected. If the buffer could possibly be
valid after the new digit is appended, then push it and reset the timer to give
the user a full quantum (600ms) to type the next digit. If the buffer would be
_prima facie_ invalid after the push, i.e., because every valid parse would
exceed the upper bound for a Wolfram code, then destroy the buffer and the timer
immediately.

```rust
impl AutomatonRuleBuilder
{
    fn buffered_input(&self) -> Option<&str> { self.builder.as_deref() }
}
```

To access the buffered input directly, we call `buffered_input`. We can chain
`is_some` to turn this into a simple check for the existence of buffered data.

Finally, we need to extract the successor rule from an `AutomatonRuleBuilder`:

```rust
impl AutomatonRuleBuilder
{
	fn new_rule(&mut self) -> Option<AutomatonRule>
	{
		match self.timer
		{
			Some(ref timer) if timer.just_finished() =>
			{
				let rule = match self.builder.as_ref().unwrap().parse::<u8>()
				{
					Ok(rule) => Some(AutomatonRule::from(rule)),
					Err(_) => None
				};
				self.builder = None;
				self.timer = None;
				rule
			}
			_ => None
		}
	}
}
```

We only want to parse the buffered data if the timer has just expired. We return
`Some` only if the parse succeeds, destroying the builder and timer on our way
out.

### Typing a new rule

With the supporting model in place, we can now confidently proceed to an
examination of the systems involved. First is `accept_digit`, which translates
keyboard presses into buffered digits:

```rust
fn accept_digit(
	keys: Res<Input<KeyCode>>,
	mut builder: ResMut<AutomatonRuleBuilder>,
	mut next_rule: Query<&mut Style, With<NextRule>>
) {
	for key in keys.get_just_pressed()
	{
		match key.to_digit()
		{
			Some(digit) => builder.push_digit(digit),
			None => {}
		}
	}
	let style = &mut next_rule.single_mut();
	style.display =
		if builder.buffered_input().is_some() { Display::Flex }
		else { Display::None };
}
```

Nothing new in the signature and nothing terribly novel in the body. In short,
we append a digit to the buffer and toggle the visibility of the overlay. We
turn our attention momentarily to `to_digit`, because it involves the only trait
that we are introducing in the whole project:

```rust
const NUMBER_ROW_RANGE: RangeInclusive<u32> =
    KeyCode::Key1 as u32 ..= KeyCode::Key0 as u32;

const NUMPAD_RANGE: RangeInclusive<u32> =
    KeyCode::Numpad0 as u32 ..= KeyCode::Numpad9 as u32;

trait ToDigit: Copy
{
	fn to_digit(self) -> Option<char>;
}

impl ToDigit for KeyCode
{
	fn to_digit(self) -> Option<char>
	{
		if NUMBER_ROW_RANGE.contains(&(self as u32))
		{
			match self
			{
				KeyCode::Key0 => Some('0'),
				key => Some(char::from_digit(key as u32 + 1, 10).unwrap())
			}
		}
		else if NUMPAD_RANGE.contains(&(self as u32))
		{
			let delta = self as u32 - KeyCode::Numpad0 as u32;
			Some(char::from_digit(delta, 10).unwrap())
		}
		else
		{
			None
		}
	}
}
```

The translation code is slightly yucky, which is why we hide it in a utility
trait outside the main narrative. `Key0` comes after `Key9`, just as it appears
on the number row of the keyboard, hence the slightly weird logic in the
`match`. But `Numpad0` comes before `Numpad1`, so that case is simpler. We
leverage the fact that `KeyCode` is an enum with `repr(u8)` to allow us to
perform arithmetic on the enum discriminants. Finally, we use `char::from_digit`
to obtain the print character that corresponds with the key press.

We also need to change the label inside the now overlay if and only if it's
visible:

```rust
fn update_next_rule(
	builder: Res<AutomatonRuleBuilder>,
	mut next_rule: Query<&mut Text, With<NextRuleLabel>>
) {
	let buffered_input = builder.buffered_input();
	if buffered_input.is_some()
	{
		let text = &mut next_rule.single_mut();
		text.sections[1].value = match builder.buffered_input()
		{
			Some(rule) if rule.parse::<u8>().is_ok() => rule.to_string(),
			_ => "Error".to_string()
		};
	}
}
```

We receive the sectioned `Text` via the `Query` so that we can update the
dynamic section at index `1`. We only perform the update if some digits are
buffered. If parsing the buffered data as a Wolfram code fails, then transiently
update the label to "Error".

Finally, using a third system, we actually update the `AutomatonRule` resource:

```rust
fn maybe_change_rule(
	time: Res<Time>,
	mut rule: ResMut<AutomatonRule>,
	mut builder: ResMut<AutomatonRuleBuilder>,
	mut query: Query<&mut Window>
) {
	builder.tick(time.delta());
	match builder.new_rule()
	{
		Some(new_rule) =>
		{
			*rule = new_rule;
			let window = &mut query.single_mut();
			set_title(window.as_mut(), *rule);
		},
		None => {}
	}
}
```

We manually tick the `AutomatonRuleBuilder`'s clock, then attempt to extract a
new rule from the buffered input. We have access to the lone `Window`, so we can
update the title to reflect a new rule; by using our cross-platform `set_title`,
we ensure that this works both on native and the web.

### Reporting FPS

Almost done! All that's left is to display the instantaneous frames per second
while the user holds down the right shift key:

```rust
fn maybe_show_fps(
	keys: Res<Input<KeyCode>>,
	mut fps: Query<&mut Style, With<Fps>>
) {
	let style = &mut fps.single_mut();
	style.display = match keys.pressed(KeyCode::ShiftRight)
	{
		true => Display::Flex,
		false => Display::None
	};
}
```

We've already talked briefly about the only otherwise new thing here. `pressed`
continues to answer `true` for as long as the right shift key remains pressed,
so the display will become `Flex` when the user presses the key and stay `Flex`
until the user releases the key. This links the visibility of the overlay
directly to the duration of that key press.

```rust
fn update_fps(
	diagnostics: Res<DiagnosticsStore>,
	mut fps: Query<&mut Text, With<FpsLabel>>
) {
    let text = &mut fps.single_mut();
    let fps = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS).unwrap();
    if let Some(value) = fps.smoothed()
    {
        text.sections[1].value = format!("{:.2}", value);
    }
}
```

`DiagnosticsStore` is our gateway to the various diagnostics that Bevy maintains
over the lifetime of our application. We added the `FrameTimeDiagnosticsPlugin`
so that the frame rate would also be available through the `DiagnosticsStore`.
We extract the desired `Diagnostic` by id, which in this case is the aptly named
`FPS`. `smoothed` fetches the precomputed
[exponentially&#32;weighted&#32;moving&#32;average](https://en.wikipedia.org/wiki/Moving_average#Exponential_moving_average)
(EWMA) for the diagnostic. We restrict the result to two decimal places and
update the dynamic section of the text at index `1`.

## Conclusion

Well, as they say these days, _that was a lot_. We covered some math history,
computer science, Rust programming, and elementary game development. Hopefully,
this is the beginning of a journey, not the end; merely a whetting of your
appetite to build awesomely with Rust. There are so many more topics and crates,
and so much more to do with Bevy alone. Whether you reached the end of this long
post or not, thanks for your time, and happy coding!
