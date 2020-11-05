use crate::io_port;
use crate::spin::Mutex;

pub struct Ps2 {
    buffer: io_port::Port,
    helper: io_port::Port,
}

impl Ps2 {
    fn status(&self) -> u8 {
        self.helper.read()
    }

    fn command(&self, cmd: u8) {
        let mut x = self.status();
        while x & 1 << 1 != 0 {
            x = self.status();
        }
        self.helper.write(cmd);
    }

    fn read(&self) -> u8 {
        let mut x = self.status();
        while x & 1 == 0 {
            x = self.status();
        }
        self.buffer.read()
    }

    fn write(&self, value: u8) {
        let mut x = self.status();
        while x & 1 << 1 != 0 {
            x = self.status();
        }
        self.buffer.write(value);
    }

    fn get_config(&self) -> u8{
        self.command(0x20);
        self.read()
    }

    fn set_config(&self, conf: u8) {
        self.command(0x60);
        self.write(conf);
    }
}

fn interface_test_ok(ps2: &Ps2, cmd: u8, n: u8) -> bool {
    ps2.command(cmd);
    print!("    Port {}: ",  n);
    match ps2.read() {
        0x0 =>  {
            println!("OK");
            return true;
        },
        0x01 => println!("Failed: clock line stuck low"),
        0x02 => println!("Failed: clock line stuck high"),
        0x03 => println!("Failed: data line stuck low"),
        0x04 => println!("Failed: data line stuck high"),
        _ => println!("Failed: unknown error"),
    }
    return false;
}
pub static PS2: Mutex<Ps2> = Mutex::new(Ps2 {
    buffer: io_port::Port(0x60),
    helper: io_port::Port(0x64),
});

pub fn init_ps2() {
    let ps2 = PS2.lock();
    ps2.command(0xAD);
    ps2.command(0xA7);
    ps2.buffer.read();
    ps2.set_config(ps2.get_config() & 0xBC); //0b10111100
    ps2.command(0xAA);
    match ps2.read() {
        0x55 => println!("PS/2 controller self test passed"),
        _ => println!("PS/2 controller self test failed"),
    };
    let mut is_dual_port = false; 
    ps2.command(0xA8);
    match ps2.get_config() {
        conf if conf & 1 << 5 == 0 => {
            println!("PS/2 support dual port. Disabled.");
            is_dual_port = true;
            ps2.command(0xA7);
        },
        _ => println!("PS/2 does not support dual port"),
    }
    println!("Interface test");
    let mut count_available_port = 0u8;
    count_available_port |= interface_test_ok(&ps2, 0xAB, 1) as u8;
    if is_dual_port {
        count_available_port |= (interface_test_ok(&ps2, 0xA9, 2) as u8) << 1;
    }
    if count_available_port == 0 {
       println!("PS/2 have no functional interface.");
       return;
    }
    let mut conf = ps2.get_config();
    if count_available_port & 1 != 0 {
        ps2.command(0xAE);
        conf |= 1;
    }
    if (count_available_port & 1 << 1) != 0 {
        ps2.command(0xA8);
        conf |= 1 << 1;
    }
    ps2.set_config(conf);
    println!("conf {:#010b}", ps2.get_config());
}

/*
let scan_code_set_1 = [
    '\0', '\0' /*esq*/,
    '1', '2', '3', '4', '5', '6', '7', '8', '9', '0',
    '-', '=',
    '\0' /*backspace*/, '\0' /*tab*/,
    'Q', 'W', 'E', 'R', 'T', 'Y', 'U', 'I', 'O', 'P',
    '[' ']',
    '\n' /*enter*/,
    '\0' /*left control*/,
    'A', 'S', 'D', 'F', 'G', 'H', 'J', 'K', 'L',
    ';', '\'', '`',
    '\0' /*left shift*/,
    '\\',
    'Z', 'X', 'C', 'V', 'B', 'N', 'M',
    ',', '.', '/',
    '\0' /*right shift*/,
    '*' /*(keypad)*/,
    '\0' /*left alt*/,
    ' ' /*space*/,
    '\0' /*CapsLock*/,
    '\0' /*F1*/, '\0' /*F2*/, '\0' /*F3*/, '\0' /*F4*/, '\0' /*F5*/, '\0' /*F6*/, '\0' /*F7*/, '\0' /*F8*/, '\0' /*F9*/, '\0' /*F10*/,
    '\0' /*NumberLock*/, '\0' /*ScrollLock*/,
    '7' /*(keypad)*/ , '8' /*(keypad)*/ , '9' /*(keypad)*/ , '-' /*(keypad)*/ , '4' /*(keypad)*/ , '5' /*(keypad)*/ , '6' /*(keypad)*/ , '+' /*(keypad)*/ , '1' /*(keypad)*/ , '2' /*(keypad)*/ , '3' /*(keypad)*/ , '0' /*(keypad)*/ , '.' /*(keypad)*/ ,
    '\0', '\0', '\0', '\0' /*F11*/, '\0' /*F12*/,
    ];
*/
