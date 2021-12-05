use std::path::*;
use std::fs::*;
use std::io::Read;

// switch vid and pid while in RCM mode
const SWITCH_VID:&str = "0955";     //"090c"; test usb device vid and pid 
const SWITCH_PID:&str = "7321";     //"1000";

pub trait SwitchRCMDevice{
    fn new() -> Self;
    fn send_payload();
}

pub struct UsbDevice{
    _vid: String,
    _pid: String,
}


impl UsbDevice{
    //linux docs say scan every file with /dev/bus/usb
    // to look for our device,not sure if there is a more efficient way.
    //docs also say the usb section is incomplete and outdated
    pub fn find_device(&self) -> Result<PathBuf,UsbError>{
        let devices_path = Path::new("/sys/bus/usb/devices/");
        
        for device in devices_path.read_dir().expect("Could not read dir"){
            if let Ok(device) = device{
                let mut path = device.path();
                path.push("idVendor");
                let mut vid_file = match File::open(path.as_path()){
                    Ok(file)    => file,
                    Err(_)    =>continue,
                };
                let mut vid = String::new();
                match vid_file.read_to_string(&mut vid){
                    Ok(_)   => (),
                    Err(_)  => {println!("could not read file");continue},
                };
                vid.pop();
                if vid != SWITCH_VID.to_string(){
                    continue;
                }
                
                path.pop();
                path.push("idProduct");
                let mut pid_file = match File::open(path.as_path()){
                    Ok(file)    => file,
                    Err(_)    => continue,
                };
                let mut pid = String::new();
                match pid_file.read_to_string(&mut pid){
                    Ok(_)   => (),
                    Err(_)  => {println!("could not read file");continue},
                };
                pid.pop();
                if pid == SWITCH_PID.to_string(){
                    path.pop();
                    return Ok(path);
                }
                
            }
            
        } 
        return Err(UsbError::CouldNotFindDevice);
    }
   
}

impl SwitchRCMDevice for UsbDevice{
    fn new() -> UsbDevice{
        UsbDevice{
            _vid : SWITCH_VID.to_string(),
            _pid : SWITCH_PID.to_string(),
        }
    }

    fn send_payload(){

    }


}
#[derive(Debug)]
pub enum UsbError{
    CouldNotFindDevice,
}
