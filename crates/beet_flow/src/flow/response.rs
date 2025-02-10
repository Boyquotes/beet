use crate::prelude::*;
use bevy::prelude::*;
use std::marker::PhantomData;


pub trait Response: ActionPayload {
	type Req: Request<Res = Self>;
}

/// Add this to an entity to prevent the run result from bubbling up.
///
/// Any action that requires this needs to manually call OnChildResult
/// on the parent entity. For an example, see [`RepeatFlow`].
#[derive(Default, Component, Reflect)]
pub struct NoBubble;

impl<T: Response> On<T> {
	pub fn trigger_bubble(&self, mut commands: Commands) {
		commands.entity(self.action).trigger(self.clone());
	}
}



/// Global observer to pass an action up to all *parent observers*,
/// so they may handle the response.
///
/// Unlike [propagate_request_to_observers], this is called on parent
/// observers.
pub fn propagate_response_to_parent_observers<R: Response>(
	res: Trigger<On<R>>,
	mut commands: Commands,
	action_observers: Query<&ActionObservers>,
	action_observer_markers: Query<(), With<ActionObserverMarker>>,
	no_bubble: Query<(), With<NoBubble>>,
	parents: Query<&Parent>,
) {
	if action_observer_markers.contains(res.entity())
		|| no_bubble.contains(res.action)
	{
		return;
	}

	if let Ok(parent) = parents.get(res.action) {
		let parent = parent.get();
		if let Ok(action_observers) = action_observers.get(parent) {
			let mut res = (*res).clone();
			res.prev_action = res.action;
			res.action = parent;
			commands.trigger_targets(res, (*action_observers).clone());
		}
	}
}

#[action(bubble_result::<R>)]
#[derive(Debug, Component, Default, Clone, Copy, PartialEq, Reflect)]
pub struct BubbleUpFlow<R: Response>(PhantomData<R>);


/// An action is usually triggered
fn bubble_result<R: Response>(trig: Trigger<On<R>>, commands: Commands) {
	trig.trigger_bubble(commands);
}

#[cfg(test)]
mod test {
	use crate::prelude::*;
	use bevy::prelude::*;
	use sweet::prelude::*;

	#[test]
	fn bubbles_up() {
		let mut app = App::new();
		app.add_plugins(BeetFlowPlugin::default());
		let world = app.world_mut();
		let counter = observe_triggers::<On<RunResult>>(world);
		let mut child = Entity::PLACEHOLDER;
		let mut grandchild = Entity::PLACEHOLDER;

		let parent = world
			.spawn(BubbleUpFlow::<RunResult>::default())
			.with_children(|parent| {
				child = parent
					.spawn(BubbleUpFlow::<RunResult>::default())
					.with_children(|parent| {
						grandchild =
							parent.spawn(RespondWith(RunResult::Success)).id();
					})
					.id();
			})
			.id();
		world.entity_mut(grandchild).flush_trigger(Run.trigger());

		expect(&counter).to_have_been_called_times(5);
		expect(&counter).to_have_returned_nth_with(0, &On {
			payload: RunResult::Success,
			origin: grandchild,
			action: grandchild,
			prev_action: Entity::PLACEHOLDER,
		});
		expect(&counter).to_have_returned_nth_with(1, &On {
			payload: RunResult::Success,
			origin: grandchild,
			action: child,
			prev_action: grandchild,
		});
		expect(&counter).to_have_returned_nth_with(3, &On {
			payload: RunResult::Success,
			origin: grandchild,
			action: parent,
			prev_action: child,
		});
	}
}
