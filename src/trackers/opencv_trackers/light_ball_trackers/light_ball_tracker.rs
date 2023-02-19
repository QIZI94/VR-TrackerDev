use bevy_ecs::prelude as ecs;
use bevy_ecs::query::With;
use bevy_ecs::system::CommandQueue;
use bevy_ecs::world;

use opencv::prelude as cv;

use crate::entity_builder;
use crate::trackers::opencv_trackers::opencv_utilities;

use crate::trackers::tracker;
use crate::entity_builder::*;
use crate::trackers::opencv_trackers::camera_observer::*;
use crate::state::*;

use crate::trackers::opencv_trackers::OpencvTrackers;



use opencv_utilities::{
	frame_component,
	window_preview
};
use std::any::type_name;

pub struct LightBallTrackerBuilder;
impl EntityBuilder for LightBallTrackerBuilder{
	fn spawn(&self, command: &mut ecs::Commands) -> ecs::Entity{
		let entity = command.spawn(()).id();

		let mut window_component = window_preview::WindowPreviewComponent::default();
		window_component.window.set_title(type_name::<LightBallTracker>());

		command.entity(entity)
			.insert((
				tracker::TrackerData::default(),
				LightBallTracker::default(),
				frame_component::FrameComponent::default(),
				window_component
			));
		command.init_resource::<CameraObservers>();
		
		return entity;
	}
	fn setup(&self, schedule: &mut ecs::Schedule, world: &mut ecs::World){
		entity_builder::spawn_from_world(world, self);
		if schedule.get_stage::<ecs::SystemStage>(CameraObserversLabel{}).is_none() {
			
			/*schedule.add_stage(CameraObserversLabel, ecs::SystemStage::parallel()
				.with_system(CameraObservers::update_system)
				.with_system(||{println!("Ping")})
				.with_system(LightBallTracker::light_ball_tracker_update_system)

			);*/

			/*OpencvTrackers::init_stage(
				schedule, 
				|stage|{
					stage
						.add_system(CameraObservers::update_system)
						.add_system(||{println!("Ping")})
						.add_system(LightBallTracker::light_ball_tracker_update_system);
				} 
			);*/
			OpencvTrackers::init_stage(schedule)
				.add_system(||{println!("Ping")})
				.add_system(LightBallTracker::light_ball_tracker_update_system); 
			
			
			
		}
		/*use crate::tracker_impl::opencv_trackers::OpencvTrackers;
		OpencvTrackers::init_stage(schedule, |stage|{
			stage.
			with_system(CameraObservers::update_system)
			.with_system(||{println!("Ping")})
			.with_system(LightBallTracker::light_ball_tracker_update_system);
			}
		);*/
	}
	
}
#[derive(ecs::StageLabel)]
pub struct LightBallTrackerProcessingBuilder;
impl EntityBuilder for LightBallTrackerProcessingBuilder{
	fn spawn(&self, commands: &mut ecs::Commands) -> ecs::Entity{
		let mut window_component = window_preview::WindowPreviewComponent::default();
		window_component.processing_function = Some(LightBallTrackerProcessing::undo_postprocess_frame_color);
		window_component.window.set_title(type_name::<LightBallTrackerProcessing>());

		commands.spawn( 
			(
				LightBallTrackerProcessing::default(),
				CameraObserverSubscriberComponent,
				frame_component::FrameComponent::new(LightBallTrackerProcessing::postprocess_frame),

				// debug components
				window_component,
			)
		).id()
	}
	fn setup(&self, schedule: &mut ecs::Schedule, world: &mut ecs::World){
		OpencvTrackers::init_stage(schedule)
			.add_system(LightBallTrackerProcessing::observer_subscribe_system);
		/*
		if schedule.get_stage::<ecs::SystemStage>(LightBallTrackerProcessingBuilder{}).is_none() {
			schedule.add_stage(LightBallTrackerProcessingBuilder, ecs::SystemStage::parallel()
				.with_system(LightBallTrackerProcessing::observer_subscribe_system)
			);
		}*/
	}
}




