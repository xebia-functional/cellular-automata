use std::fmt;
use std::fmt::Formatter;
use std::ops::{Index, IndexMut, RangeInclusive};
use std::time::Duration;

use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::{
	AlignSelf, App,
	BackgroundColor, BuildChildren, Button, ButtonBundle,
	Camera2dBundle, Changed, ChildBuilder, Color, Commands, Component,
	default, DefaultPlugins, Display,
	Input, Interaction,
	KeyCode,
	NodeBundle,
	Plugin, PluginGroup, PositionType,
	Query,
	Res, ResMut, Resource,
	Startup, Style,
	Text, TextBundle, TextSection, TextStyle, Time, Timer,
	UiRect, Update,
	Val,
	Window, WindowPlugin, With
};
use bevy::time::TimerMode;
use bevy::ui::{JustifyContent, RepeatedGridTrack};

use crate::automata::{
	AUTOMATON_HISTORY, AUTOMATON_LENGTH, AutomatonRule,
	History
};
#[cfg(doc)]
use crate::automata::Automaton;

////////////////////////////////////////////////////////////////////////////////
//                                  Plugins.                                  //
////////////////////////////////////////////////////////////////////////////////

/// The [plugin](Plugin) responsible for managing our
/// [evolutionary&#32;system](evolve).
pub struct AutomataPlugin;

impl Plugin for AutomataPlugin
{
	/// The initial [seed](Automaton) and [rule](AutomatonRule) must already
	/// have been set.
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

////////////////////////////////////////////////////////////////////////////////
//                                 Resources.                                 //
////////////////////////////////////////////////////////////////////////////////

/// A repeating [timer](Timer) timer that controls the [evolution][evolve] rate
/// of the [automaton](Automaton).
#[derive(Resource)]
struct EvolutionTimer(Timer);

impl EvolutionTimer
{
	/// Create a new [EvolutionTimer] in the paused state.
	fn new() -> Self
	{
		Self({
			let mut timer = Timer::new(HEARTBEAT, TimerMode::Repeating);
			timer.pause();
			timer
		})
	}

	/// Determine whether the [timer](Timer) is running.
	fn is_running(&self) -> bool
	{
		!self.0.paused()
	}

	/// Update the timer by the specified [duration](Duration). If the timer has
	/// expired, then run the specified function.
	#[inline]
	fn tick(&mut self, delta: Duration, on_expired: impl FnOnce())
	{
		self.0.tick(delta);
		if self.0.finished()
		{
			on_expired();
		}
	}

	/// Toggle the execution state of the [timer](Timer), between paused and
	/// unpaused.
	fn toggle(&mut self)
	{
		match self.0.paused()
		{
			true => self.0.unpause(),
			false => self.0.pause()
		}
	}
}

impl Default for EvolutionTimer
{
	#[inline]
	fn default() -> Self
	{
		Self::new()
	}
}

/// State management for a user-driven [rule](AutomatonRule) change.
#[derive(Default, Resource)]
struct AutomatonRuleBuilder
{
	/// The string buffer for constructing the next [rule](AutomatonRule) from
	/// user input. Transitions from [None] to [Some] when the first digit is
	/// submitted. Transitions from [Some] to [None] when either (1) the
	/// [timer](Timer) expires or (2) an invalid [rule](AutomatonRule) is
	/// detected.
	builder: Option<String>,

	/// The [timer](Timer) that controls user entry of the digits of the next
	/// [rule](AutomatonRule). While this timer is running, the user may press
	/// the various numeric keys on their keyboard to submit another digit to
	/// the [builder](Self::builder).
	timer: Option<Timer>
}

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

	/// Append a digit onto the [builder](AutomatonRuleBuilder). Reset the
	/// [timer](Timer) between successive digits.
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
				// If too many digits were entered, then rule conversion will
				// definitely fail. Bail early, to avoid buffering too much
				// bogus input.
				self.builder = None;
				self.timer = None;
			}
		}
	}

	/// Answer the buffered input, if any.
	fn buffered_input(&self) -> Option<&str>
	{
		self.builder.as_deref()
	}

	/// Attempt to decode a [rule](AutomatonRule) from the input supplied thus
	/// far, but only if the [timer](Timer) has recently expired.
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

