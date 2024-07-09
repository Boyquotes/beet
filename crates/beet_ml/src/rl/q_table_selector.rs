use crate::prelude::*;
use beet_ecs::prelude::*;
use bevy::prelude::*;

/// An action used for training, evaluating and running QTable agents.
/// - If any child is running do nothing
/// - If a child fails, also fail.
/// - If a child succeeds, evaluate reward and select next action.
#[derive(Debug, Clone, PartialEq, Action, Reflect)]
#[reflect(Component, ActionMeta)]
#[category(ActionCategory::Agent)]
#[systems(q_table_selector::<L>.in_set(TickSet))]
pub struct QTableSelector<L: QPolicy> {
	pub evaluate: bool,
	pub learner: L,
	pub current_episode: usize,
	pub current_step: usize,
}

fn q_table_selector<L: QPolicy>(
	mut commands: Commands,
	mut agents: Query<(&L::State, &mut L::Action, &Reward)>,
	mut query: Query<
		(Entity, &TargetAgent, &mut QTableSelector<L>, &Children),
		With<Running>,
	>,
	children_running: Query<(), With<Running>>,
	children_results: Query<&RunResult>,
) {
	for (action_entity, agent, mut selector, children) in query.iter_mut() {
		if any_child_running(children, &children_running) {
			// 1. wait for child to finish
			continue;
		}
		#[allow(unused_variables)]
		let Ok((state, action, reward)) = agents.get_mut(**agent) else {
			continue;
		};

		selector.current_step += 1;

		match first_child_result(children, &children_results) {
			Some((_index, result)) => match result {
				&RunResult::Failure => {
					// end episode
					commands.entity(action_entity).insert(RunResult::Failure);
					continue;
				}
				&RunResult::Success => {
					// evaluate reward
					// selector.learner.set_discounted_reward(params, action, reward, prev_state, next_state)

					// *action = selector.next_action(state, reward);
					// true
					// if index == children.len() - 1 {
					// 	// finish
					// 	commands.entity(parent).insert(RunResult::Success);
					// } else {
					// 	// next
					// 	commands.entity(children[index + 1]).insert(Running);
					// }
				}
			},
			None => {
				// start
				// commands.entity(children[0]).insert(Running);
			}
		}
	}
}