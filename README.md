# switch-rcm-rust 
WIP  Reimplimentation of [fusee-nano](https://github.com/DavidBuchanan314/fusee-nano) in Rust, for linux.

Currentl can:
* Detect connected switch rcm mode devices

Actuall payload launching steps completed
* ✅ read device id 
* ❌ building exploit
* ❌ pad payload properly
* ❌ change DMA buffer if necessary
* ❌ send GET_STATUS request