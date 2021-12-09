use crate::usb::{SwitchRCMDevice, UsbDevice};
mod usb;
use errno::errno;
const RED:&str    = "\x1b[31m";
const GREEN:&str  = "\x1b[32m";

fn main() {
    
    let mut switch:UsbDevice = SwitchRCMDevice::new();
    let _device_path =  match switch.find_device(){
        Ok(device) => device,
        Err(_)   => {println!("{} could not find rcm device",RED);return},
    };
    let _x =switch.set_file_descriptor();
    println!("{} found switch rcm device at:{:?}",GREEN, switch.get_usbfs_path());
    let _test = match switch.claim_interface(){
        Ok(ret)     => ret,
        Err(_)      =>{println!("{} failed to claim interface",RED);return},
    };
    println!("{} interface claimed!",GREEN);
    let dev_id = match switch.read_device_id(){
        Ok(id)  => id,
        Err(_)  => {let e  = errno();println!("error:{}",e);return},
    };

    println!("device id:{:?}",dev_id);
}