#[derive(ecs::Component, Default)]
struct LightBallTrackerProcessing;
impl LightBallTrackerProcessing {

	fn observer_subscribe_system(
		mut commands: ecs::Commands,
		camera_observers: Option<ecs::ResMut<CameraObservers>>,
		query: ecs::Query<(ecs::Entity, &frame_component::FrameComponent), With<LightBallTrackerProcessing>>
	){
		if camera_observers.is_some(){
			for camera_observer in  &mut camera_observers.unwrap().as_mut().list {
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

	fn postprocess_frame(dest: &mut cv::Mat, src: &cv::Mat) -> opencv::Result<()>{		
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
	fn undo_postprocess_frame_color(dest: &mut cv::Mat, src: &cv::Mat) -> opencv::Result<()>{
		opencv::imgproc::cvt_color(&src, dest, opencv::imgproc::COLOR_HSV2BGR, 0)?;
		Ok(())
	}

}

type Color3d = opencv::core::Vec3i;
#[derive(ecs::Component, Default)]
struct LightBallTracker{
	state: StateWrapper<String, String>,
	counter: f32
}
impl LightBallTracker {
	
	fn light_ball_tracker_update_system(
		mut commands: ecs::Commands,
		frame_query: ecs::Query<&frame_component::FrameComponent, (ecs::With<LightBallTrackerProcessing>, ecs::Changed<frame_component::FrameComponent>)>,
		mut tracker_query: ecs::Query<(ecs::Entity, &mut LightBallTracker, &mut frame_component::FrameComponent), ecs::Without<LightBallTrackerProcessing>>
	){
		for (entity, mut tracker, mut frame_mask) in tracker_query.iter_mut() {
			for frame_component in frame_query.iter(){
				let frame = frame_component.get_frame().lock().unwrap();

				if let Ok(mask) = Self::make_mask(&frame){
					
					if let Ok(_) = frame_mask.apply(&mask){}
					let position = tracker.compute_position(&frame, &mask);

					
					commands.add(move |world: &mut ecs::World|{
						let mut entity_mut = world.entity_mut(entity);
						if let Some(mut data) = entity_mut.get_mut::<tracker::TrackerData>(){
							data.position = position;
						}
						
					});
				}
			}
		}
	}
	
	fn compute_position(&mut self, frame: &cv::Mat, frame_mask: &cv::Mat) -> tracker::Position{
		
		self.counter += 6.6;
		let mut pos = tracker::Position::default();
		pos.y = self.counter;
		pos
		
	}

	fn make_mask(frame: &cv::Mat) -> opencv::Result<cv::Mat> {
		//opencv::core::Scalar::new(75.,2.,100., 0.);
		//opencv::core::Scalar::new(78., 10.,250., 0.);
		let color_lower = opencv::core::Scalar::new(75.,2.,250., 0.);//Color3d::from([75.,2.,100.]);
		let color_upper = opencv::core::Scalar::new(78., 10.,255., 0.);//Color3d::from([78., 10.,255.]);

		let mut in_range = cv::Mat::default();
		opencv::core::in_range(frame, &color_lower, &color_upper, &mut in_range)?;

		let anchor = opencv::core::Point::new(-1, -1);

		let mut eroded = cv::Mat::default();
		let border_value = opencv::imgproc::morphology_default_border_value()?;
		opencv::imgproc::erode(
			&in_range,
			&mut eroded,
			&cv::Mat::default(),
			anchor.clone(),
			2,
			opencv::core::BORDER_CONSTANT,
			border_value.clone()
		)?;
		
		let mut mask = cv::Mat::default();
		opencv::imgproc::dilate(
			&eroded,
			&mut mask,
			&cv::Mat::default(),
			anchor.clone(),
			2,
			opencv::core::BORDER_CONSTANT,
			border_value.clone()
		)?;
		//mask.clone_from(frame);
		
		Ok(mask)
	}

}


