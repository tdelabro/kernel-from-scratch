//! PS/2 driver
//!
//! # Features
//! - Proper initialisation routine
//! - Read scan codes from the buffer

use crate::io_port;
use crate::spin::Mutex;

/// The PS/2 port representation
pub struct Ps2 {
    buffer: io_port::Port<u8>,
    helper: io_port::Port<u8>,
}

impl Ps2 {
    fn status(&self) -> u8 {
        self.helper.read()
    }

    fn command(&self, cmd: u8) {
        self.helper.write(cmd);
    }

    /// Read scan code from the PS/2 buffer
    pub fn read(&self) -> u8 {
        let mut s = self.status();
        while s & 1 == 0 {
            s = self.status();
        }
        self.buffer.read()
    }

    fn write(&self, value: u8) {
        let mut s = self.status();
        while s & 1 << 1 != 0 {
            s = self.status();
        }
        self.buffer.write(value);
    }

    fn get_config(&self) -> u8 {
        self.command(0x20);
        self.read()
    }

    fn set_config(&self, conf: u8) {
        self.command(0x60);
        self.write(conf);
    }

    fn interface_test_ok(&self, cmd: u8, n: u8) -> bool {
        self.command(cmd);
        print!("    Port {}: ", n);
        match self.read() {
            0x0 => {
                println!("OK");
                return true;
            }
            0x01 => println!("Failed: clock line stuck low"),
            0x02 => println!("Failed: clock line stuck high"),
            0x03 => println!("Failed: data line stuck low"),
            0x04 => println!("Failed: data line stuck high"),
            _ => println!("Failed: unknown error"),
        }
        false
    }

    /// Initialise the PS/2 controler with proper config
    ///
    /// Must be executed before use.
    pub fn init(&self) {
        self.command(0xAD);
        self.command(0xA7);
        self.buffer.read();

        self.set_config(self.get_config() & 0xBC); //0b10111100
        self.command(0xAA);
        match self.read() {
            0x55 => println!("PS/2 controller self test passed"),
            _ => println!("PS/2 controller self test failed"),
        };

        let mut is_dual_port = false;
        self.command(0xA8);
        match self.get_config() {
            conf if conf & 1 << 5 == 0 => {
                is_dual_port = true;
                self.command(0xA7);
            }
            _ => println!("PS/2 does not support dual port"),
        }

        println!("Interface test");
        let mut count_available_port = 0u8;
        count_available_port |= self.interface_test_ok(0xAB, 1) as u8;
        if is_dual_port {
            count_available_port |= (self.interface_test_ok(0xA9, 2) as u8) << 1;
        }
        if count_available_port == 0 {
            println!("PS/2 have no functional interface.");
            return;
        }

        let mut conf = self.get_config();
        if count_available_port & 1 != 0 {
            self.command(0xAE);
            conf |= 1;
        }
        if (count_available_port & 1 << 1) != 0 {
            self.command(0xA8);
            conf |= 1 << 1;
        }
        self.set_config(conf);

        println!("PS/2 successfully initialized.");
    }
}

/// The PS/2 controler used by our kernel
pub static PS2: Mutex<Ps2> = Mutex::new(Ps2 {
    buffer: io_port::Port::new(0x60),
    helper: io_port::Port::new(0x64),
});