////////////////////////////////////////////////////////////////////////////////
//                                Components.                                 //
////////////////////////////////////////////////////////////////////////////////

/// The coordinates of some cell in the grid that renders the
/// [history](History). A [CellPosition] can serve as an [index](Index) into a
/// [history](History).
#[derive(Copy, Clone, Debug, Component)]
struct CellPosition
{
	/// The row coordinate for this cell, advancing from the
	/// [oldest](History::oldest) generation to the [newest](History::newest)
	/// generation.
	row: usize,

	/// The column coordinate for this cell, advancing from left to right. Note
	/// that this is _against_ the natural order of an [automaton](Automaton).
	column: usize
}

impl CellPosition
{
	/// Determine whether the receiver represents the [newest](History::newest)
	/// generation.
	fn is_active_automaton(&self) -> bool
	{
		self.row == AUTOMATON_HISTORY - 1
	}
}

impl fmt::Display for CellPosition
{
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result
	{
		write!(f, "({},{})", self.column, self.row)
	}
}

impl<const K: usize, const N: usize> Index<CellPosition> for History<K, N>
{
	type Output = bool;

	/// Visually, treat the automaton as though its `0` index occurs at the
	/// right edge.
	fn index(&self, index: CellPosition) -> &Self::Output
	{
		&self[index.row][K - index.column - 1]
	}
}

impl<const K: usize, const N: usize> IndexMut<CellPosition> for History<K, N>
{
	/// Visually, treat the automaton as though its `0` index occurs at the
	/// right edge.
	fn index_mut(&mut self, index: CellPosition) -> &mut Self::Output
	{
		&mut self[index.row][K - index.column - 1]
	}
}

/// The overlay that displays instructions to the user. The overlay is only
/// displayed when the evolver is paused. Since the evolver begins
/// paused, however, the user always has an upfront chance to review the
/// instructions.
#[derive(Component)]
struct Instructions;

/// The overlay that displays the partial next [rule](AutomatonRule), assuming
/// that the user is actively entering a new rule.
#[derive(Component)]
struct NextRule;

/// The label that displays the partial next rule.
#[derive(Component)]
struct NextRuleLabel;

/// The overlay that shows the instantaneous frames per second (FPS). This is a
/// debugging feature, available when the user is holding down the right shift
/// key.
#[derive(Component)]
struct Fps;

/// The label that shows the instantaneous frames per second (FPS). It resides
/// within a simple overlay, marked by [Fps].
#[derive(Component)]
struct FpsLabel;

////////////////////////////////////////////////////////////////////////////////
//                              Startup systems.                              //
////////////////////////////////////////////////////////////////////////////////

/// Add a camera to the scene, so that we can observe the [evolution](History)
/// of the [automaton](Automaton).
fn add_camera(mut commands: Commands)
{
	commands.spawn(Camera2dBundle::default());
}

/// Build the complete user interface:
///
/// * A grid representing the [history](History).
/// * An instructional banner, displayed when the evolver is paused.
/// * A rule buffer banner, displayed while the user is entering a new rule.
/// * An FPS banner, displayed while the user holds the right shift key.
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

////////////////////////////////////////////////////////////////////////////////
//                              Update systems.                               //
////////////////////////////////////////////////////////////////////////////////

/// On space, toggle the run state and the visibility of the instructional
/// overlay.
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

/// On digit, append the digit to the [AutomatonRuleBuilder].
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

/// When right shift is held, display the frames per second (FPS).
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

/// Handle toggling of the cells in the latest generation.
///
/// * On press of an active cell _while paused_, toggle the cell.
/// * On hover of an active cell _while paused_, highlight the button to
///   indicate interactivity.
/// * On un-hover of an active cell _while paused_, restore the button's
///   original [liveness&#32;color](liveness_color).
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

/// Update the next [rule](AutomatonRule) label.
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

/// Change the [rule](AutomatonRule) for future [evolutions](evolve), if another
/// [rule](AutomatonRule) is pending. Update the window title to reflect the new
/// [rule](AutomatonRule).
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

