pub mod opencv_trackers;
pub mod tracker;

use bevy_ecs::prelude::{
	Schedule, World
};

pub fn setup_entities(schedule: &mut Schedule, world: &mut World){
	opencv_trackers::OpencvTrackers::init_stage(schedule)
		.add_system(tracker::print_trackers_system);
	opencv_trackers::setup_entities(schedule, world);
}