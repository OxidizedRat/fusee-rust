use std::path::*;
use std::fs::*;
use std::io::Read;
use std::os::raw::*;
use std::ffi::CString;

// switch vid and pid while in RCM mode
const SWITCH_VID:&str = "0955";     //"090c"; test usb device vid and pid 
const SWITCH_PID:&str = "7321";     //"1000";


pub struct SwitchRCM{
    vid: String,
    pid: String,
    sysfs_path: PathBuf,
    usbfs_path: PathBuf,
    interface_number: i32, //29th byte in usbfs file
    file_descriptor: i32,
}


impl SwitchRCM{
    pub fn new() -> SwitchRCM{
        SwitchRCM{
            vid : SWITCH_VID.to_string(),
            pid : SWITCH_PID.to_string(),
            sysfs_path: PathBuf::new(),
            usbfs_path: PathBuf::new(),
            interface_number: 0,
            file_descriptor: 0,
        }
    }
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
                    //set usbfs
                    match self.set_usbfs_from_sysfs(){
                        Ok(_)       => (),
                        Err(why)    => return Err(why),
                    };
                    //set binterface number
                    match self.get_binterface_number(){
                        Ok(_)       => (),
                        Err(why)    => return Err(why),
                    };
                    //set file discriptor
                    match self.set_file_descriptor(){
                        Ok(_)   => (),
                        Err(why)=> return Err(why),
                    }
                    return Ok(path);
                }
                
            }
            
        } 
        return Err(UsbError::CouldNotFindDevice);
    }
    //get file discriptor to use with ioctl calls
    fn set_file_descriptor(&mut self) -> Result<c_int,UsbError>{
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
    fn read(&self,request: &BulkTransfer) -> Result<*const c_void,UsbError>{
        let fd = self.file_descriptor;
        let return_value:i32;
        unsafe{
            let request = std::mem::transmute::<&BulkTransfer,*const c_void>(request);
            return_value = ioctl(fd,USBDEVFS_BULK,request);
            if return_value>-1{
                return Ok(request);
            }
        }
        Err(UsbError::ReadError)
    }
    fn write(&mut self,request: &BulkTransfer) -> Result<c_int,UsbError>{
        let fd = self.file_descriptor;
        let return_value:i32;
        unsafe{
            let request = std::mem::transmute::<&BulkTransfer,*const c_void>(request);
            return_value = ioctl(fd,USBDEVFS_BULK,request);
            //println!("ret :{} errno:{}",return_value,errno());
            if return_value>-1{
                return Ok(return_value);
            }
        }
        Err(UsbError::WriteError)
    }
    pub fn read_device_id(&self)-> Result<&[u8;16],UsbError>{
    
        let device_id = CString::new("0000000000000000").expect("failed");
        let device_id = device_id.into_raw();
        unsafe{
        let device_id:*const c_void = std::mem::transmute(device_id);
            let request = &BulkTransfer{
                    endpoint : (USB_DIR_IN | 1) as u32,
                    length   : 16 as u32,
                    timeout  : 1000 as u32,
                    data     : device_id,
            };

            let returned_request = match self.read(request){
                Ok(id)      => id,
                Err(why)    => return Err(why),
            };

            let new_request:&BulkTransfer = std::mem::transmute(returned_request);
            let device_id = new_request.data;
            let device_id:&[u8;16] = std::mem::transmute(device_id);
            //let output_string = CString::from_vec_unchecked(device_id.to_vec());
            return Ok(device_id);
        }
        //Err(UsbError::ReadError)
    }
    pub fn generate_payload(&self,user_payload:&Path)-> Result<Vec<u8>,UsbError>{
        const PAYLOAD_LENGTH:u32 = 0x30298;
        const _PAYLOAD_START_ADDRESS:u32 = 0x40010E40;
        const RCM_PAYLOAD_ADDRESS:u32 =	0x40010000;
        const _STACK_SPRAY_START:u32   = 0x40014E40;
        const _STACK_SPRAY_END:u32     = 0x40017000;

        let mut payload:Vec<u8> = Vec::new();
        
        for byte in PAYLOAD_LENGTH.to_le_bytes(){
            payload.push(byte);
        }
        //pad payload till it is 680 bytes
        {
            let mut padding:Vec<u8> = vec![0;676];
            payload.append(&mut padding);
        }

        // spray stack
        {
            let count = (0x4001F000-RCM_PAYLOAD_ADDRESS)/4;
            let address:u32 = 0x4001F000;
            let payload_address_le = address.to_le_bytes();
            for _times in 0..count{
                payload.push(payload_address_le[0]);
                payload.push(payload_address_le[1]);
                payload.push(payload_address_le[2]);
                payload.push(payload_address_le[3]);
            }
        }
        //get relocator
        let relocator_path = Path::new("./intermezzo.bin");
        let mut relocator = match std::fs::read(relocator_path){
            Ok(bytes)   => bytes,
            Err(_)      => return Err(UsbError::RelocatorNotFound),
        };
        //let relocator_size = relocator.len();
        //add relocator to payload
        payload.append(&mut relocator);
        //pad 
        {
            let size_to_pad:usize = 0x40020000 - (0x4001F000+92);
            let mut padding:Vec<u8> = vec![0;size_to_pad];
            payload.append(&mut padding);
        }
        //get user payload
        let mut u_payload = match std::fs::read(user_payload){
            Ok(bytes)       => bytes,
            Err(_)          => return Err(UsbError::UserPayloadNotFound),
        };
        payload.append(&mut u_payload);
        //get lenght of payload and see if it is divisible by 0x1000
        //pad till is it
        {
            let payload_length = payload.len();
            let pad_size   = 0x1000 - (payload_length % 0x1000);
            for _number in 0..pad_size{
                payload.push(0);
            }
        }
        //check payload length 
        println!("max size = {}, payload size :{}",PAYLOAD_LENGTH,payload.len());
        if payload.len() > PAYLOAD_LENGTH as usize{
            return Err(UsbError::PayloadTooLarge);
        }

        Ok(payload)

    }
    pub fn send_payload(&mut self,payload: Vec<u8>) -> Result<c_int,UsbError>{
        let mut write_count = 0;
        let mut bytes_written = 0;
        let chunks_num = payload.len()/0x1000;
        for chunk in 0..chunks_num{
            let index = 0x1000*chunk;
            let payload_ptr: *const c_void = unsafe {std::mem::transmute(payload[index..].as_ptr())};
            let request = &BulkTransfer{
                endpoint : (USB_DIR_OUT | 1) as u32,
                length   : 0x1000,
                timeout  : 1000 as u32,
                data     : payload_ptr,
            };

            let ret_val = match self.write(request){
                Ok(val)     => val,
                Err(why)    => return Err(why),
            };
            bytes_written += ret_val;
            write_count +=1;
        }
        if write_count%2 == 0{
            let data = vec![0;0x1000];
            let ptr: *const c_void = unsafe {std::mem::transmute(data.as_ptr())};
            let request = &BulkTransfer{
                endpoint : (USB_DIR_OUT | 1) as u32,
                length   : 0x1000,
                timeout  : 1000 as u32,
                data     : ptr,
            };
            let ret_val = match self.write(request){
                Ok(val)     => val,
                Err(why)    => return Err(why),
            };
            bytes_written +=ret_val;
        }
        println!("Write count:{}",write_count);
        Ok(bytes_written)
    }

    pub fn trigger_pull(&self) -> Result<c_int,UsbError>{
        let fd = self.file_descriptor;
        //let data:&[u8;0x7000] = &[0;0x7000];
        //let data_ptr: *const c_void = unsafe{std::mem::transmute(data)};
        let request = &ControlRequest{
            request_type: 130,       //USB_DIR_IN | USB_RECIP_ENDPOINT
            request     :   0,       //USB_REQ_GET_STATUS
            value       :   0,
            index       :   0,
            length      :   0x0070, //payload length
        };
        let request_ptr:*const c_void = unsafe{std::mem::transmute(request)};
    
        let context:&u32 = &0;
        let context_ptr:*const c_void = unsafe{std::mem::transmute(context)};
        let iso = iso_frame{
            length:37108932,
            actual_length:32765,
            status:1501646240,
        };
        let urb = &URB{
            request_type : 2,             //USBDEVFS_TYPE_CONTROL
            endpoint     : 0,        
            status       : 0,
            flags        : 0,
            buffer       : request_ptr,       //required
            buffer_length: 0x7000+8,               //required
            actual_length: 0,
            start_frame  : 0,
            union        :[0;4],
            error_count  : 0,
            signal_error : 0,
            user_context : context_ptr,
            iso_packet   : &iso,
        };
        
        let urb:*const c_void = unsafe{std::mem::transmute(urb)};
        //send request
        unsafe{
            let return_value = ioctl(fd,USBDEVFS_SUBMITURB,urb);
            if return_value <0{
                return Err(UsbError::SubmitError);
            }
            return Ok(return_value);
        }
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
    PayloadTooLarge,
    RelocatorNotFound,
    UserPayloadNotFound,
    WriteError,
    SubmitError,
}

extern "C"{
    //int open(const char *pathname, int flags);
    pub fn open(path:*const c_char,flags : c_int) -> c_int;
    //int ioctl (int __fd, unsigned long int __request, ...) __THROW;
    pub fn ioctl(file_descriptor: c_int, request:u32,data : *const c_void) ->c_int;
    //int * __errno_location(void);
    //pub fn __errno_location() -> *const c_int;
}

//const O_RDONLY:c_int =00;
const O_RDWR: c_int = 02;
//IOCTL request type definitions,not sure how portable these are
//probably will only work on x86_64 systems
pub const USBDEVFS_CLAIMINTERFACE:u32 = 2147767567;
pub const _USBDEVFS_CONNECTINFO:u32 = 1074287889;
pub const USBDEVFS_SUBMITURB:u32 = 2151175434;
pub const USBDEVFS_BULK:u32 = 3222820098; //3222820098
pub const _USBDEVFS_CONTROL:u32 = 3222820096;
pub const _USBDEVFS_URB_TYPE_CONTROL:u32 = 2;
pub const _USBDEVFS_DISCARDURB:u32 = 21771;
pub const _USBDEVFS_REAPURB:u32 =1074287884;
pub const USB_DIR_IN:c_int = 128;
pub const USB_DIR_OUT:c_int = 0;

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

#[repr(C)]
struct ControlRequest{
    request_type:   u8,
    request     :   u8,
    value       :   u16,
    index       :   u16,
    length      :   u16,
}

#[repr(C)]
struct _ControlTransfer{
    request_type:   u8,
    request     :   u8,
    value       :   u16,
    index       :   u16,
    length      :   u16,
    data        :   *const c_void,
}

//56 bytes total
#[repr(C)]
struct URB{
    request_type : c_uchar,             //required
    endpoint     : c_uchar,             //required
    status       : c_int,
    flags        : c_uint,
    buffer       : *const std::ffi::c_void,       //required
    buffer_length: c_int,               //required
    actual_length: c_int,
    start_frame  : c_int,
    union        :[u8;4],
    error_count  : c_int,
    signal_error : c_uint,
    user_context : *const std::ffi::c_void,
    iso_packet   : *const iso_frame,
}

#[repr(C)]
struct iso_frame{
    length : c_uint,
    actual_length: c_uint,
    status : c_uint,
}
/*

struct usbdevfs_urb {
        unsigned char type;
        unsigned char endpoint;
        int status;
        unsigned int flags;
        void *buffer;
        int buffer_length;
        int actual_length;
        int start_frame;
        union {
               	int number_of_packets;  /* Only used for isoc urbs */
                unsigned int stream_id; /* Only used with bulk streams */
        };
	int error_count;
        unsigned int signr;     /* signal to be sent on completion,
                                  or 0 if none should be sent. */
        void *usercontext;
        struct usbdevfs_iso_packet_desc iso_frame_desc[0];
};


*/
/*
pub fn errno()->Errno{
    let err:i32 = 0;
    unsafe{
        let errno = __errno_location();

        let err_pointer:&i32 = std::mem::transmute(errno);
        let err = *err_pointer;
    }

}
*/