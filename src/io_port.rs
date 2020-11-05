pub struct Port(pub u16);

impl Port {
    pub fn read(&self) -> u8{
        inb(self.0)
    }
    pub fn write(&self, val: u8) {
        outb(self.0, val);
    }
}

fn outb(port: u16, val: u8) {
    unsafe {
        llvm_asm!("out dx, al" : : "{dx}"(port), "{al}"(val) : : "intel", "volatile");
    }
}

fn inb(port: u16) -> u8 {
    let result: u8;

    unsafe {
        llvm_asm!("in al, dx" : "={al}"(result) : "{dx}"(port) : : "intel");
    }
    result
}
