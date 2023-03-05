pub mod light_ball_tracker;
mod light_ball_processing;
use bevy_ecs::prelude::{
	Schedule, World
};


const SETUP_LIST:  &'static [&'static dyn  crate::entity_builder::EntityBuilder] = &[
	&light_ball_tracker::LightBallTrackerBuilder::default(),
	&light_ball_processing::LightBallTrackerProcessingBuilder,
];

pub fn setup_entities(schedule: &mut Schedule, world: &mut World){
		
	//world.init_resource()
	for entity_builder in SETUP_LIST{
		entity_builder.setup(schedule, world);
	}
}