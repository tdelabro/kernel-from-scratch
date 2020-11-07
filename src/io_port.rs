/// A module to handle In/Out operations on ports

pub trait InOut {
    unsafe fn port_in(port: u16) -> Self;
    unsafe fn port_out(port: u16, value: Self);
}

impl InOut for u8 {
    unsafe fn port_in(port: u16) -> u8 { inb(port) }
    unsafe fn port_out(port: u16, value: u8) { outb(port, value); }
}

impl InOut for u16 {
    unsafe fn port_in(port: u16) -> u16 { inw(port) }
    unsafe fn port_out(port: u16, value: u16) { outw(port, value); }
}

impl InOut for u32 {
    unsafe fn port_in(port: u16) -> u32 { inl(port) }
    unsafe fn port_out(port: u16, value: u32) { outl(port, value); }
}

use core::marker::PhantomData;

#[derive(Debug, Clone, Copy)]
pub struct Port<T: InOut> {
    port: u16,
    phantom: PhantomData<T>,	
}

impl <T: InOut> Port<T> {
    pub const fn new(port: u16) -> Port<T> {
	Port { port: port, phantom: PhantomData }
    }

    pub fn read(&self) -> T {
        unsafe { T::port_in(self.port) }
    }

    pub fn write(&self, value: T) {
        unsafe { T::port_out(self.port, value); }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct UnsafePort<T: InOut> {
    port: u16,
    phantom: PhantomData<T>,
}

impl<T: InOut> UnsafePort<T> {
    pub const unsafe fn new(port: u16) -> UnsafePort<T> {
        UnsafePort { port: port, phantom: PhantomData }
    }

    pub unsafe fn read(&mut self) -> T {
        T::port_in(self.port)
    }

    pub unsafe fn write(&mut self, value: T) {
        T::port_out(self.port, value);
    }
}


/// Used to wait beetween two I/O call when no other way exist.
/// Prefer IRQ and buffer ready bytes when possible
pub fn wait() {
    unsafe {
	llvm_asm!("out dx, al"
            :: "{dx}"(0x80), "{al}"(0)
            :: "intel", "volatile");
    }
}

/// Write one byte to port
fn outb(port: u16, val: u8) {
    unsafe {
	llvm_asm!("out dx, al"
            :: "{dx}"(port), "{al}"(val)
            :: "intel", "volatile");
    }
}

/// Read one byte from port
fn inb(port: u16) -> u8 {
    let result: u8;

    unsafe {
	llvm_asm!("in al, dx"
            : "={al}"(result) : "{dx}"(port)
            :: "intel", "volatile");
    }
    result
}

/// Write one word to port
fn outw(port: u16, val: u16) {
    unsafe {
	llvm_asm!("out dx, ax"
            :: "{dx}"(port), "{ax}"(val)
            :: "intel", "volatile");
    }
}

/// Read one word from port
fn inw(port: u16) -> u16 {
    let result: u16;

    unsafe {
	llvm_asm!("in ax, dx"
            : "={ax}"(result) : "{dx}"(port)
            :: "intel", "volatile");
    }
    result
}

/// Write one double word to port
fn outl(port: u16, val: u32) {
    unsafe {
	llvm_asm!("out dx, eax"
            :: "{dx}"(port), "{eax}"(val)
            :: "intel", "volatile");
    }
}

/// Read one double word from port
fn inl(port: u16) -> u32 {
    let result: u32;

    unsafe {
	llvm_asm!("in eax, dx"
            : "={eax}"(result) : "{dx}"(port)
            :: "intel", "volatile");
    }
    result
}

