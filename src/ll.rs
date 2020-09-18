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

        for (i, val) in value.iter().enumerate() {
            // Send the address
            self.address_pin
                .set_low()
                .map_err(|_| Self::InterfaceError::AddressPinError)?;

            self.communication_interface
                .write(&[address + i as u8])
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
                .write(&[*val])
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
        }

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
        /// Register containing the Waveform Select Enable and some test fields
        waveform_select_enable(RW, 0x01, 1) = {
            /// Must be set to zero before any operation
            test0: u8 = RW 6..=7,
            /// If clear, all channels will use normal sine wave. If set, register E0-F5 (Waveform Select) contents will be used.
            /// OPL3 doesn't implement this bit and should be left at 0.
            waveform_select_enable: u8 as Bit = RW 5..=5,
            /// Must be set to zero before any operation
            test1: u8 = RW 0..=4,
        },
        /// Upward 8 bit counter with a resolution of 80 µsec. If an overflow occurs, the status register bit is set, and the preset value is loaded into the timer again. 
        timer1_count(RW, 0x02, 1) = {
            preset_value: u8 = RW 0..8,
        },
        /// Same as Timer 1, but with a resolution of 320 µsec. 
        timer2_count(RW, 0x03, 1) = {
            preset_value: u8 = RW 0..8,
        },
        /// Controls the IRQ and timer settings
        timer_control(RW, 0x04, 1) = {
            /// Resets timers and IRQ flags in status register. All other bits are ignored when this bit is set.
            irq_reset: u8 as Bit = RW 7..=7,
            /// If set, status register is not affected in timer 1 overflow.
            timer1_mask: u8 as Bit = RW 6..=6,
            /// If set, status register is not affected in timer 2 overflow.
            timer2_mask: u8 as Bit = RW 5..=5,
            /// Timer 2 on or off
            timer2_start: u8 as Bit = RW 1..=1,
            /// Timer 1 on or off
            timer1_start: u8 as Bit = RW 0..=0,
        },
        note_select(RW, 0x08, 1) = {
            /// Composite sine wave mode on/off. All KEY-ON bits must be clear in order to use this mode.
            /// The card is unable to create any other sound when in CSW mode.
            /// The CSW mode is not implemented on an OPL3 and this bit is ignored.
            composite_sine_wave: u8 as Bit = RW 7..=7,
            /// Controls the split point of the keyboard. When 0, the keyboard split is the second bit from the bit 8 of the F-Number. When 1, the MSb of the F-Number is used.
            note_select: u8 as Bit = RW 6..=6,
        },
        operator_settings0(RW, [0x20, 0x21, 0x22, 0x23, 0x24, 0x25, 0x26, 0x27, 0x28, 0x29, 0x2A, 0x2B, 0x2C, 0x2D, 0x2E, 0x2F, 0x30, 0x31, 0x32, 0x33, 0x34, 0x35], 1) = {
            /// Apply amplitude modulation when set; AM depth is controlled by the AM-Depth flag in address BD. 
            amplitude_modulation: u8 as Bit = RW 7..=7,
            /// Apply vibrato when set; vibrato depth is controlled by the Vib-Depth flag in address BD.
            vibrato: u8 as Bit = RW 6..=6,
            /// When set, the sustain level of the voice is maintained until released; when clear, the sound begins to decay immediately after hitting the SUSTAIN phase.
            sustain: u8 as Bit = RW 5..=5,
            /// Keyboard scaling rate. This is another incomprehensible bit in the Sound Blaster manual. From experience, if this bit is set, the sound's envelope is foreshortened as it rises in pitch. 
            keyboard_scaling_rate: u8 as Bit = RW 4..=4,
            /// These bits indicate which harmonic the operator will produce sound (or modulation) in relation to the voice's specified frequency.
            modulator_frequency_multiple: u8 as ModulatorFrequencyMultiple = RW 0..=3,
        },
        operator_settings1(RW, [0x40, 0x41, 0x42, 0x43, 0x44, 0x45, 0x46, 0x47, 0x48, 0x49, 0x4A, 0x4B, 0x4C, 0x4D, 0x4E, 0x4F, 0x50, 0x51, 0x52, 0x53, 0x54, 0x55], 1) = {
            /// Causes output levels to decrease as the frequency rises
            level_key_scaling: u8 as ScalingLevel = RW 6..=7,
            /// Attenuates the operator output level. 0 is the loudest, 3F is the softest. Attenuation range is 48dB with 0.75dB resolution.
            output_level: u8 = RW 0..=5,
        },
    }
);

/// 4 bits
#[repr(u8)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, IntoPrimitive, TryFromPrimitive)]
pub enum ModulatorFrequencyMultiple {
    /// Factor 0.5
    OneOctaveBelow = 0x0,
    /// Factor 1
    AtSpecified = 0x1,
    /// Factor 2
    OneOctaveAbove = 0x2,
    /// Factor 3
    OneOctaveFifthAbove = 0x3,
    /// Factor 4
    TwoOctaveAbove = 0x4,
    /// Factor 5
    TwoOctaveMajorThirdAbove = 0x5,
    /// Factor 6
    TwoOctaveFifthAbove = 0x6,
    /// Factor 7
    TwoOctaveMinorSeventhAbove = 0x7,
    /// Factor 8
    ThreeOctaveAbove = 0x8,
    /// Factor 9
    ThreeOctaveMajorSecondAbove = 0x9,
    /// Factor 10
    ThreeOctaveMajorThirdAbove = 0xA,
    /// Same as ThreeOctaveMajorThirdAbove
    ThreeOctaveMajorThirdAboveAlt = 0xB,
    /// Factor 12
    ThreeOctaveFifthAbove = 0xC,
    /// Same as ThreeOctaveFifthAbove
    ThreeOctaveFifthAboveAlt = 0xD,
    /// Factor 15
    ThreeOctaveMajorSeventhAbove = 0xE,
    /// Same as ThreeOctaveMajorSeventhAbove
    ThreeOctaveMajorSeventhAboveAlt = 0xF,
}

/// 2 bits
#[repr(u8)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, IntoPrimitive, TryFromPrimitive)]
pub enum ScalingLevel {
    NoChange = 0b00,
    DB3PerOctave = 0b01,
    DB1_5PerOctave = 0b10,
    DB6PerOctave = 0b11,
}
