pub mod light_ball_tracker;
mod light_ball_processing;
use bevy::app::Plugin;

const SETUP_LIST:  &'static [&'static dyn  Plugin] = &[
	&light_ball_tracker::LightBallTrackerBuilder::default(),
	&light_ball_processing::LightBallTrackerProcessingBuilder,
];

pub fn setup_entities(app: &mut bevy::prelude::App){
		
	//world.init_resource()
	for entity_builder in SETUP_LIST{
		entity_builder.build(app);
	}
}