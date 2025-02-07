use std::borrow::Cow;
use super::*;
use crate::prelude::*;
use beet_ecs::prelude::*;
use bevy::prelude::*;

// action_list!(CoreNode, [
// ]);

#[derive(Debug,Default, Clone, BeetModule)]
#[modules(EcsModule)]
#[actions(
		//movement
		Hover,
		Translate,
		SetAgentOnRun::<Velocity>,
		//steering
		Seek,
		Wander,
		SucceedOnArrive,
		FindSteerTarget,
		ScoreSteerTarget,
		DespawnSteerTarget,
		//robotics
		SetAgentOnRun::<DualMotorValue>,
		DepthSensorScorer,
	)]
#[components(
	AutoSpawn,
	RandomizePosition,
	//render
	RenderText,
	// steering
	SteerTarget,
	//force bundle
	Mass, 
	Velocity, 
	Impulse, 
	Force,
	//steer bundle
	MaxForce,
	MaxSpeed,
	ArriveRadius,
	WanderParams,
)]
#[bundles(
	ForceBundle,
	SteerBundle,
)]
pub struct CoreModule;


#[derive(Component, Deref, DerefMut, Reflect)]
#[reflect(Component,Default)]
pub struct RenderText(pub Cow<'static, str>);

impl RenderText{
	pub fn new(text: impl Into<Cow<'static, str>>) -> Self {
		Self(text.into())
	}
}

impl Default for RenderText {
	fn default() -> Self {
		Self::new("🥕")
	}
}

#[derive(Default)]
pub struct CorePlugin;

impl Plugin for CorePlugin {
	fn build(&self, app: &mut App) {
		app /*-*/
			.add_systems(PreUpdate, auto_spawn.before(PreTickSet))
			.add_systems(Update, randomize_position.in_set(PreTickSet))
		/*-*/;
	}
}
