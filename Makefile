TARGET_RS := atmega32u4


left: build
left: flash

right::
	IS_KEYBOARD_RIGHT=1 make build
right:: flash

flash:
	avr-strip ./build/rust_keyboard.elf
	DEVICE=$$(./autoflash.sh); \
	avrdude -p m32u4 -c avr109 -P $$DEVICE -U flash:w:./build/rust_keyboard.elf

both: left
both: right

build: the_rest/lufa-rs/lufa/LUFA
	cd the_rest && cargo build --release
	mkdir -p build
	cp ./the_rest/target/$(TARGET_RS)/release/rust_keyboard.elf build

the_rest/lufa-rs/lufa/LUFA:
	@echo You need to init and update the lufa git submodule. Run git submodule update --init the_rest/lufa-rs/lufa
	@false
	
clean:
	cargo clean

build_env_setup:
	sudo pacman -S --needed avr-gcc avr-libc avrdude
	cd the_rest && rustup toolchain install nightly \
	&& rustup override set nightly \
	&& rustup component add rust-src --toolchain nightly
