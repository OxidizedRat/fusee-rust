use std::path::Path;
use crate::usb::{SwitchRCM,UsbError};
mod usb;
use errno::errno;
const RED:&str    = "\x1b[31m";
const GREEN:&str  = "\x1b[32m";
const WHITE:&str  = "\x1b[37m";

fn main() -> Result<(),UsbError> {
    let mut args = std::env::args();
    let payload_path =  match args.nth(1){
        Some(path)  =>path,
        None        =>{println!("Usage:fusee-rust [path to payload]");return Err(UsbError::UserPayloadNotFound)},
    };
    let payload_path = Path::new(&payload_path);
    if !payload_path.is_file(){
        println!("Error accessing payload file, please check it exists");
        return Err(UsbError::UserPayloadNotFound)
    }

    let mut switch = SwitchRCM::new();
    //find_device() gets all the information about the device
    let _device_path =  match switch.find_device(){
        Ok(device) => device,
        Err(why)   => {println!("{} could not find rcm device{}",RED,WHITE);return Err(why)},
    };
    println!("{}Found switch rcm device at:{:?}",GREEN, switch.get_usbfs_path());
    //use ioctl to claim the device
    let _test = match switch.claim_interface(){
        Ok(ret)     => ret,
        Err(why)      =>{println!("{}Failed to claim interface, Error:{}{}",RED,errno(),WHITE);return Err(why)},
    };
    println!("{}Interface claimed!{}",GREEN,WHITE);
    //read the device id, first step for exploiting the bug
    let dev_id = match switch.read_device_id(){
        Ok(id)  => id,
        Err(why)  => {println!("error:{}",errno());return Err(why)},
    };
    print!("Device ID: ");
    for character in dev_id{
        print!("{:02x}",character);
    }
    println!("");
    println!("Generating Payload...");
    let payload = match switch.generate_payload(payload_path){
        Ok(payload)     => payload,
        Err(why)          => return Err(why),
    };
    println!("Payload Generated");
    println!("Sending payload...");
    let bytes_written = match switch.send_payload(payload){
        Ok(num)     => num,
        Err(why)    => {println!("Error: {}",errno());return Err(why)}
    };

    println!("{} bytes sent",bytes_written);
    Ok(())
}