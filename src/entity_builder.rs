use bevy_ecs::prelude::*;
use bevy_ecs::system::CommandQueue;
pub trait EntityBuilder{
	/// Spawn entity defined by EntityBuilder implementation with all it's neceserry components and resources.
    ///
    /// # Arguments
    ///
    /// * `commands` - commands used for spawning entity, initializing components and resources
    ///
    /// # Examples
    ///
    /// ```
    /// // You can have rust code between fences inside the comments
    /// // If you pass --test to `rustdoc`, it will even test it for you!
    ///fn_some_system(commands: Commands){
    ///	let entity = EntityBuilderImplementation{}.spawn(commands);
	///    // .. use entity for other custom steps
	///}
    /// ```
	fn spawn(&self, commands: &mut Commands) -> Entity;

	/// Setups systems and other prerequisites used for components and resources of entity.
    ///
    /// # Arguments
    ///
    /// * `schedule` - used for attaching system functions for specific components and resource for this implementation
    /// * `world` - used for immidiate startup entity spawning
	/// 
    /// # Examples
    ///
    /// ```
    ///pub fn setup_entities(schedule: &mut Schedule, world: &mut World){
	///    EntityBuilderImplementation{}.setup(schedule, world);
	///    // ... setup other enities
	///}
    /// ```
	fn setup(&self, schedule: &mut Schedule, world: &mut World);

}

/// Helper function to be able to spawn entity from world without needing sperete implementation for it.
/// This is done by creating temporary commands and commands queue and running spawn function implementation with commands
///  and then immidiatly applying it to world.
/// 
/// 
/// # Arguments
///
/// * `world` - used for immidiate startup entity spawning
/// * `entity_builder` - (calls spawn(commands)) used for attaching system functions for specific components and resource for this implementation
pub fn spawn_from_world(world: &mut World, entity_builder: &dyn EntityBuilder ) -> Entity{
	let mut queue = CommandQueue::default();
	let mut commands = Commands::new(&mut queue, world);
	let entity = entity_builder.spawn(&mut commands);
	queue.apply(world);
	entity
}

// usful function

