use crate::usb::*;
mod usb;
//const RED:&str    = "\x1b[31m";
const GREEN:&str  = "\x1b[32m";

fn main() {
    let switch:UsbDevice = SwitchRCMDevice::new();

    let rcm_device = match switch.find_device(){
        Ok(path)    => path,
        Err(why)    => panic!("{:?}",why),
    };
    println!("{} found switch rcm device at:{:?}",GREEN, rcm_device);

}