use bevy_ecs::prelude::*;

use system_startup::Application;

mod system_startup;
mod entity_builder;
mod trackers;


mod state;


use linuxvideo::Device;

fn list_device(device: Device) -> std::io::Result<()> {
    let caps = device.capabilities()?;
    println!("- {}: {}", device.path()?.display(), caps.card());
    println!("  driver: {}", caps.driver());
    println!("  bus info: {}", caps.bus_info());
    println!("  all capabilities:    {:?}", caps.all_capabilities());
    println!("  avail. capabilities: {:?}", caps.device_capabilities());

    Ok(())
}

fn main() -> std::io::Result<()>{
	let mut world = World::default();
	let mut schedule = Schedule::default();
	//system_startup::setup_entities(&mut schedule, &mut world);
	
	/*schedule.add_stage("test", SystemStage::parallel()
		.with_system(tracker::print_trackers_system));*/
	
	let mut app = Application::default();


	for res in linuxvideo::list()? {
        match res.and_then(|device| list_device(device)) {
            Ok(()) => {}
            Err(e) => {
                eprintln!("skipping device due to error: {}", e);
            }
        }
    }
	//return Ok(());
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
