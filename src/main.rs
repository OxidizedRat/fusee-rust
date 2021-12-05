use crate::usb::*;
mod usb;
const RED:&str    = "\x1b[31m";
const GREEN:&str  = "\x1b[32m";

fn main() {
    let switch:UsbDevice = SwitchRCMDevice::new();
    let device_path =  match switch.find_device(){
        Ok(device) => device,
        Err(_)   => {println!("{} could not find rcm device",RED);return},
    };
    
    println!("{} found switch rcm device at:{:?}",GREEN, device_path);

}