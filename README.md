# touchscreen-transform

Emulate mouse buttom as a touchscreen drag event

### Usage

Install this virtual_touchscreen module first, http://github.com/vi/virtual_touchscreen/

Change const SLOT_BUTTOM_CODE to your mouse function keycode

```
RUST_LOG=trace cargo run -- --abs-x-max 2560 --abs-y-max 1440 --input-device /dev/input/by-id/usb-Razer_Razer_Viper_8KHz-event-mouse
```
