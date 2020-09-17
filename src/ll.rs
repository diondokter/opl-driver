use device_driver::ll::register::RegisterInterface;
use device_driver::{create_low_level_device, implement_registers, Bit};

use embedded_hal::blocking::delay::DelayUs;
use embedded_hal::blocking::spi::Write;
use embedded_hal::digital::v2::OutputPin;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use std::fmt::Debug;

#[derive(Debug)]
pub enum InterfaceError {
    AddressPinError,
    LatchPinError,
    ResetPinError,
    CommunicationError,
}

/// Our full hardware interface with the chip
pub struct ShiftInterface<SPI: Write<u8>, A: OutputPin, L: OutputPin, R: OutputPin, D: DelayUs<u8>>
{
    pub communication_interface: SPI,
    pub address_pin: A,
    pub latch_pin: L,
    pub reset_pin: R,
    pub delay: D,
    registers: [u8; u8::max_value() as usize],
}

impl<SPI: Write<u8>, A: OutputPin, L: OutputPin, R: OutputPin, D: DelayUs<u8>>
    ShiftInterface<SPI, A, L, R, D>
{
    pub fn new(
        communication_interface: SPI,
        mut address_pin: A,
        mut latch_pin: L,
        mut reset_pin: R,
        delay: D,
    ) -> Result<Self, InterfaceError> {
        latch_pin
            .set_high()
            .map_err(|_| InterfaceError::LatchPinError)?;
        reset_pin
            .set_high()
            .map_err(|_| InterfaceError::ResetPinError)?;
        address_pin
            .set_low()
            .map_err(|_| InterfaceError::AddressPinError)?;

        Ok(Self {
            communication_interface,
            address_pin,
            latch_pin,
            reset_pin,
            delay,
            registers: [0; u8::max_value() as usize],
        })
    }

    pub fn free(self) -> (SPI, A, L, R) {
        (
            self.communication_interface,
            self.address_pin,
            self.latch_pin,
            self.reset_pin,
        )
    }
}

// Implementing the register interface for the hardware interface
impl<SPI: Write<u8>, A: OutputPin, L: OutputPin, R: OutputPin, D: DelayUs<u8>> RegisterInterface
    for ShiftInterface<SPI, A, L, R, D>
{
    type Address = u8;
    type InterfaceError = InterfaceError;

    fn read_register(
        &mut self,
        address: Self::Address,
        value: &mut [u8],
    ) -> Result<(), Self::InterfaceError> {
        value.copy_from_slice(&self.registers[(address as usize)..(address as usize + value.len())]);
        Ok(())
    }

    fn write_register(
        &mut self,
        address: Self::Address,
        value: &[u8],
    ) -> Result<(), Self::InterfaceError> {
        // Save in internal data store
        self.registers[(address as usize)..(address as usize + value.len())].copy_from_slice(value);

        // Send the address
        self.address_pin
            .set_low()
            .map_err(|_| Self::InterfaceError::AddressPinError)?;

        self.communication_interface
            .write(&[address])
            .map_err(|_| Self::InterfaceError::CommunicationError)?;

        // Apply the shift latch
        self.latch_pin
            .set_low()
            .map_err(|_| Self::InterfaceError::LatchPinError)?;
        self.delay.delay_us(1);
        self.latch_pin
            .set_high()
            .map_err(|_| Self::InterfaceError::LatchPinError)?;
        self.delay.delay_us(4);

        // Send the data
        self.address_pin
            .set_high()
            .map_err(|_| Self::InterfaceError::AddressPinError)?;

        self.communication_interface
            .write(value)
            .map_err(|_| Self::InterfaceError::CommunicationError)?;

        // Apply the shift latch
        self.latch_pin
            .set_low()
            .map_err(|_| Self::InterfaceError::LatchPinError)?;
        self.delay.delay_us(1);
        self.latch_pin
            .set_high()
            .map_err(|_| Self::InterfaceError::LatchPinError)?;
        self.delay.delay_us(23);

        Ok(())
    }
}

// Create our low level device. This holds all the hardware communication definitions
create_low_level_device!({
    // The name of our new low level device
    name: Opl2LL,
    // The types of errors our low level error enum must contain
    errors: [InterfaceError],
});

// Create a register set for the device
implement_registers!(
    /// The global register set
    Opl2LL.registers<u8> = {

    }
);
