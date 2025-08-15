use core::arch::asm;

#[inline(always)]
pub fn atomic<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    let sreg: u8;
    unsafe {
        asm!("in {0}, 0x3F", out(reg) sreg);
        asm!("cli");
    }

    let result = f();

    unsafe {
        asm!("out 0x3F, {0}", in(reg) sreg);
    }

    result
}
