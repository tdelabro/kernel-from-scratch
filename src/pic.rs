use crate::io_port;

struct Pic {
    offset: u8,
    command: io_port::UnsafePort<u8>,
    data: io_port::UnsafePort<u8>,
}

impl Pic {
    fn handles_interrupt(&self, interupt_id: u8) -> bool {
        self.offset <= interupt_id && interupt_id < self.offset + 8
    }

    unsafe fn end_of_interrupt(&mut self) {
        self.command.write(0x20);   // EOI End Of Interupt
    }
}

pub struct ChainedPics {
    pics: [Pic; 2],
}

impl ChainedPics {
    pub const unsafe fn new(offset1: u8, offset2: u8) -> ChainedPics {
        ChainedPics {
            pics: [
                Pic {
                    offset: offset1,
                    command: io_port::UnsafePort::new(0x20),
                    data: io_port::UnsafePort::new(0x21),
                },
                Pic {
                    offset: offset2,
                    command: io_port::UnsafePort::new(0xA0),
                    data: io_port::UnsafePort::new(0xA1),
                },
            ]
        }
    }

    pub unsafe fn initialize(&mut self) {
        let saved_mask1 = self.pics[0].data.read();
        let saved_mask2 = self.pics[1].data.read();

        // Init signal
        self.pics[0].command.write(0x11);   // ICW1_INIT | ICW1_ICW4
        io_port::wait();
        self.pics[1].command.write(0x11);   // ICW1_INIT | ICW1_ICW4
        io_port::wait();

        // Vector offset
        self.pics[0].data.write(self.pics[0].offset);
        io_port::wait();
        self.pics[1].data.write(self.pics[1].offset);
        io_port::wait();

        // Git each pic information about the other
        self.pics[0].data.write(4);
        io_port::wait();
        self.pics[1].data.write(2);
        io_port::wait();

        // 8086/88 (MCS-80/85) mode
        self.pics[0].data.write(1);         // ICW4_8086
        io_port::wait();
        self.pics[1].data.write(1);         // ICW4_8086
        io_port::wait();

        self.pics[0].data.write(saved_mask1);
        self.pics[1].data.write(saved_mask2);
    }

    pub fn handles_interrupt(&self, interrupt_id: u8) -> bool {
        self.pics[0].handles_interrupt(interrupt_id)
            || self.pics[1].handles_interrupt(interrupt_id)
    }

    pub unsafe fn notify_end_of_interrupt(&mut self, interrupt_id: u8) {
        if self.handles_interrupt(interrupt_id) {
            if self.pics[1].handles_interrupt(interrupt_id) {
                self.pics[1].end_of_interrupt();
            }
            self.pics[0].end_of_interrupt();
        }
    }
}

use crate::spin::Mutex;

pub static PICS: Mutex<ChainedPics> = Mutex::new(
    unsafe { ChainedPics::new(0x20, 0x28) }
);

