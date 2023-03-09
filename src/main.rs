#![feature(drain_filter)]
#![allow(dead_code)]

mod entity_spawner;
mod trackers;
mod state;

fn main() -> std::io::Result<()>{

	let mut app = bevy::app::App::new();
	app
		.add_plugins(bevy::MinimalPlugins)
		.add_plugin(trackers::TrackersPlugin)
		.run();
	
		
/*

	let mut world = World::default();
	let mut schedule = Schedule::default();

	
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
	*/
	Ok(())
	
}
