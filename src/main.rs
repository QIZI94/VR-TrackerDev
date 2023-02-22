#![feature(drain_filter)]
#![allow(dead_code)]
use bevy_ecs::prelude::*;

use system_startup::Application;

mod system_startup;
mod entity_builder;
mod trackers;


mod state;





fn main() -> std::io::Result<()>{
	let mut world = World::default();
	let mut schedule = Schedule::default();
	//system_startup::setup_entities(&mut schedule, &mut world);
	
	/*schedule.add_stage("test", SystemStage::parallel()
		.with_system(tracker::print_trackers_system));*/
	
	let mut app = Application::default();


	
	while app.run(&mut world, &mut schedule){
		let resource = world.get_resource_mut::<system_startup::ApplicationControls>();
		if resource.is_some() {
			//resource.unwrap().as_mut().request_stop(Ok("Stop".to_owned()));
		}
		println!("lol");
	}

	if app.get_result().is_some(){
		match app.get_result().unwrap() {
			Ok(ok) => {
				println!("Success result: {}", ok);
			}
			Err(err) => {
				println!("Error result: {}", err);
			}
		}
	}
	Ok(())
	
}
