use std::env;
use std::path::PathBuf;

use bindgen::RustTarget;

pub fn main() {
    // Build LUFA
    let mut build = cc::Build::new();

    build
        .compiler("avr-gcc")
        .target("avr-none")
        .define("F_CPU", "16000000UL")
        .define("F_USB", "F_CPU")
        .define("ARCH", "ARCH_AVR8")
        .define("BOARD", "BOARD_USBKEY")
        .define("USE_LUFA_CONFIG_HEADER", None)
        .flag("-I.")
        .flag("-Ilufa")
        .flag("-mmcu=atmega32u4")
        .flag("-Os");

    build
        .flag("-fshort-enums")
        .flag("-fno-inline-small-functions")
        .flag("-fno-strict-aliasing")
        .flag("-funsigned-char")
        .flag("-funsigned-bitfields")
        .flag("-ffunction-sections")
        .flag("-mrelax")
        .flag("-fno-jump-tables");

    build.file("lufa/LUFA/Drivers/USB/Core/AVR8/Device_AVR8.c");
    build.file("lufa/LUFA/Drivers/USB/Core/AVR8/Endpoint_AVR8.c");
    build.file("lufa/LUFA/Drivers/USB/Core/AVR8/EndpointStream_AVR8.c");
    build.file("lufa/LUFA/Drivers/USB/Core/AVR8/Host_AVR8.c");
    // build.file("lufa/LUFA/Drivers/USB/Core/AVR8/PipeStream_AVR8.c");
    // build.file("lufa/LUFA/Drivers/USB/Core/AVR8/Pipe_AVR8.c");
    build.file("lufa/LUFA/Drivers/USB/Core/AVR8/USBController_AVR8.c");
    build.file("lufa/LUFA/Drivers/USB/Core/AVR8/USBInterrupt_AVR8.c");
    build.file("lufa/LUFA/Drivers/USB/Core/ConfigDescriptors.c");
    build.file("lufa/LUFA/Drivers/USB/Core/DeviceStandardReq.c");
    build.file("lufa/LUFA/Drivers/USB/Core/Events.c");
    build.file("lufa/LUFA/Drivers/USB/Core/HostStandardReq.c");
    build.file("lufa/LUFA/Drivers/USB/Core/USBTask.c");

    //tmp
    println!("cargo::rerun-if-changed=lufa/Demos/Device/LowLevel/Keyboard/Keyboard.c");
    println!("cargo::rerun-if-changed=lufa/Demos/Device/LowLevel/Keyboard/Descriptors.c");
    println!("cargo::rerun-if-changed=lufa/Demos/Device/LowLevel/Keyboard/Keyboard.h");
    println!("cargo::rerun-if-changed=lufa/Demos/Device/LowLevel/Keyboard/Descriptors.h");
    build.file("lufa/Demos/Device/LowLevel/Keyboard/Keyboard.c");
    build.file("lufa/Demos/Device/LowLevel/Keyboard/Descriptors.c");

    build.compile("lufa");

    let bindings = bindgen::Builder::default()
        .header("lufa/LUFA/Drivers/USB/USB.h")
        .header("LUFAConfig.h")
        .clang_arg("-mmcu=atmega32u4")
        .clang_arg("-DF_CPU=16000000UL")
        .clang_arg("-DF_USB=F_CPU")
        .blocklist_type("size_t")
        .derive_default(true)
        .formatter(bindgen::Formatter::Rustfmt)
        .enable_function_attribute_detection()
        .disable_name_namespacing()
        .default_enum_style(bindgen::EnumVariation::Rust {
            non_exhaustive: true,
        })
        .rust_target(RustTarget::nightly())
        .blocklist_function("CALLBACK_USB_GetDescriptor")
        .use_core()
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");

    println!("cargo::rustc-link-search=/usr/avr/lib/avr5/");
    println!("cargo::rustc-link-lib=static=lufa");
    println!("cargo::rustc-link-lib=static=atmega32u4");

    // tmp

    // let mut build = cc::Build::new();
    // build
    //     .compiler("avr-gcc")
    //     .archiver("avr-ar")
    //     .target("avr-none")
    //     .define("F_CPU", "16000000UL")
    //     .define("F_USB", "F_CPU")
    //     .define("ARCH", "ARCH_AVR8")
    //     .define("BOARD", "BOARD_USBKEY")
    //     .define("USE_LUFA_CONFIG_HEADER", None)
    //     .flag("-I.")
    //     .flag("-Ilufa")
    //     .flag("-mmcu=atmega32u4")
    //     .flag("-Os");
    // build
    //     .flag("-fshort-enums")
    //     .flag("-fno-inline-small-functions")
    //     .flag("-fno-strict-aliasing")
    //     .flag("-funsigned-char")
    //     .flag("-funsigned-bitfields")
    //     .flag("-ffunction-sections")
    //     .flag("-mrelax")
    //     .flag("-fno-jump-tables");

    // build.file("lufa/Demos/Device/LowLevel/Keyboard/Keyboard.c");
    // build.file("lufa/Demos/Device/LowLevel/Keyboard/Descriptors.c");
    // build.compile("lufa_demo");

    // println!("cargo::rustc-link-lib=static=lufa_demo");
}
