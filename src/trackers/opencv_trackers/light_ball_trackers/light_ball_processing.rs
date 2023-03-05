use bevy_ecs::prelude as ecs;
use bevy_ecs::query::With;



use opencv::prelude as cv;


use crate::trackers::opencv_trackers::opencv_utilities;

use crate::entity_builder::*;
use crate::trackers::opencv_trackers::camera_observer::*;


use crate::trackers::opencv_trackers::OpencvTrackers;

use opencv_utilities::{
	frame_component,
	window_preview
};

use std::any::type_name;

// ------- Light Ball Tracker Processing ------- //
pub struct LightBallTrackerProcessingBuilder;
impl EntityBuilder for LightBallTrackerProcessingBuilder{
	fn spawn(&self, commands: &mut ecs::Commands) -> ecs::Entity{
		let mut window_component = window_preview::WindowPreviewComponent::default();
		window_component.processing_function = Some(LightBallTrackerProcessing::undo_preprocess_frame_color);
		window_component.window.set_title(type_name::<LightBallTrackerProcessing>());

		commands.spawn( 
			(
				LightBallTrackerProcessing::default(),
				CameraObserverSubscriberComponent,
				frame_component::FrameComponent::new(LightBallTrackerProcessing::preprocess_frame),

				// debug components
				window_component,
				window_preview::WindowInLayout
			)
		).id()
	}
	fn setup(&self, schedule: &mut ecs::Schedule, _world: &mut ecs::World){
		OpencvTrackers::init_stage(schedule)
			.add_system(LightBallTrackerProcessing::observer_subscribe_system);
	}
}

#[derive(ecs::Component, Default)]
pub struct LightBallTrackerProcessing;
impl LightBallTrackerProcessing {

	fn observer_subscribe_system(
		mut commands: ecs::Commands,
		camera_observers: Option<ecs::ResMut<CameraObservers>>,
		query: ecs::Query<(ecs::Entity, &frame_component::FrameComponent), With<LightBallTrackerProcessing>>
	){
		if let Some(mut observers) =  camera_observers {
			for camera_observer in  &mut observers.list {
				let mut any_subscribed = false;
				for (entity, _) in query.iter() {
					if camera_observer.is_subscribed(&entity){
						any_subscribed = true;
					}
				}
				if any_subscribed == false {
					// add new subscribed frame component
					let new_entity = LightBallTrackerProcessingBuilder{}.spawn(&mut commands);
					camera_observer.subscribe(new_entity);
				}
			}
		}
	}

	pub fn preprocess_frame(dest: &mut cv::Mat, src: &cv::Mat) -> opencv::Result<()>{		
		let mut blured = cv::Mat::default();

		//double sigmaX, double sigmaY = (0.0), int borderType = 4
		opencv::imgproc::gaussian_blur(
			src,
			&mut blured,
			opencv::core::Size{width: 11, height: 11},
			0.0, 0.0,
			4
		)?;		

		opencv::imgproc::cvt_color(&blured, dest, opencv::imgproc::COLOR_BGR2HSV, 0)?;
		Ok(())
		
	}
	pub fn undo_preprocess_frame_color(dest: &mut cv::Mat, src: &cv::Mat) -> opencv::Result<()>{
		opencv::imgproc::cvt_color(&src, dest, opencv::imgproc::COLOR_HSV2BGR, 0)?;
		Ok(())
	}

}
