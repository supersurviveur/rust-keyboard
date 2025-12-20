An attempt for a rust, qmk-like, easy to configure, keyboard firmware.

Building:
- go to exemples/rust_keyboard (or you favorite one)
- install the depandancies/configure rust:
  + If on Arch / derivative: `make build_env_setup`
  + Else:
    Install the equivalent of avr-gcc and avr-libc and use these commands to setup rust
      ```
      rustup toolchain install nightly
      rustup override set nightly
      rustup component add rust-src --toolchain nightly
      ```
- run `make build` and get the output in build/rust_keyboard.elf or run `cargo build --release` and get the output in ./target/atmega32u4-none/release/rust_keyboard.elf
