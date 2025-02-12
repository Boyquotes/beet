use super::PostTickSet;
use crate::prelude::*;
use bevy::prelude::*;
use std::borrow::Cow;

/// A plugin that logs lifecycle events for action entities.
/// If they have a [`Name`] that will be used instead of the entity id.
/// It triggers [OnLogMessage] events, and also adds a listener that
/// will print to stdout if [`BeetDebugConfig::log_to_stdout`] is true.
///
/// If a [`BeetDebugConfig`] is not present, it will use the default.
#[derive(Debug, Clone)]
pub struct BeetDebugPlugin {
	pub log_on_run: bool,
	pub log_running: bool,
	pub log_on_run_result: bool,
	/// disable logging to stdout, useful if instead using rendered terminal,
	/// networking etc.
	pub no_stdout: bool,
}
impl Default for BeetDebugPlugin {
	fn default() -> Self {
		Self {
			log_on_run: true,
			log_running: false,
			log_on_run_result: false,
			no_stdout: true,
		}
	}
}

impl BeetDebugPlugin {
	pub fn with_result() -> Self {
		Self {
			log_on_run: true,
			log_on_run_result: true,
			log_running: false,
			no_stdout: true,
		}
	}
	pub fn with_all() -> Self {
		Self {
			log_on_run: true,
			log_running: true,
			log_on_run_result: true,
			no_stdout: true,
		}
	}
}

impl Plugin for BeetDebugPlugin {
	fn build(&self, app: &mut App) {
		// TODO when resolved: [Observers::run_if](https://github.com/bevyengine/bevy/issues/14195)
		app.add_plugins(bevy::log::LogPlugin::default())
			.add_observer(log_on_run.never_param_warn())
			.add_observer(log_on_run_result.never_param_warn())
			.add_observer(log_to_stdout.never_param_warn())
			.add_systems(
				Update,
				log_running
					.never_param_warn()
					.run_if(resource_exists::<LogRunningMarker>)
					.in_set(PostTickSet),
			);

		if self.log_on_run {
			app.init_resource::<LogOnRunMarker>();
		}

		if self.log_on_run_result {
			app.init_resource::<LogOnRunResultMarker>();
		}

		if self.log_running {
			app.init_resource::<LogRunningMarker>();
		}

		if !self.no_stdout {
			app.init_resource::<LogToStdoutMarker>();
		}
	}
}


/// An 'stdout observer', triggering this will log to the ui terminal.
#[derive(Debug, Event, Deref)]
pub struct OnLogMessage(pub Cow<'static, str>);

impl OnLogMessage {
	pub fn new(msg: impl Into<Cow<'static, str>>) -> Self { Self(msg.into()) }
	pub fn new_with_query(
		entity: Entity,
		query: &Query<&Name>,
		prefix: &str,
	) -> Self {
		Self::new_with_optional(entity, query.get(entity).ok(), prefix)
	}
	pub fn new_with_optional(
		entity: Entity,
		name: Option<&Name>,
		prefix: &str,
	) -> Self {
		let msg = name
			.map(|n| format!("{prefix}: {n}"))
			.unwrap_or_else(|| format!("{prefix}: {entity}"));
		Self(msg.into())
	}
	pub fn log(&self) {
		log::info!("{}", self.0);
	}
}

/// marker resource for [log_on_run]
#[derive(Debug, Default, Clone, Resource, Reflect)]
#[reflect(Resource)]
pub struct LogOnRunMarker;
/// marker resource for [log_on_run_result]
#[derive(Debug, Default, Clone, Resource, Reflect)]
#[reflect(Resource)]
pub struct LogOnRunResultMarker;
/// marker resource for [log_running]
#[derive(Debug, Default, Clone, Resource, Reflect)]
#[reflect(Resource)]
pub struct LogRunningMarker;
/// marker resource for [log_to_stdout]
#[derive(Debug, Default, Clone, Resource, Reflect)]
#[reflect(Resource)]
pub struct LogToStdoutMarker;


fn log_to_stdout(trigger: Trigger<OnLogMessage>, _m: Res<LogToStdoutMarker>) {
	trigger.log();
}

fn log_on_run(
	ev: Trigger<OnRunAction>,
	mut commands: Commands,
	query: Query<&Name>,
	_m: Res<LogOnRunMarker>,
) {
	let msg =
		OnLogMessage::new_with_query(ev.resolve_action(), &query, "OnRun");
	msg.log();
	commands.trigger(msg);
}


fn log_on_run_result(
	ev: Trigger<OnResultAction>,
	mut commands: Commands,
	query: Query<&Name>,
	_m: Res<LogOnRunResultMarker>,
) {
	let msg = OnLogMessage::new_with_query(
		ev.resolve_action(),
		&query,
		&format!("{:?}", &ev.payload),
	);
	msg.log();
	commands.trigger(msg);
}

fn log_running(
	mut commands: Commands,
	query: Populated<(Entity, Option<&Name>), With<Running>>,
) {
	for (entity, name) in query.iter() {
		let name = name
			.map(|n| n.to_string())
			.unwrap_or_else(|| entity.to_string());
		let msg = OnLogMessage::new(format!("Running: {}", name));
		msg.log();
		commands.trigger(msg);
	}
}
