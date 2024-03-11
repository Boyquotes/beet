use crate::prelude::*;
use bevy_app::App;
use bevy_ecs::prelude::*;

pub trait WorldOrCommands {
	fn spawn(&mut self, bundle: impl Bundle) -> Entity;
	fn insert(&mut self, entity: Entity, bundle: impl Bundle);
	fn insert_action(&mut self, entity: Entity, action: &dyn Action);
}

impl WorldOrCommands for World {
	fn spawn(&mut self, bundle: impl Bundle) -> Entity {
		self.spawn(bundle).id()
	}
	fn insert(&mut self, entity: Entity, bundle: impl Bundle) {
		self.entity_mut(entity).insert(bundle);
	}
	fn insert_action(&mut self, entity: Entity, action: &dyn Action) {
		action.insert_from_world(&mut self.entity_mut(entity));
	}
}
impl WorldOrCommands for App {
	fn spawn(&mut self, bundle: impl Bundle) -> Entity {
		self.world.spawn(bundle).id()
	}
	fn insert(&mut self, entity: Entity, bundle: impl Bundle) {
		self.world.entity_mut(entity).insert(bundle);
	}
	fn insert_action(&mut self, entity: Entity, action: &dyn Action) {
		action.insert_from_world(&mut self.world.entity_mut(entity));
	}
}
impl<'w, 's> WorldOrCommands for Commands<'w, 's> {
	fn spawn(&mut self, bundle: impl Bundle) -> Entity {
		self.spawn(bundle).id()
	}
	fn insert(&mut self, entity: Entity, bundle: impl Bundle) {
		self.entity(entity).insert(bundle);
	}
	fn insert_action(&mut self, entity: Entity, action: &dyn Action) {
		action.insert_from_commands(&mut self.entity(entity));
	}
}
