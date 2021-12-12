# switch-rcm-rust 
WIP  Reimplimentation of [fusee-launcher](https://github.com/Qyriad/fusee-launcher) in Rust, for linux.

Currentl can:
* Detect connected switch rcm mode devices

Actuall payload launching steps completed
* ✅ read device id 
* ✅ building exploit
* ✅ pad payload properly
* ✅ change DMA buffer if necessary
* ✅ send payload
* ❌ send GET_STATUS request