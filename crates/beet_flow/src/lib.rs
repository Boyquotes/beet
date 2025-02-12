#![cfg_attr(test, feature(test, custom_test_frameworks))]
#![cfg_attr(test, test_runner(sweet::test_runner))]
#![allow(deprecated)] // TODO remove deprecated
#![cfg_attr(feature = "reflect", feature(trait_upcasting))]
/// # BeetFlow
///
use bevy::app::PluginGroup;
use bevy::app::PluginGroupBuilder;
pub mod continue_run;
pub mod control_flow;
pub mod control_flow_actions;
pub mod tree;

pub mod prelude {
	// required for macros to work internally
	pub use super::BeetFlowPlugin;
	pub use crate as beet_flow;
	pub use crate::continue_run::*;
	pub use crate::control_flow::*;
	pub use crate::control_flow_actions::*;
	pub use crate::tree::*;
	pub use beet_flow_macros::*;
	// allow flush_trigger in examples
	// #[cfg(feature = "sweet")]
	// pub use sweet::prelude::CoreWorldExtSweet;
	// // allow flush_trigger in examples
	// #[cfg(feature = "sweet")]
	// pub use sweet::prelude::EntityWorldMutwExt;
}


#[derive(Default)]
pub struct BeetFlowPlugin {
	// lifecycle_plugin: lifecycle::LifecyclePlugin,
}

impl BeetFlowPlugin {
	pub fn new() -> Self { Self::default() }
}


impl PluginGroup for BeetFlowPlugin {
	fn build(self) -> PluginGroupBuilder {
		PluginGroupBuilder::start::<Self>()
			.add(control_flow::observer_plugin)
			.add(continue_run::continue_run_plugin)
			.build()
	}
}
