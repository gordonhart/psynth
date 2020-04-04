#![cfg(feature = "hardware")]

use std::cell::RefCell;
use std::sync::mpsc;
use std::thread;

use anyhow::Result;
use embedded_hal::blocking::i2c::Read;
use linux_embedded_hal::I2cdev;

use crate::Pot;


pub struct BlockingI2cPot<T> {
    address: u8,
    device: RefCell<I2cdev>,
    buffer: RefCell<Vec<u8>>,
    converter: Box<dyn Fn(&[u8]) -> T + Send>,
}


impl<T> BlockingI2cPot<T> {
    pub fn new<F>(
        bus: u8,
        address: u8,
        message_size: usize, // number of bytes that comprise a single reading
        converter: F,
    ) -> Result<BlockingI2cPot<T>>
    where
        F: Fn(&[u8]) -> T + Send + 'static,
    {
        let mut device = I2cdev::new(format!("/dev/i2c-{}", bus))?;
        device.set_slave_address(address as u16)?;
        Ok(BlockingI2cPot {
            address: address,
            device: RefCell::new(device),
            buffer: RefCell::new(vec![0; message_size]),
            converter: Box::new(converter),
        })
    }
}


impl<T> Pot<T> for BlockingI2cPot<T> {
    fn read(&self) -> T {
        let mut device = self.device.borrow_mut();
        let mut buffer = self.buffer.borrow_mut();
        // TODO: not panic
        device.read(self.address, buffer.as_mut_slice()).expect("i2c error");
        println!("{:?}", buffer);
        (self.converter)(buffer.as_slice())
    }
}


pub struct ThreadedI2cPot<T: Sized + Send + Copy> {
    receiver: mpsc::Receiver<T>,
    latest: Cell<T>,
}


impl<T> ThreadedI2cPot<T>
where
    T: Sized + Send + Copy + 'static, // static required for thread::spawn
{
    pub fn new<F>(
        bus: u8,
        address: u8,
        message_size: usize,
        converter: F,
    ) -> Result<ThreadedI2cPot<T>>
    where
        F: Fn(&[u8]) -> T + Send + 'static,
    {
        let (sender, receiver) = mpsc::channel();

        let blocking_pot = BlockingI2cPot::new(bus, address, message_size, converter)?;
        thread::spawn(move || loop {
            let new_val = blocking_pot.read();
            sender.send(new_val).expect("channel closed");
        });

        let first_val = receiver.recv()?;
        Ok(ThreadedI2cPot {
            receiver: receiver,
            latest: Cell::new(first_val),
        })
    }
}


impl<T> Pot<T> for ThreadedI2cPot<T>
where
    T: Sized + Send + Copy,
{
    fn read(&self) -> T {
        match self.receiver.try_recv() {
            Ok(val) => {
                self.latest.set(val);
                val
            },
            Err(_) => self.latest.get(),
        }
    }
}
