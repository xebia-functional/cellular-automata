use bevy::prelude::App;
#[cfg(doc)]
use bevy::prelude::Resource;
use clap::Parser;
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
	let args = Arguments::parse();
	let rule = match args.rule
	{
		Some(rule) => AutomatonRule::from(rule),
		None => random::<u8>().into()
	};
	let seed = match args.seed
	{
		Some(seed) => Automaton::<AUTOMATON_LENGTH>::from(seed),
		None => random::<u64>().into()
	};
	println!("seed = {}", seed);
	println!("rule = {}", rule);
	App::new()
		.insert_resource(
			History::<AUTOMATON_LENGTH, AUTOMATON_HISTORY>::from(seed)
		)
		.insert_resource(rule)
		.add_plugins(AutomataPlugin)
		.run()
}

////////////////////////////////////////////////////////////////////////////////
//                          Command-line arguments.                           //
////////////////////////////////////////////////////////////////////////////////

/// The command line arguments.
#[derive(Parser, Debug)]
struct Arguments
{
	/// The [rule](AutomatonRule), specified as a Wolfram code in `[0,255]`.
	#[arg(short, long)]
	rule: Option<u8>,

	/// The [seed](Automaton), specified as a `u64` that represents the state of
	/// the initial generation.
	#[arg(short, long)]
	seed: Option<u64>
}
