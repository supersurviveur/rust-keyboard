# lufa-rs

This crate provides Rust raw bindings for the LUFA (Lightweight USB Framework for AVRs) library. It also contains some utility functions and macros to easily create HID (Human Interface Device) applications.


## Installation

Add to your `Cargo.toml` file:

```toml
[dependencies]
lufa-rs = "0.1"
```

## Usage

LUFA needs some environment variables to be set for the build process. You can set them in your `.cargo/config` file:

```toml
[env]
F_CPU="16000000"
ARCH="ARCH_AVR8"
BOARD="BOARD_USBKEY"
MMCU="atmega32u4"
LUFA_CONFIG_PATH = { value = ".", relative = true }
```

A `LUFAConfig.h` file needs to be present in the `LUFA_CONFIG_PATH` directory. See [LUFA devices examples](https://github.com/abcminiuser/lufa/tree/master/Demos) or [rust-keyboard](https://github.com/supersurviveur/rust-keyboard/tree/main/qmk/LUFAConfig.h) to create one.

Then you can use the library in your project:

```rust
use core::arch::asm;
use lufa_rs::{USB_Init, USB_USBTask};

fn main() {
    // USB initialization before enabling interrupts
    unsafe {
        USB_Init();
    }
    // Other initialization code, then enabling interrupts
    unsafe {
        asm!("sei");
    }
    // Your main loop
    loop {
        // Your code
        unsafe {
            USB_USBTask();
        }
    }
}
```

Some extern functions must be implemented for the library to work correctly. See any [LUFA example](https://github.com/abcminiuser/lufa/tree/master/Demos) for details.

## Examples

LUFA demos can be found in the [LUFA repository](https://github.com/abcminiuser/lufa/tree/master/Demos).

See the [rust-keyboard](https://github.com/supersurviveur/rust-keyboard/) repository for usage cases.
The library is used [here](https://github.com/supersurviveur/rust-keyboard/tree/main/qmk/src/usb)

## Contribution

Contributions are welcome! Open an issue or a pull request.

## License

This project is licensed under the MIT License.