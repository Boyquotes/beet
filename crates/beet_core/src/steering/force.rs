use beet_ecs::exports::Reflect;
use bevy::prelude::*;


/// A vector measured in (m/s)
#[derive(
	Debug, Default, Clone, PartialEq, Deref, DerefMut, Component, Reflect,
)]
#[reflect(Component, Default)]
pub struct Velocity(pub Vec3);
/// A force that is cleared each frame.
#[derive(
	Debug, Default, Clone, PartialEq, Deref, DerefMut, Component, Reflect,
)]
#[reflect(Component, Default)]
pub struct Impulse(pub Vec3);
/// A constant force that is cleared each frame.
#[derive(
	Debug, Default, Clone, PartialEq, Deref, DerefMut, Component, Reflect,
)]
#[reflect(Component, Default)]
pub struct Force(pub Vec3);
/// A constant force that is cleared each frame.
#[derive(
	Debug, Copy, Clone, PartialEq, Deref, DerefMut, Component, Reflect,
)]
#[reflect(Component, Default)]
pub struct Mass(pub f32);

impl Default for Mass {
	fn default() -> Self { Self(1.0) }
}


#[derive(Default, Bundle)]
pub struct ForceBundle {
	pub mass: Mass,
	pub velocity: Velocity,
	pub impulse: Impulse,
	pub force: Force,
}



/// Implementation of position, velocity, force integration
/// as described by Daniel Shiffman
/// https://natureofcode.com/vectors/#acceleration
pub fn integrate_force(
	time: Res<Time>,
	mut query: Query<(
		&mut Transform,
		Option<&Mass>,
		&mut Velocity,
		Option<&Force>,
		Option<&mut Impulse>,
	)>,
) {
	for (mut transform, mass, mut velocity, force, mut impulse) in
		query.iter_mut()
	{
		let mut force = force.map(|f| **f).unwrap_or_default();
		let mass = mass.map(|m| **m).unwrap_or(1.0);
		if let Some(impulse) = impulse.as_mut() {
			force += ***impulse;
			***impulse = Vec3::ZERO;
		}
		let acceleration = force / mass;
		**velocity += acceleration;
		transform.translation += **velocity * time.delta_seconds();
	}
}


#[cfg(test)]
mod test {
	use crate::prelude::*;
	use anyhow::Result;
	use bevy::prelude::*;
	use sweet::*;

	#[test]
	pub fn works() -> Result<()> {
		let mut app = App::new();

		app.add_plugins(SteeringPlugin {
			wrap_around: WrapAround {
				half_extents: Vec3::new(100., 100., 100.),
			},
		});
		app.insert_time();

		let velocity_entity = app
			.world_mut()
			.spawn((TransformBundle::default(), ForceBundle {
				velocity: Velocity(Vec3::new(1., 0., 0.)),
				..default()
			}))
			.id();
		let force_entity = app
			.world_mut()
			.spawn((TransformBundle::default(), ForceBundle {
				force: Force(Vec3::new(1., 0., 0.)),
				..default()
			}))
			.id();
		let impulse_entity = app
			.world_mut()
			.spawn((TransformBundle::default(), ForceBundle {
				impulse: Impulse(Vec3::new(1., 0., 0.)),
				..default()
			}))
			.id();

		let mass_entity = app
			.world_mut()
			.spawn((TransformBundle::default(), ForceBundle {
				mass: Mass(2.),
				impulse: Impulse(Vec3::new(1., 0., 0.)),
				..default()
			}))
			.id();



		app.update_with_secs(1);

		expect(&app)
			.component::<Transform>(velocity_entity)?
			.map(|t| t.translation)
			.to_be(Vec3::new(1., 0., 0.))?;
		expect(&app)
			.component::<Transform>(force_entity)?
			.map(|t| t.translation)
			.to_be(Vec3::new(1., 0., 0.))?;
		expect(&app)
			.component::<Transform>(impulse_entity)?
			.map(|t| t.translation)
			.to_be(Vec3::new(1., 0., 0.))?;
		expect(&app) // impulses are cleared each frame
			.component(impulse_entity)?
			.to_be(&Impulse(Vec3::ZERO))?;
		expect(&app)
			.component::<Transform>(mass_entity)?
			.map(|t| t.translation)
			.to_be(Vec3::new(0.5, 0., 0.))?;

		app.update_with_secs(1);

		expect(&app)
			.component::<Transform>(velocity_entity)?
			.map(|t| t.translation)
			.to_be(Vec3::new(2., 0., 0.))?;
		expect(&app)
			.component::<Transform>(force_entity)?
			.map(|t| t.translation)
			.to_be(Vec3::new(3., 0., 0.))?;
		expect(&app)
			.component::<Transform>(impulse_entity)?
			.map(|t| t.translation)
			.to_be(Vec3::new(2., 0., 0.))?;
		expect(&app)
			.component::<Transform>(mass_entity)?
			.map(|t| t.translation)
			.to_be(Vec3::new(1., 0., 0.))?;


		Ok(())
	}
}
