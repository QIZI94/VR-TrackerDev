use std::any::type_name;


use bevy::ecs::prelude::*;
use bevy::ecs::world;

use crate::entity_spawner::EntitySpawner;
use crate::state;
use crate::trackers::opencv_trackers::camera;


use opencv::{
    prelude::*,
    videoio
}; 

use crate::trackers::opencv_trackers::opencv_utilities::{
	window_preview,
	frame_component
};



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
						if let Ok(_) = frame_component.apply(&new_frame){
							// get rid of warning(maybe handle this error)					
						}
					} 
				}
				
				//camera_observer.update();
				
			}
		}
		
	}

	pub fn assignment_system(mut commands:  Commands, camera_observers: Option<ResMut<CameraObservers>>){
		if let Some(mut observers) =  camera_observers {
			let mut  camera_list = camera::CameraDevice::list_unique_devices().unwrap();
			for camera_observer in &mut observers.list{
				if camera_list.contains_key(&camera_observer.bus){
					camera_list.remove(&camera_observer.bus);
				}
				else if !camera_observer.state.is_done() {
					let old_state= std::mem::replace(&mut camera_observer.state, state::State::None);
					match old_state {
						state::State::Start(result) => {
							camera_observer.state = state::State::Stop(result);
						},
						state::State::Run(result) => {
							camera_observer.state = state::State::Stop(result);
						},
						state::State::Stop(result) => {
							camera_observer.state = state::State::Stop(result);
						}
						state::State::Done(result) => {
							camera_observer.state = state::State::Done(result);
						}
						_ => {}
					}
				}

				
			}
			for (bus , new_unassigned_camera) in camera_list{
				let entity = CameraPreviewBuilder{}.spawn(&mut commands);
				let mut camera = CameraObserver::default();
				camera.bus = bus; 
				camera.path = new_unassigned_camera.path;
				camera.subscribed_entities.insert(entity);
				commands.add(
					move |world: &mut World| {
						let mut observers_mut = world.get_resource_mut::<CameraObservers>().unwrap();
						observers_mut.list.push(camera)
					} 
				);
			}

			commands.add(
				move |world: &mut World| {
					let mut observers_mut = world.get_resource_mut::<CameraObservers>().unwrap();

					let removed_observers = observers_mut.list.drain_filter(|observer| observer.state.is_done()).collect::<Vec<_>>();
					for observer in removed_observers.iter(){
						for entity in observer.subscribed_entities.iter(){
							world.despawn(entity.clone());
						}
					}
				} 
			);
		}
		else {
			commands.init_resource::<CameraObservers>();
		}
	}

}

impl world::FromWorld for CameraObservers{
	fn from_world(_world: &mut World) -> Self {
		
		CameraObservers{list: vec![]}
	}
}

/**
 * 
 */
#[derive(Default)]
pub struct CameraObserver {
	state: state::State<OpencvCameraObserver, opencv::Error>,
	bus: String,
	path: String,
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

	fn init_opencv_observer(filepath: &str) -> opencv::Result<OpencvCameraObserver>{		
		// Open the web-camera (assuming you have one)
		let mut camera = videoio::VideoCapture::from_file(filepath, videoio::CAP_ANY)?; //videoio::VideoCapture::new(0, videoio::CAP_ANY)?;
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
				self.state.pass(CameraObserver::init_opencv_observer(&self.path));
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


struct CameraPreviewBuilder;

impl EntitySpawner for CameraPreviewBuilder{
	fn spawn(&self, commands: &mut Commands) -> Entity {
		let window_builder = window_preview::WindowPreviewBuilder{
			setup: |window: &mut window_preview::WindowPreview| {
				window.set_title(type_name::<CameraObserver>());
			}
		};
		commands.spawn(
			(
				frame_component::FrameComponent::default(),
				window_preview::WindowPreviewComponent{window: window_builder.build(), processing_function: None},
				window_preview::WindowInLayout,
				CameraObserverSubscriberComponent
			)
		).id()
	}
}
