use bevy_ecs::prelude::*;
#[derive(Default, Debug)]
pub struct Position{
	pub x: f32,
	pub y: f32,
	pub z: f32
}
#[derive(Default)]
pub struct Rotation(Position);

#[derive(Component, Default)]
pub struct TrackerData { 
	pub position: Position,
	pub rotation: Rotation
}

fn print_tracker(tracker: &TrackerData){
	let p = &tracker.position;
	println!("Position: [{},{},{}]", p.x, p.y, p.z);
}

pub fn print_trackers_system(query: Query<&TrackerData, Changed<TrackerData>>){
	for tracker in &query {
		print_tracker(tracker);
	}
}

