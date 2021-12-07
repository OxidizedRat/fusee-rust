use std::path::*;
use std::fs::*;
use std::io::Read;
use std::os::raw::*;
use std::ffi::CString;

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
    _sysfs_path: PathBuf,
    _usbfs_path: PathBuf,
    _interface_number: i32, //29th byte in usbfs file
}


impl UsbDevice{
    //linux docs say scan every file with /dev/bus/usb
    // to look for our device,not sure if there is a more efficient way.
    //docs also say the usb section is incomplete and outdated
    pub fn find_device(&mut self) -> Result<PathBuf,UsbError>{
        let devices_path = Path::new("/sys/bus/usb/devices/");
        
        for device in devices_path.read_dir().expect("Could not read dir"){
            //ignore errors and iterate over dir entries
            if let Ok(device) = device{
                //get path
                let mut path = device.path();
                //append idvendor to the end(contains VID)
                path.push("idVendor");
                //read vid file and set it as self.vid
                let mut vid_file = match File::open(path.as_path()){
                    Ok(file)    => file,
                    Err(_)    =>continue,
                };
                let mut vid = String::new();
                match vid_file.read_to_string(&mut vid){
                    Ok(_)   => (),
                    Err(_)  => {println!("could not read file");continue},
                };
                //remove new line from string
                vid.pop();
                //if vendor id is wrong just go to next loop
                if vid != SWITCH_VID.to_string(){
                    continue;
                }
                
                path.pop();
                //same as above but for PID
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
                    
                    self._sysfs_path = path.clone();
                    match self.set_usbfs_from_sysfs(){
                        Ok(_)       => (),
                        Err(why)    => return Err(why),
                    };
                    match self.get_binterface_number(){
                        Ok(_)       => (),
                        Err(why)    => return Err(why),
                    };

                    return Ok(path);
                }
                
            }
            
        } 
        return Err(UsbError::CouldNotFindDevice);
    }
    //get file discriptor to use with ioctl calls
    pub fn get_file_descriptor(&self) -> Result<c_int,UsbError>{
        //convert our pathbuf into a c compatible char pointer    
        //not sure if this formatting is idiomatic
        let path = match self._usbfs_path
                             .clone()
                             .into_os_string()
                             .into_string(){
                                 Ok(string)  =>string,
                                 Err(_)      => return Err(UsbError::NotUnicodeString),
                    };
     
        let path = match CString::new(path){
            Ok(cstring)     => cstring,
            Err(_)           => return Err(UsbError::CouldNotCreateCString),
        };
        let path: *const i8 = path.as_c_str().as_ptr();
        
        unsafe{
            let file_desc = open(path,O_RDONLY);
            Ok(file_desc)
        }
        
    }
    pub fn get_usbfs_path(&self) -> PathBuf{
        self._usbfs_path.clone()
        
    }
    fn set_usbfs_from_sysfs(&mut self) -> Result<PathBuf,UsbError> {
        let mut sysfs = self._sysfs_path.clone();
        //one way of getting the devpath
        sysfs.push("uevent");
        //read contents of uevent
        let uevent = match std::fs::read_to_string(sysfs.as_path()){
            Ok(file)    => file,
            Err(_)      => return Err(UsbError::CouldNotOpenUevent),
        };
        //get third line which contains devpath
        let mut lines = uevent.lines();
        let devpath = match lines.nth(2){
            Some(line) => line,
            None       => return Err(UsbError::CouldNotGetDevPath)
        };
        //format devpath properly
        let devpath = devpath.trim_start_matches("DEVNAME=");
        let devpath = "/dev/".to_string() + devpath;

        let path = PathBuf::from(&devpath);
        self._usbfs_path = path.clone();
        Ok(path)
    }
    fn get_binterface_number(&mut self) -> Result<i32,UsbError>{
        let dev_path = self._usbfs_path.clone();
        let file_buffer = match read(dev_path.as_path()){
            Ok(buffer)      => buffer,
            Err(_)          => return Err(UsbError::CouldNotReadDevPath),
        };
        let interface_num = file_buffer[29].try_into().unwrap();
        self._interface_number = interface_num;
        Ok(interface_num)
    }
    pub fn claim_interface(&self)-> Result<i32,UsbError>{
        let file_descriptor = match self.get_file_descriptor(){
            Ok(fd)      =>fd,
            Err(why)    => return Err(why),
        };

        unsafe{
            let pointer = self._interface_number as *const c_int;
            //let pointer = std::mem::transmute::<*const i32, *const c_void>(pointer);
            let return_value = ioctl(file_descriptor,USBDEVFS_CLAIMINTERFACE,pointer);
            println!("{}",return_value);
            if return_value > -1 {
                return Ok(return_value);
            }
            Err(UsbError::ClaimingInterfaceFailed)
        }
    }
}

impl SwitchRCMDevice for UsbDevice{
    fn new() -> UsbDevice{
        UsbDevice{
            _vid : SWITCH_VID.to_string(),
            _pid : SWITCH_PID.to_string(),
            _sysfs_path: PathBuf::new(),
            _usbfs_path: PathBuf::new(),
            _interface_number: 0,
        }
    }

    fn send_payload(){

    }


}
#[derive(Debug)]
pub enum UsbError{
    CouldNotFindDevice,
    CouldNotOpenUevent,
    CouldNotGetDevPath,
    NotUnicodeString,
    CouldNotCreateCString,
    CouldNotReadDevPath,
    ClaimingInterfaceFailed
}

extern "C"{
    //int open(const char *pathname, int flags);
    pub fn open(path:*const c_char,flags : c_int) -> c_int;
    //int ioctl (int __fd, unsigned long int __request, ...) __THROW;
    pub fn ioctl(file_descriptor: c_int, request:u32,data : *const c_int) ->c_int;
}

const O_RDONLY:c_int =00;
//IOCTL request type definitions,not sure how portable these are
//probably will only work on x86_64 systems
pub const USBDEVFS_CLAIMINTERFACE:u32 = 2147767567;
