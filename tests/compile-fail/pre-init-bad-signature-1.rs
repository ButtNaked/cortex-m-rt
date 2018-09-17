#![no_main]
#![no_std]

extern crate cortex_m_rt;
extern crate panic_halt;

use cortex_m_rt::{entry, pre_init};

#[pre_init] //~ ERROR custom attribute panicked
//~^ HELP `#[pre_init]` function must have signature `unsafe fn()`
fn foo() {}

#[entry]
fn bar() -> ! {
    loop {}
}
