pub type SerialCallback = Box<dyn FnMut(u8)>;

// TODO: serial INT
pub struct Serial {
    data: u8,
    control: u8,
    callback: SerialCallback,
}

impl Serial {
    pub fn new(callback: Option<SerialCallback>) -> Self {
        Serial {
            data: 0x00,
            control: 0x00,
            callback: callback.unwrap_or(Box::new(|_: u8| {})),
        }
    }

    pub fn read_data(&self) -> u8 {
        self.data
    }
    pub fn write_data(&mut self, data: u8) {
        self.data = data;
    }

    pub fn read_control(&self) -> u8 {
        self.control
    }
    pub fn write_control(&mut self, control: u8) {
        self.control = control;
        if control == 0x81 {
            (self.callback)(self.data);
        }
    }
}
