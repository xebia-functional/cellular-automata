use bevy::prelude::App;
#[cfg(doc)]
use bevy::prelude::Resource;
use rand::random;

use crate::automata::{
	Automaton, AUTOMATON_HISTORY, AUTOMATON_LENGTH, AutomatonRule,
	History
};
use crate::ecs::AutomataPlugin;

mod automata;
mod ecs;

/// The entry point for the whole application. Parse the
/// [command&#32;line&#32;arguments](Arguments), attach them to the [App] as
/// [resources](Resource), then hand control over to Bevy.
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

////////////////////////////////////////////////////////////////////////////////
//                             Program arguments.                             //
////////////////////////////////////////////////////////////////////////////////

#[cfg(not(target_family = "wasm"))]
use clap::Parser;

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

////////////////////////////////////////////////////////////////////////////////
//                         Reading program arguments.                         //
////////////////////////////////////////////////////////////////////////////////

/// Read the program [arguments](Arguments) from the command line. Available for
/// native builds only.
#[cfg(not(target_family = "wasm"))]
fn arguments() -> Option<Arguments>
{
	Some(Arguments::parse())
}

/// Read the program [arguments](Arguments) from the search parameters within
/// the query string. Available for WASM builds only.
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
