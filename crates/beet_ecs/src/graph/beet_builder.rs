use crate::prelude::*;
use bevy::prelude::*;
use bevy::reflect::GetTypeRegistration;
use bevy::utils::HashSet;

/// Couldn't resist.. Marker to identify the root of a behavior graph
#[derive(Debug, Default, Component, Reflect)]
#[reflect(Component, Default)]
pub struct BeetRoot;

pub type SpawnFunc = Box<dyn FnOnce(&mut World) -> Entity>;

pub trait BeetBundle: Bundle + Reflect + GetTypeRegistration {}
impl<T: Bundle + Reflect + GetTypeRegistration> BeetBundle for T {}

/// An opaque intermediary structure between a [`Bundle`] graph and a spawned [`Entity`]
/// This does the following when build
/// - Registers the bundle types
/// - Spawns the entities and forms parent-child relationships
pub struct BeetBuilder {
	pub children: Vec<BeetBuilder>,
	/// Inserts [`(Running, BeetRoot)`] components to this node if its the root
	pub insert_root_defaults: bool,
	/// Inserts [`(Name, RunTimer)`] components to this node
	pub insert_defaults: bool,
	pub spawn_func: SpawnFunc,
	// great name buddy
	pub misc_funcs: Vec<Box<dyn FnOnce(&mut World)>>,
	// pub world: Arc<RwLock<World>>,
}

impl BeetBuilder {
	pub fn new<T: BeetBundle>(bundle: T) -> Self {
		Self {
			children: Vec::new(),
			spawn_func: Box::new(move |world: &mut World| {
				Self::register_type::<T>(world);
				world.spawn(bundle).id()
			}),
			misc_funcs: Vec::new(),
			insert_root_defaults: true,
			insert_defaults: true,
		}
	}
	pub fn with_type<T: GetTypeRegistration>(mut self) -> Self {
		self.misc_funcs.push(Box::new(|world: &mut World| {
			Self::register_type::<T>(world);
		}));
		self
	}

	fn register_type<T: GetTypeRegistration>(world: &mut World) {
		world.init_resource::<AppTypeRegistry>();
		world
			.resource_mut::<AppTypeRegistry>()
			.write()
			.register::<T>();
	}

	pub fn build(self, world: &mut World) -> EntityTree {
		let insert_root_defaults = self.insert_root_defaults;
		let tree = self.build_recursive(world, &mut HashSet::default());
		if insert_root_defaults {
			world.entity_mut(tree.value).insert((BeetRoot, Running));
		}
		EntityTree(tree)
	}

	pub fn into_node(self, world: &mut World) -> EntityIdent {
		let root = self.build(world).value;
		EntityIdent::new(root)
	}

	pub fn into_scene<T: BeetModule>(self) -> BeetSceneSerde<T> {
		let mut world = World::new();
		world.insert_resource(T::type_registry());
		self.into_node(&mut world);
		BeetSceneSerde::<T>::new(&world)
	}


	// TODO deprecate this in favor of an optional bundle
	pub fn insert_default_components(
		entity: &mut EntityWorldMut,
		default_name: String,
	) {
		entity.insert((RunTimer::default(), RootIsTargetAgent));
		if entity.contains::<Name>() == false {
			entity.insert(Name::new(default_name));
		}
	}

	fn build_recursive(
		self,
		world: &mut World,
		visited: &mut HashSet<Entity>,
	) -> Tree<Entity> {
		for func in self.misc_funcs {
			func(world);
		}
		let entity = (self.spawn_func)(world);
		visited.insert(entity);

		let children = self
			.children
			.into_iter()
			.map(|child| child.build_recursive(world, visited))
			.collect::<Vec<_>>();

		let mut entity = world.entity_mut(entity);

		for child in children.iter() {
			entity.add_child(child.value);
		}

		if self.insert_defaults {
			let default_name = format!("Node {}", visited.len());
			Self::insert_default_components(&mut entity, default_name);
		}
		Tree {
			value: entity.id(),
			children,
		}
	}

	pub fn child<M>(mut self, child: impl IntoBeetBuilder<M>) -> Self {
		self.children.push(child.into_beet_builder());
		self
	}
}



pub struct IntoIntoBeetBuilder;
pub struct ItemIntoBeetBuilder;
pub struct TupleIntoBeetBuilder;

pub trait IntoBeetBuilder<M>: Sized {
	fn into_beet_builder(self) -> BeetBuilder;
	fn child<M2>(self, child: impl IntoBeetBuilder<M2>) -> BeetBuilder {
		self.into_beet_builder().child(child)
	}
}

impl<T> IntoBeetBuilder<IntoIntoBeetBuilder> for T
where
	T: Into<BeetBuilder>,
{
	fn into_beet_builder(self) -> BeetBuilder { self.into() }
}

impl<T: BeetBundle> IntoBeetBuilder<ItemIntoBeetBuilder> for T {
	fn into_beet_builder(self) -> BeetBuilder { BeetBuilder::new(self) }
}


#[cfg(test)]
mod test {
	use crate::prelude::*;
	use anyhow::Result;
	use bevy::prelude::*;
	use sweet::*;

	#[derive(Component, Reflect)]
	pub struct Foobar;

	#[test]
	fn works() -> Result<()> {
		let _node = BeetBuilder::new(EmptyAction);
		let _node2 = BeetBuilder::new((
			EmptyAction,
			Foobar,
			SetOnSpawn::<Score>::default(),
		));
		let _node = EmptyAction.child(
			(EmptyAction, SetOnSpawn::<Score>::default()).child(EmptyAction),
		);

		Ok(())
	}

	#[test]
	fn into() -> Result<()> {
		fn foo<M>(_val: impl IntoBeetBuilder<M>) {}

		let _ = foo(EmptyAction.child(EmptyAction));
		let _ = foo(EmptyAction
			.child((EmptyAction, EmptyAction))
			.child(EmptyAction)
			.child(
				(EmptyAction, EmptyAction)
					.child(EmptyAction)
					.child(EmptyAction),
			));


		Ok(())
	}

	#[test]
	fn spawns() -> Result<()> {
		let mut world = World::new();

		let root = (Score::default(), SetOnSpawn(Score::Weight(0.5)))
			.into_beet_builder()
			.with_type::<Score>() // not needed by happenstance but usually required
			.build(&mut world)
			.value;

		expect(&world).to_have_entity(root)?;
		expect(&world)
			.component(root)?
			.to_be(&SetOnSpawn(Score::Weight(0.5)))?;

		// test shared component
		expect(&world).component(root)?.to_be(&Score::default())?;

		Ok(())
	}

	#[test]
	fn default_components() -> Result<()> {
		let mut app = App::new();
		let actions = test_constant_behavior_tree();
		let root = actions.build(app.world_mut()).value;

		expect(&app).to_have_component::<SetOnSpawn<Score>>(root)?;
		expect(&app).to_have_component::<RootIsTargetAgent>(root)?;
		expect(&app).to_have_component::<RunTimer>(root)?;
		expect(&app).to_have_component::<Score>(root)?;

		Ok(())
	}
}
