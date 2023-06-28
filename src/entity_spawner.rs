use bevy::ecs::prelude::*;
use bevy::ecs::system::CommandQueue;
pub trait EntitySpawner{
	/// Spawn entity defined by EntitySpawner implementation with all it's necessary components and resources.
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


}

/// Helper function to be able to spawn entity from world without needing separate implementation for it.
/// This is done by creating temporary commands and commands queue and running spawn function implementation with commands
///  and then immediately applying it to world.
/// 
/// 
/// # Arguments
///
/// * `world` - used for immidiate startup entity spawning
/// * `entity_builder` - (calls spawn(commands)) used for attaching system functions for specific components and resource for this implementation
pub fn spawn_from_world(world: &mut World, entity_builder: &dyn EntitySpawner ) -> Entity{
	let mut queue = CommandQueue::default();
	let mut commands = Commands::new(&mut queue, world);
	let entity = entity_builder.spawn(&mut commands);
	queue.apply(world);
	entity
}

// useful function

