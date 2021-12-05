use std::path::*;
use std::fs::*;
use std::io::Read;

// switch vid and pid while in RCM mode
const SWITCH_VID:u16 = 0x090c; //0x0955;
const SWITCH_PID:u16 = 0x1000; //0x7321;

pub trait SwitchRCMDevice{
    fn new() -> Self;
    fn send_payload();
}

pub struct UsbDevice{
    vid: u16,
    pid: u16,
}


impl UsbDevice{
    fn open(){

    }
    //linux docs say scan every file with /dev/bus/usb
    // to look for our device,not sure if there is a more efficient way.
    //docs also say the usb section is incomplete and outdated
    pub fn find_device(&self) -> Result<PathBuf,UsbError>{
        let path = Path::new("/dev/bus/usb/");
        if !path.exists(){
            return Err(UsbError::PathDoesNotExist)
        }
        if !path.is_dir(){
            return Err(UsbError::PathNotDirectory)
        }
        
        let usb_directory:ReadDir = match path.read_dir(){
            Ok(dir) => dir,
            Err(_) => return Err(UsbError::DirReadError),
        };
        let mut device_paths:Vec<PathBuf> = Vec::new();
        //iterate trough the usb directory and get a DirEntry for each subdirectory
        //get path of each subdirectory. call read_dir on each subdirectory path to
        //get iterators of the device files witin. get path of each device file
        for bus in usb_directory{
            let bus_dirs = match bus{
                Ok(directory) => directory,
                Err(_) => return Err(UsbError::DirReadError),
            };
            let path = bus_dirs.path();
            let bus_dir_iterator = match path.read_dir(){
                Ok(directory) => directory,
                Err(_) => return Err(UsbError::DirReadError),
            };
            //not sure how to avoid this nested for loop
            for device in bus_dir_iterator{
                let device_file = match device{
                    Ok(device) => device,
                    Err(_)     => return Err(UsbError::DirReadError),
                };
                let device_file_path = device_file.path();
                device_paths.push(device_file_path);
            }
        }
        for path in device_paths{
            let found = match self.check_device(path.as_path()){
                Ok(found)   => found,
                Err(why)    => {println!("could not access:{:?} {:?}",path,why);continue},
            };
            if found == true{
                return Ok(path);
            }
        }
        Err(UsbError::CouldNotFindDevice)


    }
    //check if device file contains the VID and PID of our usb device
    //if yes return true
    fn check_device(&self,path: &Path) -> Result<bool,UsbError>{
        let vid = self.vid;
        let pid = self.pid;
        let mut device_file = match File::open(path){
            Ok(file)    => file,
            //make this more verbose later
            Err(_)      => return Err(UsbError::OpenDeviceFileError)
        };
        let mut contents = vec![0;189];
        let _bytes_read = device_file.read(&mut contents);

        let read_vid:u16;
        let read_pid:u16;

        read_vid = match le_bytes_to_u16(&contents[8..10]){
            Ok(vid) => vid,
            Err(why)=> return Err(why),
        };

        read_pid = match le_bytes_to_u16(&contents[10..12]){
            Ok(pid) =>pid,
            Err(why)=> return Err(why),
        };
        if self.vid == read_vid && self.pid ==read_pid{
            return Ok(true);
        }
        Ok(false)
    }
}

impl SwitchRCMDevice for UsbDevice{
    fn new() -> UsbDevice{
        UsbDevice{
            vid : SWITCH_VID,
            pid : SWITCH_PID,
        }
    }

    fn send_payload(){

    }


}
#[derive(Debug)]
pub enum UsbError{
    PathDoesNotExist,
    PathNotDirectory,
    DirReadError,
    OpenDeviceFileError,
    CouldNotFindDevice,
    BufferSizeMissmatch,
}


pub fn le_bytes_to_u16(buffer: &[u8]) -> Result<u16,UsbError>{ //convert little endian bytes to
    if buffer.len() != 2{                                        // u32 for easier comparisons
         return Err(UsbError::BufferSizeMissmatch);
    }
    let output = ((buffer[0] as u16) << 0)  +
                 ((buffer[1] as u16) << 8);
    Ok(output)
}