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
    println!("{} found switch rcm device at:{:?}",GREEN, switch.get_usbfs_path());
    let _test = match switch.claim_interface(){
        Ok(ret)     => ret,
        Err(_)      =>{println!("{} failed to claim interface{}",RED,WHITE);return},
    };
    println!("{} interface claimed!{}",GREEN,WHITE);
    let dev_id = match switch.read_device_id(){
        Ok(id)  => id,
        Err(_)  => {let e  = errno();println!("error:{}",e);return},
    };

    println!("device id:{:?}",dev_id);
}