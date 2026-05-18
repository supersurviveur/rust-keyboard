TARGET_RS := atmega32u4-none

.PHONY: build

left: build
left: flash

right::
	IS_KEYBOARD_RIGHT=1 make

flash:
	avr-strip ./build/rust-keyboard.elf
	DEVICE=$$(./examples/common/autoflash.sh); \
	avrdude -p m32u4 -c avr109 -P $$DEVICE -U flash:w:./build/rust-keyboard.elf -U eeprom:w:./build/rust-keyboard.elf

both: left
both: right

build:
	cargo build --release -p keyboard-$(USER) -Zjson-target-spec
	mkdir -p build
	cp ./target/$(TARGET_RS)/release/keyboard-$(USER).elf build/rust-keyboard.elf

clean:
	cargo clean

build_env_setup:
	sudo pacman -S --needed avr-gcc avr-libc avrdude
	rustup toolchain install nightly
	rustup override set nightly
	rustup component add rust-src --toolchain nightly
