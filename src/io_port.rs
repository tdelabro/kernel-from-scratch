//! In/Out operations on ports

pub trait InOut {
    /// Read data from a port
    unsafe fn port_in(port: u16) -> Self;
    /// Write data to a port
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
            port: port,
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

/// An unsafe I/O port
///
/// Interaction with a specific unsafe port is done through this stucture.
/// The type specify the size of the data been read or written.
///
/// Read and write are executed through volatile instructions and therefore
/// will not be optimised out by the compilator.
///
/// # Examples
///
/// ```
/// let port128: Port<u8> = UnsafePort::new(0x80);
///
/// unsafe {
///     let value = port128.read();
///     port128.write(0xFF);
/// }
/// ```
#[derive(Clone, Copy)]
pub struct UnsafePort<T: InOut> {
    port: u16,
    phantom: PhantomData<T>,
}

impl<T: InOut> UnsafePort<T> {
    pub const unsafe fn new(port: u16) -> UnsafePort<T> {
        UnsafePort {
            port: port,
            phantom: PhantomData,
        }
    }

    pub unsafe fn read(&mut self) -> T {
        T::port_in(self.port)
    }

    pub unsafe fn write(&mut self, value: T) {
        T::port_out(self.port, value);
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
        llvm_asm!("out dx, al"
            :: "{dx}"(0x80), "{al}"(0)
            :: "intel", "volatile");
    }
}

fn outb(port: u16, val: u8) {
    unsafe {
        llvm_asm!("out dx, al"
            :: "{dx}"(port), "{al}"(val)
            :: "intel", "volatile");
    }
}

fn inb(port: u16) -> u8 {
    let result: u8;

    unsafe {
        llvm_asm!("in al, dx"
            : "={al}"(result) : "{dx}"(port)
            :: "intel", "volatile");
    }
    result
}

fn outw(port: u16, val: u16) {
    unsafe {
        llvm_asm!("out dx, ax"
            :: "{dx}"(port), "{ax}"(val)
            :: "intel", "volatile");
    }
}

fn inw(port: u16) -> u16 {
    let result: u16;

    unsafe {
        llvm_asm!("in ax, dx"
            : "={ax}"(result) : "{dx}"(port)
            :: "intel", "volatile");
    }
    result
}

fn outl(port: u16, val: u32) {
    unsafe {
        llvm_asm!("out dx, eax"
            :: "{dx}"(port), "{eax}"(val)
            :: "intel", "volatile");
    }
}

fn inl(port: u16) -> u32 {
    let result: u32;

    unsafe {
        llvm_asm!("in eax, dx"
            : "={eax}"(result) : "{dx}"(port)
            :: "intel", "volatile");
    }
    result
}
