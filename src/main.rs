use bevy::app::App;
use clap::Parser;
use rand::random;

use crate::automata::{
	Automaton, AUTOMATON_HISTORY, AUTOMATON_LENGTH, AutomatonRule,
	History
};
use crate::ecs::AutomataPlugin;

mod automata;
mod ecs;

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
	/// The [rule](AutomatonRule].
	#[arg(short, long)]
	rule: Option<u8>,

	/// The initial [seed](Automaton).
	#[arg(short, long)]
	seed: Option<u64>
}
