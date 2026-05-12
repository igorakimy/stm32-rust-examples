build:
	cargo build

debug:
	arm-none-eabi-gdb /home/aki/Projects/Coding/Rust/stm32-rust-examples/examples/led-blink/target/thumbv7em-none-eabihf/release/led-blink

renode:
	renode --console led-blink.resc

disasm:
	cargo objdump --bin led-blink --release -- --disassemble --no-show-raw-insn --print-imm-hex
