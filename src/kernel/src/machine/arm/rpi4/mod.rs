mod gpio;
pub mod info;
pub mod interrupt;
pub mod memory;
pub mod processor;
pub mod serial;

pub mod tests;

pub fn machine_post_init() {
    // TODO: initialize uart with interrupts
}
