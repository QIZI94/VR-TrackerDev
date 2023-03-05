use bevy_ecs::prelude as ecs;

use opencv::prelude::MatTraitConstManual;
use opencv::prelude::MatTraitConst;
use opencv::prelude as cv;
use crate::trackers::opencv_trackers::light_ball_trackers::light_ball_tracker::window_preview::WindowPreviewComponent;


use crate::entity_builder;
use crate::trackers::opencv_trackers::opencv_utilities;

use crate::trackers::tracker;
use crate::entity_builder::*;
use crate::state::*;

use crate::trackers::opencv_trackers::OpencvTrackers;
use crate::trackers::opencv_trackers::light_ball_trackers::light_ball_processing;


use opencv_utilities::{
	frame_component,
	window_preview
};

use std::any::type_name;
// ------- Light Ball Tracker ------- //

pub struct LightBallTrackerBuilder{
	colors: [ColorRangeHSV; 2]
}
impl LightBallTrackerBuilder {
	pub const fn default() -> Self{
		LightBallTrackerBuilder{colors: [
			// RED
			ColorRangeHSV{
				color_lower: opencv::core::VecN{0: [0.0, 2.00, 245., 0.]},
				color_upper: opencv::core::VecN{0: [76., 100., 252., 0.]},
			},
			// BLUE
			ColorRangeHSV{
				color_lower: opencv::core::VecN{0: [90., 100., 245., 0.]},
				color_upper: opencv::core::VecN{0: [255., 255., 255., 0.]},
			}


		]}
	}
}

impl EntityBuilder for LightBallTrackerBuilder{
	fn spawn(&self, command: &mut ecs::Commands) -> ecs::Entity{
		let entity = command.spawn(()).id();

		let mut window_component = window_preview::WindowPreviewComponent::default();
		window_component.window.set_title(type_name::<LightBallTracker>());
		//window_component.window.set_position(500, 550)

		command.entity(entity)
			.insert((
				tracker::TrackerData::default(),
				LightBallTracker::default(),
				frame_component::FrameComponent::default(),
				window_component,
				window_preview::WindowInLayout
			));
		
		
		return entity;
	}
	fn setup(&self, schedule: &mut ecs::Schedule, world: &mut ecs::World){
		for color in &self.colors {
			let light_ball = entity_builder::spawn_from_world(world, self);
			if let Some(mut tracker) = world.entity_mut(light_ball).get_mut::<LightBallTracker>(){
				tracker.color_range = color.clone();
			}
		}

		OpencvTrackers::init_stage(schedule)
			.add_system(||{println!("Ping")})
			.add_system(LightBallTracker::light_ball_tracker_update_system); 		
	}
	
}

type Color3d = opencv::core::Vec3i;
#[derive(ecs::Component, Default)]
pub struct LightBallTracker{
	state: StateWrapper<String, String>,
	counter: f32,
	color_range: ColorRangeHSV,
	calibration: LightBallCalibration
}
impl LightBallTracker {
	
	fn light_ball_tracker_update_system(
		mut commands: ecs::Commands,
		frame_query: ecs::Query<&frame_component::FrameComponent, (ecs::With<light_ball_processing::LightBallTrackerProcessing>, ecs::Changed<frame_component::FrameComponent>)>,
		mut tracker_query: ecs::Query<(ecs::Entity, &mut LightBallTracker, &mut frame_component::FrameComponent), ecs::Without<light_ball_processing::LightBallTrackerProcessing>>,
		mut debug_screen_space_view_entity: ecs::Local<Option<ecs::Entity>>
	){
		
		let mut debug_screen_space_frame: Option<cv::Mat> = None;
		for (entity, mut tracker, mut frame_mask) in tracker_query.iter_mut() {
			for frame_component in frame_query.iter() {
				let frame = frame_component.get_frame().lock().unwrap();

				if let Ok(mask) = Self::make_mask(&frame, &tracker.color_range){
					
					if let Ok(screen_space) = Self::compute_screen_space_position(&mask) {
						if let Some(position) = tracker.compute_position(&screen_space, &mask.size().unwrap()){
							commands.add(move |world: &mut ecs::World| {
								let mut entity_mut = world.entity_mut(entity);
								if let Some(mut data) = entity_mut.get_mut::<tracker::TrackerData>(){
									data.position = position;
								}
							});
						}

						if let None = debug_screen_space_frame{
							debug_screen_space_frame = Some(frame.clone())
						}
						
						Self::debug_screen_space(&mut debug_screen_space_frame.as_mut().unwrap(), &screen_space, &tracker.calibration, &tracker.color_range, 0.85)
							.unwrap_or_default();
					}
					
					
					// feed mask to frame mask of this 
					frame_mask.take(mask);
				}
			}
		}
		
		// debug view

		if let Some(debug_view) = *debug_screen_space_view_entity{
			let debug_frame_component = 
			frame_component::FrameComponent::new_with_frame(
				frame_component::FrameComponent::default_processing_function, debug_screen_space_frame.unwrap_or_default()
			);
			commands.entity(debug_view).insert(debug_frame_component);
		}
		else{
			let screen_space_window = window_preview::WindowPreviewBuilder{setup: 
				|window :&mut window_preview::WindowPreview|{
					window.set_title("Screen Space Debug")
				}
			}.build();
			*debug_screen_space_view_entity = Some(commands.spawn(
				(
					frame_component::FrameComponent::default(),
					WindowPreviewComponent{
						window: screen_space_window,
						processing_function: Some(light_ball_processing::LightBallTrackerProcessing::undo_preprocess_frame_color)
					},
					window_preview::WindowInLayout{}
				)
			).id());
		}
		
	}
	

