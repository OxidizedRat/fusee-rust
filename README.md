# switch-rcm-rust 
WIP  fusee gelee payload injector for nintendo switch devices that are vulnerable.
For linux, written in Rust hopefully with zero dependencies.

Currentl can:
* Detect connected switch rcm mode devices

Actuall payload launching steps completed
    ✅ read device id 
    ❌ building exploit
    ❌ pad payload properly
    ❌ change DMA buffer if necessary
    ❌ send GET_STATUS request