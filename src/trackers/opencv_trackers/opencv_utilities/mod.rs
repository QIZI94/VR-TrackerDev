pub mod window_preview;
pub mod frame_component;

use crate::trackers::opencv_trackers::OpencvTrackers;

use bevy_ecs::prelude::{
	Schedule, World
};

pub fn setup_entities(schedule: &mut Schedule, _: &mut World){
	OpencvTrackers::init_stage(schedule)
			.add_system(window_preview::WindowPreviewComponent::window_display_system)
			.add_system(window_preview::WindowPreviewComponent::window_layout_system);
	
}