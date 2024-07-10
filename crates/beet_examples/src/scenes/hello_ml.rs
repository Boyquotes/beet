use crate::beet::prelude::*;
use crate::prelude::*;
use bevy::prelude::*;

pub fn hello_ml(mut commands: Commands) {
	let bert_handle = AssetPlaceholder::<Bert>::new("default-bert.ron");

	commands
		.spawn((
			Name::new("Sentence Selector"),
			Sentence::new("please kill the baddies"),
			AssetLoadBlockAppReady,
			RunOnSentenceChange::default(),
			RunOnAppReady::default(),
			bert_handle,
			SetSentenceOnUserInput::default(),
			SentenceFlow::default(),
		))
		.with_children(|parent| {
			parent.spawn((Name::new("Heal Behavior"), Sentence::new("heal")));
			parent
				.spawn((Name::new("Attack Behavior"), Sentence::new("attack")));
		});
}