	fn debug_screen_space(frame: &mut cv::Mat, screen_space: &EnclosingCircle, calibration: &LightBallCalibration, color_range: &ColorRangeHSV, font_size: f64) -> opencv::Result<()>{
		//cv::circle(debugProcessedFrame, screenSpace.position ,screenSpace.radius, cv::Scalar(255, 166, 0),4);
		type Pos = opencv::core::Point2i;
		opencv::imgproc::circle(
			frame,
			opencv::core::Point2f::new(screen_space.position.x, screen_space.position.y).to::<i32>().unwrap(),
			screen_space.radius as i32,
			color_range.color_lower,
			4,
			opencv::imgproc::LineTypes::LINE_8 as i32,
			0
		)?;
		
		let mut text_pos = screen_space.position.to::<i32>().unwrap();
		text_pos += Pos::new(screen_space.radius as i32,  -screen_space.radius as i32);
		let x_pos = "x: ".to_owned() + &screen_space.position.x.to_string();
		opencv::imgproc::put_text(
			frame,
			&x_pos,
			text_pos + Pos::new(0, (font_size * 30.) as i32),
			opencv::imgproc::FONT_HERSHEY_SIMPLEX,
			font_size,
			opencv::core::Scalar::new(255., 255., 255., 255.),
			2,
			8,
			false
		)?;
		let y_pos = "y: ".to_owned() + &screen_space.position.y.to_string();
		opencv::imgproc::put_text(
			frame,
			&y_pos,
			text_pos + Pos::new(0, (font_size * 60.) as i32),
			opencv::imgproc::FONT_HERSHEY_SIMPLEX,
			font_size,
			opencv::core::Scalar::new(255., 255., 255., 255.),
			2,
			8,
			false
		)?;
		let radius 
			= "r: ".to_owned() + &screen_space.radius.to_string();
		opencv::imgproc::put_text(
			frame,
			&radius,
			text_pos + Pos::new(0, (font_size * 90.) as i32),
			opencv::imgproc::FONT_HERSHEY_SIMPLEX,
			font_size,
			opencv::core::Scalar::new(255., 255., 255., 255.),
			2,
			8,
			false
		)?;
		let distance 
			= "Distance: ".to_string() + &(calibration.object_real_radius * calibration.focal_length / screen_space.radius as f64).to_string();
		opencv::imgproc::put_text(
			frame,
			&distance,
			text_pos + Pos::new(0, (font_size * 120.) as i32),
			opencv::imgproc::FONT_HERSHEY_SIMPLEX,
			font_size,
			opencv::core::Scalar::new(255., 255., 255., 255.),
			2,
			8,
			false
		)?;
		Ok(())
	}

