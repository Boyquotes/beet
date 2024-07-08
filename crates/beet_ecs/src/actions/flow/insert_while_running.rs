use crate::prelude::*;
use bevy::prelude::*;

pub type LongRun = InsertWhileRunning<Running>;


/// 1. Adds the provided component when [`OnRun`] is called
/// 2. Removes the component when [`OnRunResult`] is called
#[derive(Default, Action, Reflect)]
#[reflect(Default, Component)]
#[generic_observers(on_start_running, on_stop_running)]
pub struct InsertWhileRunning<T: Default + GenericActionComponent>(pub T);

fn on_start_running<T: Default + GenericActionComponent>(
	trigger: Trigger<OnRun>,
	query: Query<&InsertWhileRunning<T>>,
	mut commands: Commands,
) {
	if let Ok(insert_while_running) = query.get(trigger.entity()) {
		commands
			.entity(trigger.entity())
			.insert(insert_while_running.0.clone());
	}
}
fn on_stop_running<T: Default + GenericActionComponent>(
	trigger: Trigger<OnRunResult>,
	mut commands: Commands,
) {
	commands.entity(trigger.entity()).remove::<T>();
}


#[cfg(test)]
mod test {
	use crate::prelude::*;
	use anyhow::Result;
	use bevy::prelude::*;
	use sweet::*;

	#[test]
	fn works() -> Result<()> {
		let mut world = World::new();

		let entity = world.spawn(LongRun::default()).trigger(OnRun).id();
		expect(world.entities().len()).to_be(3)?;
		expect(world.entity(entity).get::<Running>()).to_be_none()?;
		world.flush();
		expect(world.entity(entity).get::<Running>()).to_be_some()?;
		world.entity_mut(entity).trigger(OnRunResult::success());
		expect(world.entity(entity).get::<Running>()).to_be_some()?;
		world.flush();
		expect(world.entity(entity).get::<Running>()).to_be_none()?;
		Ok(())
	}
}
