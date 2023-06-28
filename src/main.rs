#![feature(drain_filter)]
#![allow(dead_code)]

use bevy::prelude::*;

mod entity_spawner;
mod trackers;
mod state;

fn setup(mut commands: Commands){
	commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

	
}

fn spawn_basic_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
	let plane = PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Plane { size: 5.0, .. Default::default()})),
        material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
        ..default()
    };
    let cube = PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        material: materials.add(Color::rgb(0.67, 0.84, 0.92).into()),
        transform: Transform::from_xyz(0.0, 0.5, 0.0),
        ..default()
    };
	
	commands.spawn(plane);
    commands.spawn(cube);
}

fn main() -> std::io::Result<()>{

	let mut app = bevy::app::App::new();
	app
		.add_plugins(bevy::DefaultPlugins)
		.add_startup_system(setup)
		.add_startup_system(spawn_basic_scene)
		.add_plugin(trackers::TrackersPlugin)
		.add_plugin(bevy_editor_pls::EditorPlugin)
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
