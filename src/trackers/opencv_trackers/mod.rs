pub mod camera;
pub mod camera_observer;
pub mod opencv_utilities;
pub mod light_ball_trackers;

use bevy::ecs::prelude as ecs;


#[derive(bevy::ecs::schedule::ScheduleLabel,Debug, PartialEq, Eq, Hash, Clone)]
pub struct OpencvTrackers;
impl OpencvTrackers {
	pub fn init_schedule(app: &mut bevy::prelude::App) -> &mut ecs::Schedule{
		if let None = app.get_schedule_mut(OpencvTrackers){
			app.init_schedule(OpencvTrackers);
			let schedule = app.get_schedule_mut(OpencvTrackers).unwrap();
			schedule.set_executor_kind(bevy::ecs::schedule::ExecutorKind::MultiThreaded);
		}
		app.get_schedule_mut(OpencvTrackers).unwrap()
	}
	fn run_schedule(world: &mut ecs::World){
		world.run_schedule(OpencvTrackers);
	}
}

pub fn setup_entities(app: &mut bevy::prelude::App) {
	// TODO make it run before debug render phase and before exposing it to OpenXR
	app.add_system(OpencvTrackers::run_schedule);

	OpencvTrackers::init_schedule(app)
		.add_system(camera_observer::CameraObservers::assignment_system)
		.add_system(camera_observer::CameraObservers::update_system);
			
	opencv_utilities::setup_entities(app);
	light_ball_trackers::setup_entities(app);
}