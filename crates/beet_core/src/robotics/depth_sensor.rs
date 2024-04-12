use beet_ecs::prelude::*;
use bevy::prelude::*;

pub const DEFAULT_ULTRASOUND_MAX_DEPTH: f32 = 2.0;

#[derive(Debug, Clone, Component, PartialEq, Reflect)]
pub struct DepthSensor {
	// TODO Option<f32>
	pub value: f32,     // distance in meters
	pub max_depth: f32, // max distance in meters
	pub size: Vec3,
	pub pos: Vec3,
}

impl DepthSensor {
	pub fn new(offset: Vec3) -> Self {
		let size = Vec3::new(0.02, 0.02, 0.04);

		let pos = offset + Vec3::new(0., size.y * 0.5, 0.);

		let max_depth = DEFAULT_ULTRASOUND_MAX_DEPTH;
		let this = Self {
			max_depth,
			value: max_depth,
			size,
			pos,
		};

		this
	}
}

#[derive_action]
pub struct DepthSensorScorer {
	#[shared]
	pub score: Score,
	#[number(step = 0.1)]
	pub threshold_dist: f32,
	pub low_weight: u8,
	pub high_weight: u8,
}

impl Default for DepthSensorScorer {
	fn default() -> Self {
		Self {
			score: Score::Fail,
			threshold_dist: 0.5,
			low_weight: 40,
			high_weight: 60,
		}
	}
}

impl DepthSensorScorer {
	pub fn new(threshold_dist: f32, low_weight: u8, high_weight: u8) -> Self {
		Self {
			score: Score::Fail,
			threshold_dist,
			low_weight,
			high_weight,
		}
	}
}

pub fn depth_sensor_scorer(
	sensors: Query<&DepthSensor, Changed<DepthSensor>>,
	mut scorers: Query<(&mut DepthSensorScorer, &TargetEntity)>,
) {
	for (mut scorer, target) in scorers.iter_mut() {
		if let Ok(sensor) = sensors.get(**target) {
			let next_score = if sensor.value > scorer.threshold_dist {
				Score::Weight(scorer.low_weight)
			} else {
				Score::Weight(scorer.high_weight)
			};
			if next_score != scorer.score {
				println!("depth score updated: {:?}", next_score);
				scorer.score = next_score;
			}
		}
	}
}
