#[allow(unused)]
use crate::prelude::*;
use bevy::ecs::entity::MapEntities;
use bevy::ecs::reflect::ReflectMapEntities;
use bevy::prelude::*;

/// Used by actions to specify some target, ie seek.
#[derive(Debug, PartialEq, Deref, DerefMut, Component, Reflect)]
#[reflect(Component, MapEntities)]
pub struct ActionTarget(pub Entity);

impl MapEntities for ActionTarget {
	fn map_entities<M: EntityMapper>(&mut self, entity_mapper: &mut M) {
		**self = entity_mapper.map_entity(**self);
	}
}


#[cfg(test)]
mod test {
	use crate::prelude::*;
	use anyhow::Result;
	use bevy::prelude::*;
	use sweet::*;

	#[test]
	fn despawn() -> Result<()> {
		let mut app = App::new();
		// app.add_systems(PostUpdate, despawn_graph_on_agent_removed);
		app.add_plugins(BeetSystemsPlugin::<EcsModule, _>::default());

		let behavior = InsertOnRun(RunResult::Success)
			.into_beet_builder()
			.build(app.world_mut())
			.value;
		let target = app.world_mut().spawn_empty().add_child(behavior).id();

		expect(app.world().entities().len()).to_be(2)?;
		app.update();
		despawn_with_children_recursive(app.world_mut(), target);

		expect(app.world().entities().len()).to_be(0)?;

		Ok(())
	}
}
