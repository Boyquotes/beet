use anyhow::Result;
use beet::prelude::*;
use bevy::prelude::*;
use flume::Receiver;
use flume::Sender;
use serde::Deserialize;
use serde::Serialize;
use std::marker::PhantomData;


#[derive(Default)]
pub struct BeetMessagePlugin<T: BeetModule>(pub PhantomData<T>);

impl<T: BeetModule> Plugin for BeetMessagePlugin<T> {
	fn build(&self, app: &mut App) {
		let (send, recv) = flume::unbounded();
		app /*-*/
    .insert_resource(BeetMessageRecv(recv))
		.insert_resource(BeetMessageSend(send))
		.add_systems(Update,
			message_handler::<T>
				.pipe(log_error)
				.before(PreTickSet)
		)
		/*-*/;
	}
}

#[derive(Serialize, Deserialize)]
pub enum BeetMessage {
	Spawn { bincode: Vec<u8> },
}
#[derive(Clone,Resource, Deref, DerefMut)]
pub struct BeetMessageRecv(pub Receiver<BeetMessage>);
#[derive(Clone,Resource, Deref, DerefMut)]
pub struct BeetMessageSend(pub Sender<BeetMessage>);


impl BeetMessage {
	pub fn spawn_bundle<T:BeetModule>(bundle: impl Bundle)->Result<Self>{
		let serde = BeetSceneSerde::<T>::new_with_bundle(bundle);
		let bincode = bincode::serialize(&serde)?;
		Ok(BeetMessage::Spawn { bincode })
	}
}

fn message_handler<T: BeetModule>(world: &mut World) -> Result<()> {
	let Ok(messages) = world.resource_mut::<BeetMessageRecv>().try_recv_all()
	else {
		return Ok(()); // disconnected
	};

	for message in messages {
		match message {
			BeetMessage::Spawn { bincode } => {
				let serde: BeetSceneSerde<T> = bincode::deserialize(&bincode)?;
				serde.write(world)?;
			}
		}
	}
	Ok(())
}
fn log_error(val: In<Result<()>>) {
	if let Err(e) = val.0 {
		log::error!("{e}");
	}
}



#[cfg(test)]
mod test {
	use crate::prelude::*;
	use anyhow::Result;
	use beet_ecs::prelude::*;
	use bevy::prelude::*;
	use sweet::*;

	#[test]
	fn works() -> Result<()> {
		pretty_env_logger::try_init().ok();
		let mut app = App::new();
		app /*-*/		
			.add_plugins(BeetMessagePlugin::<CoreModule>(default()))
			.add_plugins(BeetModulePlugin::<CoreModule>(default()))
		/*-*/;

		let send = app.world_mut().resource::<BeetMessageSend>().clone();

		let scene = BeetBuilder::new(Score::Weight(0.1))
			.into_scene::<CoreModule>();
		let bincode = bincode::serialize(&scene)?;
		// log::info!("{:?}", bincode);
		send.send(BeetMessage::Spawn{bincode})?;

		expect(app.world().entities().len()).to_be(0)?;
		app.update();
		expect(app.world().entities().len()).to_be(1)?;

		let first = app.world().iter_entities().next().unwrap().id();

		expect(&app).component(first)?.to_be(&Score::Weight(0.1))?;

		Ok(())
	}
}
