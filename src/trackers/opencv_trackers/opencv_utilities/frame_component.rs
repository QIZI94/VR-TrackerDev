use opencv::prelude as cv;
use bevy::ecs::prelude as ecs;

pub type ProcessingFunction = fn(dest: &mut cv::Mat, src: &cv::Mat) -> opencv::Result<()>;


#[derive(ecs::Component)]
pub struct FrameComponent{
	frame: std::sync::Mutex<cv::Mat>,
	pub processing_function: ProcessingFunction 
}


impl FrameComponent {
	pub fn new(processing_function: ProcessingFunction) -> Self{
		FrameComponent{
			frame: std::sync::Mutex::new(cv::Mat::default()),
			processing_function: processing_function
		}
	}
	pub fn new_with_frame(processing_function: ProcessingFunction, frame: cv::Mat) -> Self{
		FrameComponent{
			frame: std::sync::Mutex::new(frame),
			processing_function: processing_function
		}
	}

	pub fn get_frame(&self) -> &std::sync::Mutex<cv::Mat> {
		&self.frame
	}

	pub fn apply(&mut self, other_frame: &cv::Mat) -> opencv::Result<()>{
		let mut frame = self.frame.lock().unwrap();
		(self.processing_function)(&mut frame, other_frame)
	}

	pub fn process(&self, other_frame: &cv::Mat) -> opencv::Result<cv::Mat>{
		let mut frame = cv::Mat::default();
		if let Err(error) = (self.processing_function)(&mut frame, other_frame){
			return Err(error)
		}
		Ok(frame)
	}

	pub fn take(&mut self, other_frame: cv::Mat) {
		let mut frame = self.frame.lock().unwrap();
		*frame = other_frame;
	}

	pub fn default_processing_function(dest: &mut cv::Mat, src: &cv::Mat) -> opencv::Result<()>{
		
		dest.clone_from(src);
		Ok(())
	}
}

impl Default for FrameComponent {
	fn default() -> Self {
		FrameComponent{
			frame: std::sync::Mutex::new(cv::Mat::default()),
			processing_function: FrameComponent::default_processing_function
		}
	}
}
