//! A moderate example of beet demonstrating a simple enemy behavior
//! with a mix of utility ai and behavior tree paradigms.
//!
//! The enemy, Malenia, will try to heal herself if her health is below 50% and she has potions.
//! Otherwise she will attack the player.
//!
//! https://eldenring.wiki.fextralife.com/Malenia+Blade+of+Miquella
use beet::prelude::*;
use bevy::prelude::*;
use std::io::Write;
use std::io::{
	self,
};
use sweet::prelude::RandomSource;


fn await_key() {
	loop {
		print!("Press any key...");
		io::stdout().flush().unwrap();
		let mut input = String::new();
		io::stdin().read_line(&mut input).unwrap();
		println!("hello");
	}
}


fn main() {
	await_key();d
	println!("👩\tMalenia says: {INTRO}");

	let mut app = App::new();
	app.add_plugins((MinimalPlugins, BeetFlowPlugin::default()))
		.add_systems(Update, health_handler)
		.init_resource::<RandomSource>();

	// in this example the player is a pacifist that doesnt do anything
	app.world_mut()
		.spawn((Name::new("Elden Lord"), Health::default()));

	app.world_mut()
		// this is our agent, here the behavior (FallbackFlow) is attached directly
		// but it could be a child or completely seperate
		.spawn((
			Name::new("Malenia"),
			Health::default(),
			HealingPotions(2),
			FallbackFlow::default(),
			RepeatFlow::default(),
		))
		.with_children(|root| {
			// A common behavior tree pattern is to 'try' actions, ie try to heal self
			// if low health and has potions, else attack player
			root.spawn((Name::new("Try Heal Self"), TryHealSelf));

			// if TryHeal fails the tree will 'fallback' to the next action
			// lets use utility ai to determine which attack to use
			root.spawn((Name::new("Attack"), ScoreFlow::default()))
				.with_child((
					Name::new("Waterfoul Dance"),
					// swap this out for a more advanced score provider based on
					// player health, distance, etc
					RandomScoreProvider::default(),
					AttackPlayer {
						max_damage: 15.0,
						max_recoil: 30.0,
					},
					ReturnWith(RunResult::Success),
				))
				.with_child((
					// pretty much doomed if she decides to use this
					// so lets give it a low score. This is a 5% chance because the alternative
					// is a random value between 0 and 1
					Name::new("Scarlet Aeonia"),
					ReturnWith(ScoreValue(0.05)),
					AttackPlayer {
						max_damage: 10_000.0,
						max_recoil: 10.0,
					},
					ReturnWith(RunResult::Success),
				));
		})
		// actions can be declared inline if they have no parameters
		// .observe(|_: Trigger<OnRunAction>| {
		// 	println!("👩\tMalenia is thinking..");
		// })
		.trigger(OnRun::local());
	app.run();
	println!("done");
}


#[action(attack_player)]
#[derive(Component)]
struct AttackPlayer {
	max_damage: f32,
	max_recoil: f32,
}

fn attack_player(
	ev: Trigger<OnRun>,
	attacks: Query<(&AttackPlayer, &Name)>,
	mut query: Query<(&mut Health, &Name)>,
	mut random_source: ResMut<RandomSource>,
) {
	let (attack, attack_name) = attacks
		.get(ev.action)
		.expect(&expect_action::to_have_action(&ev));
	println!("🔪  \tMalenia attacks with {}", attack_name);

	for (mut health, name) in query.iter_mut() {
		if name.as_str() == "Malenia" {
			let damage: f32 =
				random_source.random_range(0.0..attack.max_recoil).round();
			health.0 -= damage;
			println!(
				"❗  \tMalenia takes {} recoil damage, current health: {}",
				damage, health.0
			);
		} else {
			let damage: f32 =
				random_source.random_range(0.0..attack.max_damage).round();
			health.0 -= damage;
			println!(
				"❗  \tPlayer takes {} damage, current health: {}",
				damage, health.0
			);
		}
	}
	println!();
}


fn health_handler(
	query: Populated<(&Health, &Name), Changed<Health>>,
	mut exit: EventWriter<AppExit>,
) {
	for (health, name) in query.iter() {
		if health.0 > 0. {
			continue;
		} else if name.as_str() == "Malenia" {
			println!("👩\tMalenia says: 'Your strength, extraordinary...'\n✅\tYou win!");
		} else {
			println!("👩\tMalenia says: 'I am Malenia. Blade of Miquella'\n❌\tYou lose");
		}
		exit.send(AppExit::Success);
		println!("here");
		// std::process::exit(0);
	}
}

#[derive(Component)]
struct Health(f32);
#[derive(Component)]
struct HealingPotions(usize);

impl Default for Health {
	fn default() -> Self { Self(100.0) }
}

#[action(provide_random_score)]
#[derive(Component, Reflect)]
struct RandomScoreProvider {
	pub scalar: f32,
	pub offset: f32,
}

impl Default for RandomScoreProvider {
	fn default() -> Self {
		Self {
			scalar: 1.0,
			offset: 0.0,
		}
	}
}


fn provide_random_score(
	ev: Trigger<OnRun<RequestScore>>,
	commands: Commands,
	mut random_source: ResMut<RandomSource>,
	query: Query<&RandomScoreProvider>,
) {
	let score_provider = query
		.get(ev.action)
		.expect(&expect_action::to_have_action(&ev));

	let rnd: f32 = random_source.random();
	ev.trigger_result(
		commands,
		ScoreValue(rnd * score_provider.scalar + score_provider.offset),
	);
}


#[action(try_heal_self)]
#[derive(Component, Reflect)]
struct TryHealSelf;

fn try_heal_self(
	ev: Trigger<OnRun>,
	commands: Commands,
	mut query: Query<(&mut Health, &mut HealingPotions)>,
) {
	let (mut health, mut potions) = query
		.get_mut(ev.origin)
		.expect(&expect_action::to_have_origin(&ev));

	if health.0 < 50.0 && potions.0 > 0 {
		health.0 += 30.;
		potions.0 -= 1;
		println!("💊\tMalenia heals herself, current health: {}\n", health.0);
		ev.trigger_result(commands, RunResult::Success);
	} else {
		// we didnt do anything so action was a failure
		ev.trigger_result(commands, RunResult::Failure);
	}
}

const INTRO: &str = r#"
			I dreamt for so long.
			My flesh was dull gold...and my blood, rotted.
			Corpse after corpse, left in my wake...
			As I awaited... his return.
			... Heed my words.
			I am Malenia. Blade of Miquella.
			And I have never known defeat.
"#;
