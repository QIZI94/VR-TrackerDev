use linuxvideo::Device;
use std::collections::hash_map;
use std::cmp::Ordering;

fn list_device(device: Device) -> std::io::Result<()> {
    let caps = device.capabilities()?;
	
    println!("- {}: {}", device.path()?.display(), caps.card());
    println!("  driver: {}", caps.driver());
    println!("  bus info: {}", caps.bus_info());
    println!("  all capabilities:    {:?}", caps.all_capabilities());
    println!("  avail. capabilities: {:?}", caps.device_capabilities());

    Ok(())
}




pub struct CameraDevice{
	pub name: String,
	pub path: String
}
type UniqueCameraDeviceList = hash_map::HashMap<String, CameraDevice>;

impl CameraDevice {


//# static
	fn assign_device(device_list: &mut UniqueCameraDeviceList, device: Device) -> std::io::Result<()> {
		let caps = device.capabilities()?;
		let path = device.path()?.display().to_string();
		let bus = caps.bus_info().to_string();
		let name = caps.card().to_string();
		if let Some(device) = device_list.get_mut(&bus){
			if device.path.cmp(&path) == Ordering::Greater{
				device.path = path;
			}
		}
		else {
			device_list.insert(
				bus,
				CameraDevice{
					name: name,
					path: path
				}
			);
		}
		Ok(())
		
		/*device.path()?.display()
		
		println!("- {}: {}", device.path()?.display(), caps.card());
		println!("  driver: {}", caps.driver());
		println!("  bus info: {}", caps.bus_info());
		println!("  all capabilities:    {:?}", caps.all_capabilities());
		println!("  avail. capabilities: {:?}", caps.device_capabilities());

		Ok(())*/
	}
	
	pub fn list_unique_devices() -> std::io::Result<UniqueCameraDeviceList>{
		let mut device_list = UniqueCameraDeviceList::default();
		for res in linuxvideo::list()? {
			if let Ok(device) = res {
				Self::assign_device(&mut device_list, device)?;
			}
		}

		
		
		Ok(device_list)
	}
}

