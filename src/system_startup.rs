use bevy_ecs::prelude::*;
use crate::{state, trackers};


#[derive(Resource, Default)]
pub struct ApplicationControls{
	result: Option<state::PreviousStateResult<String, String>>
}
impl ApplicationControls {
	pub fn request_stop(&mut self, new_previous_result: state::PreviousStateResult<String, String>){
		if self.result.is_none() {
			self.result = Some(new_previous_result);
		}
	}
}

#[derive(Default)]
pub struct Application {
	state: state::State<String, String>
}

impl Application{

	pub fn run(&mut self, world: &mut World, schedule: &mut Schedule) -> bool{
		self.state_handler(world, schedule);
		!self.state.is_done()
	}

	

	fn state_handler(&mut self, world: &mut World, schedule: &mut Schedule){
		use state::*;
		match &mut self.state {
			State::None => {
				self.state.restart_with(Ok("".to_owned()));
			},
			State::Start(_) => {
				world.init_resource::<ApplicationControls>();
				Self::setup_entities(schedule, world);
				self.state.skip();
			},
			State::Run(_) => {
				let mut application_controls = world.resource_mut::<ApplicationControls>();
				if application_controls.result.is_none() {
					schedule.run(world);
				}
				else {
					self.state.pass(core::mem::take(&mut application_controls.result).unwrap());
				}
			},
			State::Stop(_) => {
				self.state.skip();
			}
			_ => {}
		}
	}


	pub fn get_result(&self) -> Option<&state::PreviousStateResult<String, String>>{
		if self.state.is_done() {
			let result = self.state.get_last_result();
			return result;
		}
		None
	}

	fn setup_entities(schedule: &mut Schedule, world: &mut World){
		
		trackers::setup_entities(schedule, world);
	}

}
