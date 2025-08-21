use linux_embedded_hal::I2cdev;
use embedded_hal::i2c::I2c;
use std::sync::{Arc, Mutex};

#[allow(dead_code)]
pub const TCA9548A_ADDRESS: u8 = 0x70;

pub struct Tca9548a {
    i2c: Arc<Mutex<I2cdev>>,
    address: u8,
    current_channel: Option<u8>,
}

impl Tca9548a {
    #[allow(dead_code)]
    pub fn new(i2c: Arc<Mutex<I2cdev>>) -> Self {
        Self::with_address(i2c, TCA9548A_ADDRESS)
    }

    pub fn with_address(i2c: Arc<Mutex<I2cdev>>, address: u8) -> Self {
        Self {
            i2c,
            address,
            current_channel: None,
        }
    }

    pub fn select_channel(&mut self, channel: u8) -> Result<(), Box<dyn std::error::Error>> {
        if channel > 7 {
            return Err("Channel must be between 0 and 7".into());
        }

        let channel_mask = 1u8 << channel;
        
        let mut i2c = self.i2c.lock().unwrap();
        i2c.write(self.address, &[channel_mask])?;
        drop(i2c);
        
        self.current_channel = Some(channel);
        Ok(())
    }

    #[allow(dead_code)]
    pub fn disable_all_channels(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut i2c = self.i2c.lock().unwrap();
        i2c.write(self.address, &[0x00])?;
        drop(i2c);
        
        self.current_channel = None;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn get_current_channel(&self) -> Option<u8> {
        self.current_channel
    }

    #[allow(dead_code)]
    pub fn get_i2c(&self) -> Arc<Mutex<I2cdev>> {
        Arc::clone(&self.i2c)
    }
}

#[allow(dead_code)]
pub struct MultiplexedI2c {
    multiplexer: Arc<Mutex<Tca9548a>>,
    channel: u8,
}

#[allow(dead_code)]
impl MultiplexedI2c {
    pub fn new(multiplexer: Arc<Mutex<Tca9548a>>, channel: u8) -> Self {
        Self {
            multiplexer,
            channel,
        }
    }

    pub fn with_channel<F, R>(&mut self, f: F) -> Result<R, Box<dyn std::error::Error>>
    where
        F: FnOnce(&mut I2cdev) -> Result<R, Box<dyn std::error::Error>>,
    {
        let mut mux = self.multiplexer.lock().unwrap();
        mux.select_channel(self.channel)?;
        
        let i2c = mux.get_i2c();
        let mut i2c_lock = i2c.lock().unwrap();
        
        f(&mut *i2c_lock)
    }
}