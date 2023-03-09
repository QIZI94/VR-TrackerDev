pub mod window_preview;
pub mod frame_component;

use crate::trackers::opencv_trackers::OpencvTrackers;

pub fn setup_entities(app: &mut bevy::prelude::App){
	OpencvTrackers::init_schedule(app)
		.add_system(window_preview::WindowPreviewComponent::window_display_system)
		.add_system(window_preview::WindowPreviewComponent::window_layout_system);
}