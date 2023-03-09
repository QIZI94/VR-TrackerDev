use opencv::{
    prelude as cv,
    highgui
}; 


use bevy::ecs::prelude as ecs;
use uuid::Uuid;
use crate::trackers::opencv_trackers::opencv_utilities::frame_component;




pub struct WindowPreviewBuilder{
	pub setup: fn(&mut WindowPreview)
}
impl WindowPreviewBuilder{
	pub fn build(&self) -> WindowPreview{
		let mut window = WindowPreview::default();
		(self.setup)(&mut window);
		window
	}
}

pub struct WindowPreview{
    id_str:	String,
	title: String,
	position: opencv::core::Point2i,
	size: opencv::core::Size2i,
}

impl Default for WindowPreview{
	fn default() -> Self {
		// let it panic
		highgui::start_window_thread().unwrap();

		let id =  Uuid::new_v4().to_string();
		highgui::named_window(&id, highgui::WINDOW_NORMAL).unwrap();
		

		let mut window = WindowPreview { id_str: id, title: String::default(), position: opencv::core::Point2i::default(), size: opencv::core::Size2i::default()};
		window.set_position(0, 0);
		window.set_size(640, 480);
		window.set_title("Window Preview");
		
		window
	}
}

impl  Drop for  WindowPreview {
	fn drop(&mut self) {
		if let Err(error) = highgui::destroy_window(&self.id_str) {
			println!("While destroying window: {}", error);
		} ;
		
	}
}

impl WindowPreview{

	pub fn display(&self, frame: &opencv::prelude::Mat) -> opencv::Result<()>{
		highgui::imshow(&self.id_str, &*frame)
	}

	pub fn set_title(&mut self, title: &str){
		self.title = title.to_owned();
		highgui::set_window_title(& self.id_str, &title).unwrap();
	}

	pub fn set_position(&mut self, x: i32, y: i32){
		self.position.x = x;
		self.position.y = y;
		highgui::move_window(&self.id_str, self.position.x, self.position.y).unwrap();
	}

	pub fn set_size(&mut self, width: i32, height: i32){
		self.size.width = width;
		self.size.height = height;
		highgui::resize_window(&self.id_str, width, height).unwrap();
	}

	pub fn set_property(&mut self, property: highgui::WindowPropertyFlags, prop_value: f64){
		highgui::set_window_property(&self.id_str, property as i32, prop_value).unwrap();
	}

	pub fn get_title(&self) -> &String {
		&self.title
	}

	pub fn get_property(&mut self, property: highgui::WindowPropertyFlags) -> Result<f64, opencv::Error>{
		highgui::get_window_property(&self.id_str, property as i32)
	}

	pub fn get_image_rect(&self) -> Result<opencv::core::Rect_<i32>, opencv::Error>{
		highgui::get_window_image_rect(&self.id_str)
	}

	pub fn get_position(&self) -> &opencv::core::Point2i{
		&self.position
	}

	pub fn get_size(&self) -> &opencv::core::Size2i{
		&self.size
	}

	

	fn select_region_block(){

	}
}


#[derive(ecs::Component)]
pub struct WindowInLayout;

#[derive(ecs::Component, Default)]
pub struct WindowPreviewComponent{
	pub window: WindowPreview,
	pub processing_function: Option<frame_component::ProcessingFunction>
}

impl WindowPreviewComponent {
	pub fn window_display_system(query: ecs::Query<(&WindowPreviewComponent, &frame_component::FrameComponent), ecs::Changed<frame_component::FrameComponent>>){
		for (window_component, frame_component) in query.iter(){
			let frame = frame_component.get_frame().lock().unwrap();
			#[allow(unused_assignments)]
			let mut result: opencv::Result<()> = Ok(());
			if let Some(processing_function) = window_component.processing_function{
			    let mut processed_frame = cv::Mat::default();
				result = match processing_function(&mut processed_frame, &frame){
					Ok(_) => window_component.window.display(&processed_frame),
					Err(error) => Err(error)
				};		
			}
			else{
				result = window_component.window.display(&frame);
			}

			if let Err(err) = result{
				println!("{}",err.message)
			}
		}
	}
	pub fn window_layout_system(
		mut commands: ecs::Commands,
		window_layout: Option<ecs::ResMut<crate::trackers::WindowLayout>>,
		query: ecs::Query<(ecs::Entity, &WindowPreviewComponent), ecs::With<WindowInLayout>>
	){
		if let Some(mut layout) = window_layout {
			for (entity, window_component) in query.iter(){
				let position = layout.get_new_poition(window_component.window.get_size());

				commands.add(
					move |world: &mut ecs::World|{
						if let Some(mut entity_mut) = world.get_entity_mut(entity){
							if let Some(mut component) = entity_mut.get_mut::<WindowPreviewComponent>(){
								component.window.set_position(position.x, position.y);
							}
							entity_mut.remove::<WindowInLayout>();
						}
					}
				
				);
			}
		}
	}
}