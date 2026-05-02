TARGET_RS := atmega32u4-none

.PHONY: build

left: build
left: flash

right::
	IS_KEYBOARD_RIGHT=1 make

flash:
	avr-strip ./build/rust-keyboard.elf
	DEVICE=$$(./autoflash.sh); \
	avrdude -p m32u4 -c avr109 -P $$DEVICE -U flash:w:./build/rust-keyboard.elf -U eeprom:w:./build/rust-keyboard.elf

both: left
both: right

build:
	cargo build --release -Zjson-target-spec
	mkdir -p build
	cp ./target/$(TARGET_RS)/release/rust-keyboard.elf build

clean:
	cargo clean

build_env_setup:
	sudo pacman -S --needed avr-gcc avr-libc avrdude
	rustup toolchain install nightly
	rustup override set nightly
	rustup component add rust-src --toolchain nightly
