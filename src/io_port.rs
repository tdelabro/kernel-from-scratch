//! In/Out operations on ports

pub trait InOut {
    /// Read data from a port
    ///
    /// # Safety
    ///
    /// This function should be called on a port
    unsafe fn port_in(port: u16) -> Self;

    /// Write data to a port
    ///
    /// # Safety
    ///
    /// This function should be called on a port
    unsafe fn port_out(port: u16, value: Self);
}

impl InOut for u8 {
    unsafe fn port_in(port: u16) -> u8 {
        inb(port)
    }
    unsafe fn port_out(port: u16, value: u8) {
        outb(port, value);
    }
}

impl InOut for u16 {
    unsafe fn port_in(port: u16) -> u16 {
        inw(port)
    }
    unsafe fn port_out(port: u16, value: u16) {
        outw(port, value);
    }
}

impl InOut for u32 {
    unsafe fn port_in(port: u16) -> u32 {
        inl(port)
    }
    unsafe fn port_out(port: u16, value: u32) {
        outl(port, value);
    }
}

use core::marker::PhantomData;

/// An I/O port
///
/// Interaction with a specific port is done through this stucture.
/// The type specify the size of the data been read or written.
///
/// Read and write are executed through volatile instructions and therefore
/// will not be optimised out by the compilator.
///
/// # Examples
///
/// ```
/// let port128: Port<u8> = Port::new(0x80);
///
/// let value = port128.read();
/// port128.write(0xFF);
/// ```
pub struct Port<T: InOut> {
    port: u16,
    phantom: PhantomData<T>,
}

impl<T: InOut> Port<T> {
    pub const fn new(port: u16) -> Port<T> {
        Port {
            port,
            phantom: PhantomData,
        }
    }

    pub fn read(&self) -> T {
        unsafe { T::port_in(self.port) }
    }

    pub fn write(&self, value: T) {
        unsafe {
            T::port_out(self.port, value);
        }
    }
}

/// Wait between I/O calls
///
/// Prefer IRQ and buffer ready bytes when possible
///
/// # Examples
///
/// ```
/// let port128: Port<u8> = Port::new(0x80);
///
/// port128.write(0xFF);
/// wait();
/// let value = port128.read();
/// ```
pub fn wait() {
    unsafe {
        asm!("out dx, al", in("dx") 0x80, in("al") 0u8, options(nostack));
    }
}

fn outb(port: u16, val: u8) {
    unsafe {
        asm!("out dx, al", in("dx") port, in("al") val, options(nostack));
    }
}

fn inb(port: u16) -> u8 {
    let result: u8;

    unsafe {
        asm!("in al, dx", out("al") result, in("dx") port, options(nostack));
    }
    result
}

fn outw(port: u16, val: u16) {
    unsafe {
        asm!("out dx, ax", in("dx") port, in("ax") val, options(nostack));
    }
}

fn inw(port: u16) -> u16 {
    let result: u16;

    unsafe {
        asm!("in ax, dx", out("ax") result, in("dx") port, options(nostack));
    }
    result
}

fn outl(port: u16, val: u32) {
    unsafe {
        asm!("out dx, eax", in("dx") port, in("eax") val, options(nostack));
    }
}

fn inl(port: u16) -> u32 {
    let result: u32;

    unsafe {
        asm!("in eax, dx", out("eax") result, in("dx") port, options(nostack));
    }
    result
}
