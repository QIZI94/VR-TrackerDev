
use std::any::type_name;
use std::result;

use bevy_ecs::prelude::*;
use bevy_ecs::world;

use crate::trackers::tracker::*;
use crate::state;



use opencv::{
    prelude::*,
    videoio,
    highgui
}; 

use crate::trackers::opencv_trackers::opencv_utilities::{
	window_preview,
	frame_component
};


#[derive(StageLabel)]
pub struct CameraObserversLabel;

#[derive(Component)]
pub struct CameraObserverSubscriberComponent;

#[derive(Resource)]
pub struct CameraObservers{
	pub list: Vec<CameraObserver>
}



impl CameraObservers{
	pub fn update_system( camera_observers: Option<ResMut<CameraObservers>>, mut query: Query<&mut frame_component::FrameComponent, With<CameraObserverSubscriberComponent>>){
		if camera_observers.is_some(){
			
			//query.get(entity)

			for camera_observer in  &mut camera_observers.unwrap().as_mut().list {
				let mut new_frame = Mat::default();
				camera_observer.update(&mut new_frame);

				for entity in &camera_observer.subscribed_entities {
					let frame_component_result = query.get_mut(*entity);
					if let Ok(mut frame_component) = frame_component_result {
						frame_component.apply(&new_frame);
					} 
					
				}
				
				//camera_observer.update();
				
			}
		}
	}

}

impl world::FromWorld for CameraObservers{
	fn from_world(world: &mut World) -> Self {
		let mut camera = CameraObserver::default();
		
		let window_builder = window_preview::WindowPreviewBuilder{
			setup: |window: &mut window_preview::WindowPreview| {
				window.set_title(type_name::<CameraObserver>());
			}
		};

		let entity = world.spawn(
			(
				frame_component::FrameComponent::default(),
				window_preview::WindowPreviewComponent{window: window_builder.build(), processing_function: None},
				CameraObserverSubscriberComponent
			)
		);
		camera.subscribed_entities.insert(entity.id());
		CameraObservers{list: vec![camera]}
	}
}

/**
 * 
 */
#[derive(Default)]
pub struct CameraObserver {
	state: state::State<OpencvCameraObserver, opencv::Error>,
	subscribed_entities: std::collections::HashSet<Entity>
}



impl CameraObserver{

	pub fn subscribe(&mut self, entity: Entity){
		self.subscribed_entities.insert(entity);
	}

	pub fn get_subscribers(&self) -> &std::collections::HashSet<Entity>{
		&self.subscribed_entities
	}

	pub fn is_subscribed(&self, entity: &Entity) -> bool{
		self.subscribed_entities.contains(entity)
	}

// private:

	fn init_opencv_observer() -> opencv::Result<OpencvCameraObserver>{		
		// Open the web-camera (assuming you have one)
		let mut camera = videoio::VideoCapture::from_file("/dev/video0", videoio::CAP_ANY)?; //videoio::VideoCapture::new(0, videoio::CAP_ANY)?;
		camera.set(videoio::CAP_PROP_FRAME_WIDTH, 1024.)?;
		camera.set(videoio::CAP_PROP_FRAME_HEIGHT,768.)?;
		camera.set(videoio::CAP_PROP_FPS, 30.)?;
		

		let opencv_observer = OpencvCameraObserver::new(camera);
		Ok(opencv_observer)
	}

	fn update_frame(observer: &mut OpencvCameraObserver, frame: &mut Mat) -> opencv::Result<()>{
		let mut cam = observer.video_capture.lock().unwrap();
		cam.read(frame)?;

		Ok(())
	}

	fn update(&mut self, frame: &mut Mat){
		use state::*;
		match &mut self.state {
			State::None => {
				self.state.restart_with(Ok(OpencvCameraObserver::default()));
			},
			State::Start(_) => {
				self.state.pass(CameraObserver::init_opencv_observer());
			},
			State::Run(opencv_result) => {
				let result = Self::update_frame(opencv_result.as_mut().unwrap(), frame);
				match result {
					Ok(_) => {},
					Err(err) => {
						self.state.failed(err);
					}
				}
			},
			State::Stop(opencv_result) => {
				if let Err(error) = opencv_result{
					println!("Error: {}", error);
				}				
				self.state.skip();
			}
			_ => {}
		}
		
	}

	

}

 
struct OpencvCameraObserver {
	video_capture: std::sync::Mutex<videoio::VideoCapture>
}

impl OpencvCameraObserver {
	fn new(video_capture: videoio::VideoCapture) -> Self {
		OpencvCameraObserver{video_capture: std::sync::Mutex::new(video_capture)}
	}
}

impl Default for OpencvCameraObserver {
	fn default() -> Self {
		OpencvCameraObserver{video_capture: std::sync::Mutex::new(videoio::VideoCapture::default().unwrap())}
	}
}
