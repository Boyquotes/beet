use beet_ecs::prelude::*;
use bevy::prelude::*;
use forky_core::ResultTEExt;

#[derive_action]
#[action(graph_role=GraphRole::Agent)]
/// Applies constant translation, multiplied by [`Time::delta_seconds`]
pub struct Translate {
	/// Translation to apply, in meters per second
	#[inspector(min=-2., max=2., step=0.1)]
	pub translation: Vec3,
}

impl Translate {
	pub fn new(translation: Vec3) -> Self { Self { translation } }
}

fn translate(
	mut _commands: Commands,
	time: Res<Time>,
	mut transforms: Query<&mut Transform>,
	query: Query<(&TargetAgent, &Translate), With<Running>>,
) {
	for (target, translate) in query.iter() {
		if let Some(mut transform) =
			transforms.get_mut(**target).ok_or(|e| log::warn!("{e}"))
		{
			transform.translation +=
				translate.translation * time.delta_seconds();
		}
	}
}
