pub fn main() {
    // Build LUFA
    let mut build = cc::Build::new();

    build
        .compiler("avr-gcc")
        .target("avr-none")
        .define("F_CPU", Some("16000000UL"))
        .define("F_USB", "F_CPU")
        .flag("-mmcu=atmega32u4")
        .flag("-Os");
    // .define("ARCH", Some("ARCH_AVR8"))
    // .define("USE_FLASH_DESCRIPTORS", None)
    // .define("USE_STATIC_OPTIONS", Some("USB_DEVICE_OPT_FULLSPEED"))
    // build.flag("-fshort-enums")
    // .flag("-funsigned-char")
    // .flag("-funsigned-bitfields")
    // .flag("-ffunction-sections")
    // .flag("-fpack-struct")
    // .flag("-fno-jump-tables");

    build.file("lufa/LUFA/Drivers/USB/Core/AVR8/Device_AVR8.c");
    build.file("lufa/LUFA/Drivers/USB/Core/AVR8/Endpoint_AVR8.c");
    build.file("lufa/LUFA/Drivers/USB/Core/AVR8/EndpointStream_AVR8.c");
    build.file("lufa/LUFA/Drivers/USB/Core/AVR8/Host_AVR8.c");
    build.file("lufa/LUFA/Drivers/USB/Core/AVR8/PipeStream_AVR8.c");
    build.file("lufa/LUFA/Drivers/USB/Core/AVR8/Pipe_AVR8.c");
    build.file("lufa/LUFA/Drivers/USB/Core/AVR8/USBController_AVR8.c");
    build.file("lufa/LUFA/Drivers/USB/Core/AVR8/USBInterrupt_AVR8.c");
    build.file("lufa/LUFA/Drivers/USB/Core/ConfigDescriptors.c");
    build.file("lufa/LUFA/Drivers/USB/Core/DeviceStandardReq.c");
    build.file("lufa/LUFA/Drivers/USB/Core/Events.c");
    build.file("lufa/LUFA/Drivers/USB/Core/HostStandardReq.c");
    build.file("lufa/LUFA/Drivers/USB/Core/USBTask.c");

    build.compile("lufa");

    println!("cargo::rustc-link-search=/usr/avr/lib/avr5/");
    println!("cargo::rustc-link-lib=static=lufa");
    println!("cargo::rustc-link-lib=static=atmega32u4")
}
