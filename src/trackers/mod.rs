pub mod opencv_trackers;
pub mod tracker;

use bevy::ecs::prelude::Resource;


use::opencv::core::{
	Size2i, Point2i
};
#[derive(Resource)]
pub struct WindowLayout{
	origin: Point2i,
	offset: Point2i,
	boundaries: Size2i
}

impl WindowLayout {
	pub fn new(boundaries: Size2i) -> Self{
		WindowLayout{origin: Point2i::default(), offset: Point2i::default(), boundaries: boundaries}
	}
	pub fn new_with_origin(boundaries: Size2i, origin: Point2i) -> Self{
		WindowLayout{origin: origin, offset: origin, boundaries: boundaries}
	}
	pub fn get_new_poition(&mut self, window_size: &Size2i) -> Point2i{
		let new_position = self.offset.clone();

		self.offset.x += window_size.width + 1;
		if self.offset.x >  self.boundaries.width  {
			self.offset.y += window_size.height + 1;
			if self.offset.y > self.boundaries.height {
				self.offset = self.origin.clone();
			}
			else {
				self.offset.x = self.origin.x;
			}
		}
		new_position
	}

	pub fn set_origin(&mut self, origin: Point2i){
		self.origin = origin;
		self.offset = self.origin.clone();
	}

	pub fn set_boundaries(&mut self, boundaries: Size2i){
		self.boundaries = boundaries;
	}

	pub fn get_origin(&self) -> &Point2i{
		&self.origin 
	}

	pub fn get_offset(&self) -> &Point2i{
		&self.offset 
	}
	pub fn get_boundries(&self) -> &Size2i{
		&self.boundaries 
	}
	

}

pub struct TrackersPlugin;
impl bevy::app::Plugin for TrackersPlugin {
	fn build(&self, app: &mut bevy::prelude::App) {
		app
			.add_system(tracker::print_trackers_system);
		app.world.insert_resource(WindowLayout::new_with_origin(Size2i::new(2560, 1440), Point2i::new(30, 60)));
		opencv_trackers::setup_entities(app);
	}
}
