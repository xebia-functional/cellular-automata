use std::fmt::{Display, Formatter};
use std::fmt;
use std::ops::{Index, IndexMut};

use bevy::prelude::Resource;
use ringbuffer::{ConstGenericRingBuffer, RingBuffer};

////////////////////////////////////////////////////////////////////////////////
//                                   Rules.                                   //
////////////////////////////////////////////////////////////////////////////////

/// [AutomatonRule] represents the Wolfram code for the evolutionary rule
/// governing a 1-dimensional cellular automaton.
///
/// Under the [Wolfram&#32;coding] scheme, each of the 256 possible
/// 1-dimensional cellular automata are assigned a unique integer in `[0, 255]`.
/// The least significant bit (LSB) has ordinal `0` and the most significant bit
/// (MSB) has ordinal `7`. The binary representations of the `8` possible
/// ordinals themselves encode the possible neighborhood populations, such that
/// the MSB represents the left cell, the center bit represents the center cell,
/// and the LSB represents the right cell. If a bit `k` is clear in a Wolfram
/// code, it means that the population denoted by the corresponding ordinal `k`
/// produces a clear cell in the next generation; if `k` is set, then the cell
/// is set in the next generation.
///
/// To illustrate the ordinal encoding above, here is the table of
/// neighborhoods, as binary renditions of the ordinals themselves:
///
/// | Ordinal | Bit pattern / Occupancy of neighborhood |
/// | ------- | --------------------------------------- |
/// |    0    |                   000                   |
/// |    1    |                   001                   |
/// |    2    |                   010                   |
/// |    3    |                   011                   |
/// |    4    |                   100                   |
/// |    5    |                   101                   |
/// |    6    |                   110                   |
/// |    7    |                   111                   |
///
/// And here is an illustration of [Rule&#32;110] (= 0110 1110), which famously
/// supports universal computation:
///
/// | Neighborhood      | 111 | 110 | 101 | 100 | 011 | 010 | 001 | 000 |
/// | ----------------- | --- | --- | --- | --- | --- | --- | --- | --- |
/// | Next neighborhood |  0  |  1  |  1  |  0  |  1  |  1  |  1  |  0  |
///
/// [Wolfram&#32;coding]: https://en.wikipedia.org/wiki/Wolfram_code
/// [Rule&#32;110]: https://en.wikipedia.org/wiki/Rule_110
#[derive(Copy, Clone, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Resource)]
pub struct AutomatonRule(u8);

impl AutomatonRule
{
	/// Given a suitable population ordinal, index the Wolfram code to determine
	/// the occupancy of the successor of some unspecified corresponding cell.
	#[inline]
	const fn next_cell(self, ordinal: u8) -> bool
	{
		self.0 & (1 << ordinal) != 0
	}
}

impl From<u8> for AutomatonRule
{
	/// Given that [AutomatonRule] is a simple newtype, it feels natural to use
	/// `from` and `into` as constructors for this type.
	fn from(value: u8) -> Self
	{
		AutomatonRule(value)
	}
}

impl Display for AutomatonRule
{
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result
	{
		write!(f, "Rule #{}", self.0)
	}
}

////////////////////////////////////////////////////////////////////////////////
//                                 Automata.                                  //
////////////////////////////////////////////////////////////////////////////////

/// [Automaton] represents a [1-dimensional&#32;cellular&#32;automaton]. The
/// automaton itself is a sequence of cells, each represented by a `bool`, which
/// may be occupied (`true`) or vacant (`false`). The rightmost cell has the
/// index `0`, and the leftmost cell has the index `K-1`. A
/// [rule](AutomatonRule) may be applied to an automaton to produce the next
/// generation. `K` is the length of the automaton, in cells, and must be ≥3,
/// which sadly is unenforceable on the `stable` channel. Note that the two ends
/// of the automaton are considered adjacent for the purpose of computing the
/// next generation.
///
/// N.B.: Rust does not guarantee a packed representation for a `bool` array; in
/// fact, LLVM does not pack arrays of `u1` at this time, so the representation
/// will not be maximally efficient on space. It will still have relatively good
/// spatial and temporal performance, however, and this approach obviates the
/// need for any external crates, e.g.,
/// [`bitvec`](https://crates.io/crates/bitvec), and permits derivation of
/// [Copy].
///
/// [1-dimensional&#32;cellular&#32;automaton]: https://en.wikipedia.org/wiki/Elementary_cellular_automaton
#[derive(Copy, Clone, Debug)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub struct Automaton<const K: usize = AUTOMATON_LENGTH>([bool; K]);

impl<const K: usize> Automaton<K>
{
	/// Construct a new [Automaton] that is completely vacant, i.e., each cell
	/// is unoccupied.
	pub const fn new() -> Self
	{
		Self([false; K])
	}

	/// Compute the successor [automaton][Automaton] in accordance with the
	/// specified [rule](AutomatonRule).
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

	/// Answer an [iterator](Iterator) that traverse the cells of the
	/// [automaton](Automaton) in right-to-left order.
	pub fn iter(&self) -> impl Iterator<Item=&bool>
	{
		self.0.iter()
	}
}

/// Note that we cannot auto-derive [Default] because of the generic parameter,
/// so we manually implement it here.
impl<const K: usize> Default for Automaton<K>
{
	/// Construct a new [Automaton] that is completely vacant, i.e., each cell
	/// is unoccupied.
	fn default() -> Self
	{
		Self::new()
	}
}