/// [Evolve](History::evolve) the [automaton](Automaton), and update the visual
/// [history](History).
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
			// Run the evolver one step.
			history.evolve(*rule);

			// Update each of the cells to reflect its new state in the model.
			for (position, mut color) in &mut cells
			{
				*color = liveness_color(history[*position]);
			}
		});
	}
}

/// Update the frames per second (FPS) label.
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

////////////////////////////////////////////////////////////////////////////////
//                              User interface.                               //
////////////////////////////////////////////////////////////////////////////////

/// Build the grid that corresponds to the [history](History).
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

/// Add a visual cell to the component whose [builder](ChildBuilder) is
/// specified, attaching the specified [position](CellPosition) as a
/// [component](Component). Render a live cell with [LIVE_COLOR]. Render a dead
/// cell with [DEAD_COLOR]. Use [LIVE_COLOR] to paint a border around the cell.
/// If the [position](CellPosition) designates the [newest](History::newest)
/// generation, then emit clickable buttons instead of colorful rectangles.
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
			}
			else
			{
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

/// Answer the appropriate [BackgroundColor] for the specified cell liveness,
/// rendering a live cell with [LIVE_COLOR] and a dead cell with [DEAD_COLOR].
#[inline]
fn liveness_color(live: bool) -> BackgroundColor
{
	BackgroundColor(if live { LIVE_COLOR } else { DEAD_COLOR })
}

/// Create a transparent overlay that is visible when the evolver is paused.
/// Note that centering text is particularly hard, and all of the online
/// examples I could find were wrong, so here are the salient points:
///
/// * Set `display` to `Display::Flex` in the parent.
/// * Set `justify_content` to `JustifyContent::Center` in the parent.
/// * Set `align_self` to `AlignSelf::Center` in the `style` of the `TextBundle`
///   itself.
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

/// Create a label that displays the next rule to run, but only if such a rule
/// is actively being input. Place it in the lower left.
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

/// Create an FPS label that displays only when the player holds right shift.
/// Place it in the lower right.
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

/// Set the title of the window to show the active [rule](AutomatonRule).
#[cfg(not(target_family = "wasm"))]
fn set_title(window: &mut Window, rule: AutomatonRule)
{
	window.title = rule.to_string();
}

/// Set the title of the window to show the active [rule](AutomatonRule). The
/// Bevy window is not wired to the browser, so it doesn't have a title bar.
/// Tell the document to update its label instead.
#[cfg(target_family = "wasm")]
fn set_title(_window: &mut Window, rule: AutomatonRule)
{
	web_sys::window().unwrap().document().unwrap().set_title(&rule.to_string());
}

////////////////////////////////////////////////////////////////////////////////
//                                 Utilities.                                 //
////////////////////////////////////////////////////////////////////////////////

/// Contract for value conversion to a digit character.
trait ToDigit: Copy
{
	/// Convert the receiver into a digit character.
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

////////////////////////////////////////////////////////////////////////////////
//                                 Constants.                                 //
////////////////////////////////////////////////////////////////////////////////

/// The heartbeat for a running [evolution&#32;system](evolve).
const HEARTBEAT: Duration = Duration::from_millis(250);

/// How long to delay between digit submissions before accepting the input so
/// far as the next [rule](AutomatonRule).
const RULE_ENTRY_GRACE: Duration = Duration::from_millis(600);

/// The [color](Color) to use for live cells.
const LIVE_COLOR: Color = Color::BLACK;

/// The [color](Color) to use for dead cells.
const DEAD_COLOR: Color = Color::WHITE;

/// The [color](Color) of a depressed button.
const PRESSED_COLOR: Color = Color::YELLOW;

/// The [color](Color) of text labels.
const LABEL_COLOR: Color = Color::YELLOW;

/// The range of [key&#32;codes](KeyCode) that correspond to the number row.
const NUMBER_ROW_RANGE: RangeInclusive<u32> =
	KeyCode::Key1 as u32 ..= KeyCode::Key0 as u32;

/// The range of [key&#32;codes](KeyCode) that correspond to the numpad digits.
const NUMPAD_RANGE: RangeInclusive<u32> =
	KeyCode::Numpad0 as u32 ..= KeyCode::Numpad9 as u32;