use crate::usb::SwitchRCM;
mod usb;
use errno::errno;
const RED:&str    = "\x1b[31m";
const GREEN:&str  = "\x1b[32m";
const WHITE:&str  = "\x1b[37m";

fn main() {
    
    let mut switch = SwitchRCM::new();
    let _device_path =  match switch.find_device(){
        Ok(device) => device,
        Err(_)   => {println!("{} could not find rcm device{}",RED,WHITE);return},
    };
    println!("{}Found switch rcm device at:{:?}",GREEN, switch.get_usbfs_path());
    let _test = match switch.claim_interface(){
        Ok(ret)     => ret,
        Err(_)      =>{println!("{}Failed to claim interface, Error:{}{}",RED,errno(),WHITE);return},
    };
    println!("{}Interface claimed!{}",GREEN,WHITE);
    let dev_id = match switch.read_device_id(){
        Ok(id)  => id,
        Err(_)  => {println!("error:{}",errno());return},
    };
    println!("Device ID: ");
    for character in dev_id{
        print!("{:02x}",character);
    }
    println!("");

}