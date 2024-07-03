use beet::prelude::*;
use beet_examples::prelude::*;
use bevy::prelude::*;

pub fn hello_net(mut commands: Commands) {
	commands
		.spawn((SerializeMarker, SequenceSelector::default(), Running))
		.with_children(|parent| {
			parent.spawn((
				LogOnRun::new("Send: AppReady"),
				TriggerOnRun(AppReady),
			));
		});
	commands.spawn((
		SerializeMarker,
		InsertOnTrigger::<OnUserMessage, Running>::new(Running),
		LogOnRun::new("Recv: Player Message"),
	));
}
