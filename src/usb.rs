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
    fn read_device_id(&self)->Result<CString, UsbError>;
    fn send_payload(&self);
}

pub struct UsbDevice{
    vid: String,
    pid: String,
    sysfs_path: PathBuf,
    usbfs_path: PathBuf,
    interface_number: i32, //29th byte in usbfs file
    file_descriptor: i32,
}


impl UsbDevice{
    //linux docs say scan every file with /dev/bus/usb
    // to look for our device,not sure if there is a more efficient way.
    //docs also say the usb section is incomplete and outdated
    pub fn find_device(&mut self) -> Result<PathBuf,UsbError>{
        let devices_path = Path::new("/sys/bus/usb/devices/");
        
        for device in devices_path.read_dir().expect("Could not read dir"){
            //only process valid dir entries
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
                if vid != self.vid.to_string(){
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
                if pid == self.pid.to_string(){
                    path.pop();
                    
                    self.sysfs_path = path.clone();
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
    pub fn set_file_descriptor(&mut self) -> Result<c_int,UsbError>{
        //convert our pathbuf into a c compatible char pointer    
        //not sure if this formatting is idiomatic
        let path = match self.usbfs_path
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
            let file_desc = open(path,O_RDWR);
            self.file_descriptor = file_desc;
            Ok(file_desc)
        }
        
    }
    pub fn get_usbfs_path(&self) -> PathBuf{
        self.usbfs_path.clone()
        
    }
    fn set_usbfs_from_sysfs(&mut self) -> Result<PathBuf,UsbError> {
        let mut sysfs = self.sysfs_path.clone();
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
        self.usbfs_path = path.clone();
        Ok(path)
    }
    fn get_binterface_number(&mut self) -> Result<i32,UsbError>{
        let dev_path = self.usbfs_path.clone();
        let file_buffer = match read(dev_path.as_path()){
            Ok(buffer)      => buffer,
            Err(_)          => return Err(UsbError::CouldNotReadDevPath),
        };
        let interface_num = file_buffer[29].try_into().unwrap();
        self.interface_number = interface_num;
        Ok(interface_num)
    }
    pub fn claim_interface(&self)-> Result<i32,UsbError>{
        let file_descriptor = self.file_descriptor;

        unsafe{
            let pointer = self.interface_number;
            let pointer = std::mem::transmute::<&i32, *const c_void>(&pointer);
            let return_value = ioctl(file_descriptor,USBDEVFS_CLAIMINTERFACE,pointer);
            if return_value > -1 {
                return Ok(return_value);
            }
            Err(UsbError::ClaimingInterfaceFailed)
        }
    }
    fn read(&self,request: BulkTransfer) -> Result<*const c_void,UsbError>{
        let fd = self.file_descriptor;
        let return_value:i32;
        unsafe{
            let request = std::mem::transmute::<&BulkTransfer,*const c_void>(&request);
            return_value = ioctl(fd,USBDEVFS_BULK,request);
        }

        if return_value>-1{
            return Ok(request.data);
        }

        Err(UsbError::ReadError)
    }
    //just testing ioctl commands
    pub fn _get_connect_info(&self) -> Result<ConnectInfo,UsbError>{
        let fd = self.file_descriptor;
        let connect_info = &ConnectInfo{
                                dev_num:0, 
                                slow : 0,
        };
        unsafe{
            let info_pointer = std::mem::transmute::<&ConnectInfo, *const c_void>(connect_info);
            let ioctl_ret = ioctl(fd,_USBDEVFS_CONNECTINFO,info_pointer);
            let connect_info = std::mem::transmute::<*const c_void, &ConnectInfo>(info_pointer);
            if ioctl_ret >-1{
                return Ok(*connect_info);
            }
        }

        Err(UsbError::ClaimingInterfaceFailed)
    }
}

impl SwitchRCMDevice for UsbDevice{
    fn new() -> UsbDevice{
        UsbDevice{
            vid : SWITCH_VID.to_string(),
            pid : SWITCH_PID.to_string(),
            sysfs_path: PathBuf::new(),
            usbfs_path: PathBuf::new(),
            interface_number: 0,
            file_descriptor: 0,
        }
    }

    fn read_device_id(&self)-> Result<CString,UsbError>{
    
        let device_id:&[c_char;16] = &[0;16];
        unsafe{
        let device_id = std::mem::transmute::<&[c_char;16],*const c_void>(device_id);
        
            let request = BulkTransfer{
                    endpoint : USB_DIR_IN | 1,
                    length   : 16,
                    timeout  : 1000,
                    data     : device_id,
            };

            let device_id = match self.read(request){
                Ok(id)      => id,
                Err(why)    => return Err(why),
            };

            let device_id:*mut c_char = std::mem::transmute(device_id);
            let output_string = CString::from_raw(device_id);
            return Ok(output_string);
        }
        //Err(UsbError::ReadError)
    }

    fn send_payload(&self){
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
    ClaimingInterfaceFailed,
    ReadError,
}

extern "C"{
    //int open(const char *pathname, int flags);
    pub fn open(path:*const c_char,flags : c_int) -> c_int;
    //int ioctl (int __fd, unsigned long int __request, ...) __THROW;
    pub fn ioctl(file_descriptor: c_int, request:u32,data : *const c_void) ->c_int;
}

//const O_RDONLY:c_int =00;
const O_RDWR: c_int = 02;
//IOCTL request type definitions,not sure how portable these are
//probably will only work on x86_64 systems
pub const USBDEVFS_CLAIMINTERFACE:u32 = 2147767567;
pub const _USBDEVFS_CONNECTINFO:u32 = 1074287889;
pub const _USBDEVFS_SUBMITURB:u32 = 2151175434;
pub const USBDEVFS_BULK:u32 = 3222820098;
pub const USB_DIR_IN:u32 = 128;

#[repr(C)]
#[derive(Copy,Clone,Debug)]
pub struct ConnectInfo{
    dev_num:c_uint,
    slow: c_uchar,
}

#[repr(C)]
struct BulkTransfer{
    endpoint : c_uint,
    length   : c_uint,
    timeout  : c_uint,
    data     : *const c_void,
}