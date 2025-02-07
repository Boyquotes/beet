use bevy::prelude::*;

pub trait IntoWorld {
	fn into_world(self) -> World;
	fn into_world_ref(&self) -> &World;
	fn into_world_mut(&mut self) -> &mut World;
}

impl IntoWorld for World {
	fn into_world(self) -> World { self }
	fn into_world_ref(&self) -> &World { self }
	fn into_world_mut(&mut self) -> &mut World { self }
}
impl IntoWorld for App {
	fn into_world(mut self) -> World { std::mem::take(self.world_mut()) }
	fn into_world_ref(&self) -> &World { self.world() }
	fn into_world_mut(&mut self) -> &mut World { self.world_mut() }
}
