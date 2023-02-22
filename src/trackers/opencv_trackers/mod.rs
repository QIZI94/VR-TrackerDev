pub mod camera;
pub mod camera_observer;
pub mod opencv_utilities;
pub mod light_ball_trackers;
use bevy_ecs::prelude::{
	Schedule, World, StageLabel
};
use bevy_ecs::prelude as ecs;


#[derive(StageLabel)]
pub struct OpencvTrackers;
impl OpencvTrackers {
	pub fn init_stage(schedule: &mut Schedule) -> &mut ecs::SystemStage{
		if schedule.get_stage::<ecs::SystemStage>(OpencvTrackers{}).is_none() {
			schedule.add_stage(OpencvTrackers, ecs::SystemStage::parallel());		
		}
		schedule.get_stage_mut::<ecs::SystemStage>(OpencvTrackers{}).unwrap()
	}
}

pub fn setup_entities(schedule: &mut Schedule, world: &mut World) {
	OpencvTrackers::init_stage(schedule)
				.add_system(camera_observer::CameraObservers::assignment_system)
				.add_system(camera_observer::CameraObservers::update_system);
			
	opencv_utilities::setup_entities(schedule, world);
	light_ball_trackers::setup_entities(schedule, world);
}