	fn compute_position(&mut self, screen_space: &EnclosingCircle, screen_size: &opencv::core::Size) -> Option<tracker::Position>{
		if self.calibration.focal_length == 0.0{// temporary solution till, there will be a proper calibration process
			self.calibration
			 = LightBallCalibration::from_real_object_distance(screen_size, 115., 4., 19. /*232,6.5,20*/);
		}

		
		//https://www.pyimagesearch.com/2015/01/19/find-distance-camera-objectmarker-using-python-opencv/
		//see calibrateRealObjectDistance() function
		let distance     = self.calibration.object_real_radius * self.calibration.focal_length / screen_space.radius as f64;
		let azimuth      = (screen_size.width - screen_space.position.x as i32) as f64 * (self.calibration.angle_of_view.width / screen_size.width as f64);
		let inclination  = (screen_size.height - screen_space.position.y as i32) as f64 * (self.calibration.angle_of_view.height / screen_size.height as f64);

		//std::cout<<"dist:"<<distance<<";inc: "<<toDegrees(inclination)<<" deg;azi: "<<toDegrees(azimuth)<<" deg\n";
		/* Old solution not sure why it is not working (x axis is going along with y axis): from https://en.wikipedia.org/wiki/Spherical_coordinate_system
		x = distance * std::sin(inclination) * std::cos(azimuth);
		y = distance * std::sin(inclination) * std::sin(azimuth);
		z = distance * std::cos(inclination);
		*/
		//this works, treating each axis as if it was solved for [x;y] and [z;y] in 2D seperatly
		let x = distance * inclination.cos() * azimuth.sin();
		let y = distance * inclination.sin();
		let z = distance * inclination.cos();
		
		if (x * y * z) == std::f64::INFINITY {
			return None
		}

		Some(tracker::Position{x,y,z})
		/*self.counter += 6.6;
		let mut pos = tracker::Position::default();
		pos.y = self.counter;
		pos*/
		
	}

	fn make_mask(frame: &cv::Mat, color_range: &ColorRangeHSV) -> opencv::Result<cv::Mat> {
		//opencv::core::Scalar::new(75.,2.,100., 0.);
		//opencv::core::Scalar::new(78., 10.,250., 0.);
		//let color_lower = opencv::core::Scalar::new(0.,2.,245., 0.);//Color3d::from([75.,2.,100.]);
		//let color_upper = opencv::core::Scalar::new(76., 100.,252., 0.);//Color3d::from([78., 10.,255.]);

		let mut in_range = cv::Mat::default();
		opencv::core::in_range(frame, &color_range.color_lower, &color_range.color_upper, &mut in_range)?;

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
	

	//finds a biggets volume of pixels and make a enclosing circle around it
	fn compute_screen_space_position(mask: &cv::Mat) -> opencv::Result<EnclosingCircle>{
		let mut circle = EnclosingCircle::default();
		let mut contours = Contours::default();
		if !mask.empty() {
			opencv::imgproc::find_contours(
				mask,
				&mut contours,
				opencv::imgproc::RETR_EXTERNAL,
				opencv::imgproc::CHAIN_APPROX_SIMPLE,
				opencv::core::Point::default()
			)?;
			let max_area_index = Self::get_max_area_contour(&contours);
	 
			if let Some(area_index) = max_area_index {
				if let Ok(max_controur) = contours.get(area_index){
					opencv::imgproc::min_enclosing_circle(&max_controur, &mut circle.position, &mut circle.radius)?;
					//opencv::imgproc::min_area_rect(points)
				}
			}
		}
		Ok(circle)
	}

	
	fn get_max_area_contour(contours: &Contours) -> Option<usize> {
		//source https://stackoverflow.com/questions/46187563/finding-largest-contours-c
		let mut max_area = 0.0;
		let mut max_area_contour_index: Option<usize> = None;
	
		
		for (index, contour) in contours.iter().enumerate() {
			if let Ok(new_area) = opencv::imgproc::contour_area(&contour, false){
				if new_area > max_area {
					max_area = new_area;
					max_area_contour_index = Some(index);
				}
			}
		}
		return max_area_contour_index;
	}

}
#[derive(Default, Clone)]
struct ColorRangeHSV{
	color_lower: opencv::core::Scalar,
	color_upper: opencv::core::Scalar
}



#[derive(Default)]
struct LightBallCalibration{
	angle_of_view: opencv::core::Size2d,
	focal_length: f64,
	object_real_radius: f64
}

impl LightBallCalibration {
	fn from_real_object_distance(frame_size: &opencv::core::Size, pixel_radius: f64, real_radius: f64, real_distance: f64) -> Self{
		//https://www.pyimagesearch.com/2015/01/19/find-distance-camera-objectmarker-using-python-opencv/
		let mut calibration = LightBallCalibration::default();
		calibration.focal_length       = pixel_radius * real_distance / real_radius;
		calibration.object_real_radius  = real_radius;

		//https://en.wikipedia.org/wiki/Angle_of_view
		calibration.angle_of_view.width         = 2.0 * (frame_size.width as f64 / (2.0 * calibration.focal_length)).atan();
		calibration.angle_of_view.height        = 2.0 * (frame_size.height as f64 / (2.0 * calibration.focal_length)).atan();

		calibration
	}
}

#[derive(Default)]
struct EnclosingCircle{
	position: opencv::core::Point2f,
	radius: f32
}
type Contours = opencv::core::Vector<opencv::core::Vector<opencv::core::Point>>;
