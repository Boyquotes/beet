use beet::prelude::*;
use bevy::prelude::*;
use std::f32::consts::TAU;


#[derive_action]
#[action(graph_role=GraphRole::Agent)]
/// Translate the agent up and down in a sine wave
pub struct Hover {
	/// Measured in Hz
	#[inspector(min = 0.1, max = 3., step = 0.1)]
	pub speed: f32,
	/// Measured in meters
	#[inspector(min = 0.1, max = 3., step = 0.1)]
	pub height: f32,
}

impl Hover {
	pub fn new(speed: f32, height: f32) -> Self { Self { speed, height } }
}

fn hover(
	mut _commands: Commands,
	time: Res<Time>,
	mut transforms: Query<&mut Transform>,
	query: Query<(&TargetAgent, &Hover), With<Running>>,
) {
	for (target, hover) in query.iter() {
		let elapsed = time.elapsed_seconds();
		let y = f32::sin(TAU * elapsed * hover.speed) * hover.height;
		transforms.get_mut(**target).unwrap().translation.y = y;
	}
}