impl<const K: usize> From<u64> for Automaton<K>
{
	/// Initialize an [automaton](Automaton) by treating the specified `u64` as
	/// a bit vector of up to 64 bits. Ignore high bits beyond index `K`.
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

impl<const K: usize> Display for Automaton<K>
{
	/// Render an automaton with a prefix that specifies its length followed by
	/// a densely-packed series of `X` and `•` that represent occupancy and
	/// vacancy, respectively.
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result
	{
		write!(f, "Automaton[{}]: ", K)?;
		for i in 0 ..= K - 1
		{
			write!(f, "{}", if self[i] { "X" } else { "•" })?;
		}
		Ok(())
	}
}

impl<const K: usize> Index<usize> for Automaton<K>
{
	type Output = bool;

	#[inline]
	fn index(&self, index: usize) -> &Self::Output
	{
		&self.0[index]
	}
}

impl<const K: usize> IndexMut<usize> for Automaton<K>
{
	#[inline]
	fn index_mut(&mut self, index: usize) -> &mut Self::Output
	{
		&mut self.0[index]
	}
}

////////////////////////////////////////////////////////////////////////////////
//                                 Histories.                                 //
////////////////////////////////////////////////////////////////////////////////

/// The last `N` generations of a [cellular&#32;automaton](Automaton). Each
/// automaton comprises `K` cells.
#[derive(Debug, Resource)]
pub struct History<
	const K: usize = AUTOMATON_LENGTH,
	const N: usize = AUTOMATON_HISTORY
>(
	ConstGenericRingBuffer<Automaton<K>, N>
);

impl<const K: usize, const N: usize> History<K, N>
{
	/// Construct an empty [History].
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

	/// Answer a reference to the [automaton](Automaton) that represents the
	/// newest generation.
	/// [default](Default::default)&#32;[automaton](Automaton).
	pub fn newest(&self) -> &Automaton<K>
	{
		self.0.back().unwrap()
	}

	/// Answer a reference to the [automaton](Automaton) that represents the
	/// oldest generation.
	#[allow(dead_code)]
	pub fn oldest(&self) -> &Automaton<K>
	{
		self.0.front().unwrap()
	}

	/// Replace the [newest](Self::newest)&#32;[automaton](Automaton) with the
	/// one provided. This is provided to support user customization of the
	/// seed.
	pub fn replace(&mut self, replacement: Automaton<K>)
	{
		match self.0.back_mut()
		{
			Some(newest) => *newest = replacement,
			None => self.0.push(replacement)
		}
	}

	/// Evolve the [newest](Self::newest)&#32;[automaton](Automaton) according
	/// to the specified [rule](AutomatonRule). Append the result to the
	/// [history](History). If the [history](History) is full, then the
	/// [oldest](Self::oldest)&#32;[automaton](Automaton) will be forgotten.
	pub fn evolve(&mut self, rule: AutomatonRule)
	{
		self.0.push(self.newest().next(rule));
	}

	/// Answer an iterator that traverses the [history](History) from
	/// [oldest](Self::oldest) to [newest](Self::newest).
	pub fn iter(&self) -> impl Iterator<Item=&Automaton<K>>
	{
		self.0.iter()
	}
}

impl<const K: usize, const N: usize> Default for History<K, N>
{
	fn default() -> Self
	{
		Self::new()
	}
}

impl<const K: usize, const N: usize> From<Automaton<K>> for History<K, N>
{
	/// Given a single [automaton](Automaton), start a new (history)[History]
	/// that uses the automaton as its first generation.
	fn from(value: Automaton<K>) -> Self
	{
		let mut history = Self::default();
		history.replace(value);
		history
	}
}

impl<const K: usize, const N: usize> Index<usize> for History<K, N>
{
	type Output = Automaton<K>;

	/// Borrow the `index`-th cell. `index` is zero-based.
	#[inline]
	fn index(&self, index: usize) -> &Self::Output
	{
		&self.0[index]
	}
}

impl<const K: usize, const N: usize> IndexMut<usize> for History<K, N>
{
	/// Mutably borrow the `index`-th cell. `index` is zero-based.
	#[inline]
	fn index_mut(&mut self, index: usize) -> &mut Self::Output
	{
		&mut self.0[index]
	}
}

////////////////////////////////////////////////////////////////////////////////
//                                 Utilities.                                 //
////////////////////////////////////////////////////////////////////////////////

/// Compute the population ordinal for some unspecified [rule](AutomatonRule)
/// based on the occupancy of the left, middle, and right cells of some
/// unspecified [automaton](Automaton). The result will be value in `[0,7]`.
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

////////////////////////////////////////////////////////////////////////////////
//                                 Constants.                                 //
////////////////////////////////////////////////////////////////////////////////

/// The length of all [cellular&#32;automata](Automaton) in this application.
pub const AUTOMATON_LENGTH: usize = 64;

/// The number of generations to preserve during the evolution of a
/// [cellular&#32;automaton](Automaton). This serves as the size of the
/// [RingBuffer] that supports the singleton [History].
pub const AUTOMATON_HISTORY: usize = 50;

////////////////////////////////////////////////////////////////////////////////
//                                   Tests.                                   //
////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod test
{
	use crate::automata::Automaton;
	#[cfg(doc)]
	use crate::automata::AutomatonRule;

	/// Use a well-known [cellular&32;automaton][Automaton] to verify correct
	/// construction of the second generation under
	/// [Rule&#32;#30](AutomatonRule).
	//noinspection SpellCheckingInspection
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

	/// Use a well-known [cellular&32;automaton][Automaton] to verify correct
	/// construction of the second generation under
	/// [Rule&#32;#110](AutomatonRule).
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
}
