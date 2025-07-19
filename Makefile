TARGET_RS := avr-none


left: build
left: flash

right::
	IS_KEYBOARD_RIGHT=1 make build
right:: flash

flash:
	DEVICE=$$(./autoflash.sh); \
	avrdude -p m32u4 -c avr109 -P $$DEVICE -U flash:w:./target/$(TARGET_RS)/release/rust_keyboard.elf

both: left
both: right

build:
	cargo build --release

clean:
	cargo clean

build_env_setup:
	sudo pacman -S --needed avr-gcc avr-libc avrdude
	rustup toolchain install nightly
	rustup override set nightly
	rustup component add rust-src --toolchain nightly
