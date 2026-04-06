mod test;

use std::io::Read;
use std::sync::mpsc;
use std::time::Duration;
use std::{io, thread};
use vulkano::VulkanLibrary;
use vulkano::instance::{Instance, InstanceCreateFlags, InstanceCreateInfo};

fn main() {
	let library = VulkanLibrary::new().expect("no local Vulkan library/DLL");
	let instance = Instance::new(
		library,
		InstanceCreateInfo {
			flags: InstanceCreateFlags::ENUMERATE_PORTABILITY,
			..Default::default()
		},
	)
		.expect("failed to create instance");
	
	let physical_device = instance
		.enumerate_physical_devices()
		.expect("could not enumerate devices")
		.next()
		.expect("no devices available");
	
	println!("{:?}", physical_device.properties().device_name);
	
	for family in physical_device.queue_family_properties() {
		println!(
			"Found a queue family with {:?} queue(s)",
			family.queue_count
		);
	}
	
	use vulkano::device::QueueFlags;
	
	let queue_family_index = physical_device
		.queue_family_properties()
		.iter()
		.position(|queue_family_properties| {
			queue_family_properties
				.queue_flags
				.contains(QueueFlags::GRAPHICS)
		})
		.expect("couldn't find a graphical queue family") as u32;
	
	use vulkano::device::{Device, DeviceCreateInfo, QueueCreateInfo};
	
	let (device, mut queues) = Device::new(
		physical_device,
		DeviceCreateInfo {
			queue_create_infos: vec![QueueCreateInfo {
				queue_family_index,
				..Default::default()
			}],
			..Default::default()
		},
	)
		.expect("failed to create device");
	
	test();
}

fn test() {
	let num = 0;
	println!(
		"{}",
		(|mut a: i32| -> i32 {
			a += 1;
			a
		})(num)
	); //1
	
	println!("{:?}", std::env::args());
	
	let a = 10;
	let ref b = a;
	let vec = vec![1, 2, 3];
	println!("{}", *b); //10
	
	let em = [1, 2, 3, 4, 5];
	
	let c = 100;
	println!("{}", c); //100
	
	let d = move || c;
	println!("{}", d()); //100
	
	let c = 200;
	println!("{}", c);
	//200
	let handle = thread::spawn(|| {
		for i in 0..5 {
			println!("spawned thread print {}", i);
			thread::sleep(Duration::from_millis(1));
		}
	});
	
	let (tx, rx) = mpsc::channel();
	
	thread::spawn(move || {
		let val = String::from("hi");
		tx.send(val).unwrap();
	});
	
	let received = rx.recv().unwrap();
	println!("Got: {}", received);
	
	apply_to_num(
		1,
		|_| {
			return 5;
		},
		|_| {
			return 7;
		},
	);
	
	macro_rules! test_print {
		($a:ty,$b:tt,$c:expr,$d:expr) => {
			let c:$a=$c;println!("{:?}", $c);
			println!("{}", $d $b c);
		};
	}
	
	test_print!(i32,-,1+1,7);
}

fn apply_to_num<F, G>(num: i32, f: F, f2: G)
where
	F: Fn(i32) -> i32,
	G: Fn(i32) -> i32,
{
	println!("{}", num + f(1) + f2(2));
	// let mut input = String::new();
	//
	// io::stdin().read_line(&mut input).expect("");
}
