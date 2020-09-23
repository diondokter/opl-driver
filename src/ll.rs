use device_driver::ll::register::RegisterInterface;
use device_driver::{create_low_level_device, implement_registers};

use core::fmt::Debug;
use embedded_hal::blocking::delay::{DelayMs, DelayUs};
use embedded_hal::blocking::spi::Write;
use embedded_hal::digital::v2::OutputPin;
use num_enum::{IntoPrimitive, TryFromPrimitive};

pub use device_driver::Bit;

#[derive(Debug)]
pub enum InterfaceError {
    AddressPinError,
    LatchPinError,
    ResetPinError,
    CommunicationError,
}

/// Our hardware interface with the chip using the shift register that is present on the opl2 audio board by Maarten Janssen
pub struct ShiftInterface<
    SPI: Write<u8>,
    A: OutputPin,
    L: OutputPin,
    R: OutputPin,
    D: DelayUs<u8> + DelayMs<u8>,
> {
    /// The spi interface we use to drive the shift register
    communication_interface: SPI,
    /// The pin connected to the A0 input
    address_pin: A,
    /// The pin connected to the latch input of the shift register
    latch_pin: L,
    /// The pin connected to the reset input
    reset_pin: R,
    /// Some kind of delay provider
    delay: D,
    /// A copy of all the registers in memory.
    ///
    /// We need this because we can't read the OPL registers.
    /// By keeping track of this ourselves, we can still present a read/write interface which is useful for modifying registers.
    registers: [u8; u8::max_value() as usize],
}

impl<SPI: Write<u8>, A: OutputPin, L: OutputPin, R: OutputPin, D: DelayUs<u8> + DelayMs<u8>>
    ShiftInterface<SPI, A, L, R, D>
{
    /// Creates a new hardware interface
    pub fn new(
        communication_interface: SPI,
        address_pin: A,
        latch_pin: L,
        reset_pin: R,
        delay: D,
    ) -> Self {
        Self {
            communication_interface,
            address_pin,
            latch_pin,
            reset_pin,
            delay,
            registers: [0; u8::max_value() as usize],
        }
    }

    /// Destructs the hardware interface into its pieces.
    pub fn free(self) -> (SPI, A, L, R) {
        (
            self.communication_interface,
            self.address_pin,
            self.latch_pin,
            self.reset_pin,
        )
    }
}

impl<SPI: Write<u8>, A: OutputPin, L: OutputPin, R: OutputPin, D: DelayUs<u8> + DelayMs<u8>>
    HardwareInterface for ShiftInterface<SPI, A, L, R, D>
{
    fn reset(&mut self) -> Result<(), InterfaceError> {
        // Set the pins to the default level
        self.latch_pin
            .set_high()
            .map_err(|_| InterfaceError::LatchPinError)?;
        self.reset_pin
            .set_high()
            .map_err(|_| InterfaceError::ResetPinError)?;
        self.address_pin
            .set_low()
            .map_err(|_| InterfaceError::AddressPinError)?;

        // Make a reset cycle
        self.reset_pin
            .set_low()
            .map_err(|_| InterfaceError::ResetPinError)?;
        self.delay.delay_ms(1);
        self.reset_pin
            .set_high()
            .map_err(|_| InterfaceError::ResetPinError)?;

        // Reset the internal registers
        self.registers = [0x00; 0xFF];
        self.write_register(0x00, &[0x00; 0xFF])?;

        Ok(())
    }
}

/// Implementing the register interface for the hardware interface
impl<SPI: Write<u8>, A: OutputPin, L: OutputPin, R: OutputPin, D: DelayUs<u8> + DelayMs<u8>>
    RegisterInterface for ShiftInterface<SPI, A, L, R, D>
{
    type Address = u8;
    type InterfaceError = InterfaceError;

    fn read_register(
        &mut self,
        address: Self::Address,
        value: &mut [u8],
    ) -> Result<(), Self::InterfaceError> {
        value
            .copy_from_slice(&self.registers[(address as usize)..(address as usize + value.len())]);
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
create_low_level_device!(
    /// Low level access to the Opl2 chip
    Opl2LL {
        // The types of errors our low level error enum must contain
        errors: [InterfaceError],
        hardware_interface_requirements: { RegisterInterface<Address = u8, InterfaceError = InterfaceError> },
        hardware_interface_capabilities: {
            /// Asserts the reset pin
            fn reset(&mut self) -> Result<(), InterfaceError>;
        }
    }
);

// Create a register set for the device
implement_registers!(
    /// The global register set
    Opl2LL.registers<u8> = {
        /// Register containing the Waveform Select Enable and some test fields
        waveform_select_enable(RW, 0x01, 1) = {
            /// Must be set to zero before any operation
            test0: u8 = RW 6..=7,
            /// If clear, all channels will use normal sine wave. If set, register E0-F5 (Waveform Select) contents will be used.
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
        operator_settings2(RW, [0x60, 0x61, 0x62, 0x63, 0x64, 0x65, 0x66, 0x67, 0x68, 0x69, 0x6A, 0x6B, 0x6C, 0x6D, 0x6E, 0x6F, 0x70, 0x71, 0x72, 0x73, 0x74, 0x75], 1) = {
            /// Determines the rising time for the sound. The higher the value, the faster the attack. If value is 0, the sound will never attack, and if value is 15, the volume jumps directly from minimum to maximum.
            attack_rate: u8 = RW 4..=7,
            /// Determines the diminishing time for the sound. The higher the value, the shorter the decay. If value is 0, the sound does not decay towards sustain level and stays at maximum volume after attack.
            decay_rate: u8 = RW 0..=3,
        },
        operator_settings3(RW, [0x80, 0x81, 0x82, 0x83, 0x84, 0x85, 0x86, 0x87, 0x88, 0x89, 0x8A, 0x8B, 0x8C, 0x8D, 0x8E, 0x8F, 0x90, 0x91, 0x92, 0x93, 0x94, 0x95], 1) = {
            /// Determines the point at which the sound ceases to decay and chages to a sound having a constant level.
            /// The sustain level is expressed as a fraction of the maximum level. 15 is the softest and 0 is the loudest sustain level.
            ///
            /// *Note: the Sustain-bit in the register 20-35 must be set for this to have an effect.
            /// Otherwise the sound will continue with release phase after hitting sustain level.*
            ///
            /// LSB is -3dB
            ///
            /// There is an exception when all bits are set (value=15), the actual level is -93dB instead, matching as if the value were 31.
            sustain_level: u8 = RW 4..=7,
            /// Determines the rate at which the sound disappears after KEY-OFF. The higher the value, the shorter the release. Value of 0 causes the sound not to release at all, it will continue to produce sound at level before KEY-OFF.
            release_rate: u8 = RW 0..=3,
        },
        channel_settings0(RW, [0xA0, 0xA1, 0xA2, 0xA3, 0xA4, 0xA5, 0xA6, 0xA7, 0xA8], 1) = {
            frequency_number_low: u8 = RW 0..8,
        },
        channel_settings1(RW, [0xB0, 0xB1, 0xB2, 0xB3, 0xB4, 0xB5, 0xB6, 0xB7, 0xB8], 1) = {
            /// Channel is voiced when set, silent when clear.
            key_on: u8 as Bit = RW 5..=5,
            /// Octave (0-7). 0 is lowest, 7 is highest.
            block_number: u8 = RW 2..=4,
            frequency_number_high: u8 = RW 0..=1,
        },
        rhythm_settings(RW, 0xBD, 1) = {
            /// Tremolo (Amplitude Vibrato) Depth. 0 = 1.0dB, 1 = 4.8dB
            tremolo_depth: u8 as TremoloDepth = RW 7..=7,
            /// Frequency Vibrato Depth. 0 = 7 cents, 1 = 14 cents. A "cent" is 1/100 of a semi-tone.
            vibrato_depth: u8 as VibratoDepth = RW 6..=6,
            /// Percussion Mode. 0 = Melodic Mode, 1 = Percussion Mode.
            instrument_mode: u8 as InstrumentMode = RW 5..=5,
            bass_drum_on: u8 as Bit = RW 4..=4,
            snare_drum_on: u8 as Bit = RW 3..=3,
            tom_tom_on: u8 as Bit = RW 2..=2,
            cymbal_on: u8 as Bit = RW 1..=1,
            hi_hat_on: u8 as Bit = RW 0..=0,
        },
        channel_settings2(RW, [0xC0, 0xC1, 0xC2, 0xC3, 0xC4, 0xC5, 0xC6, 0xC7, 0xC8], 1) = {
            feedback: u8 = RW 1..=3,
            synthesis_type: u8 as SynthesisType = RW 0..=0,
        },
        operator_settings4(RW, [0xE0, 0xE1, 0xE2, 0xE3, 0xE4, 0xE5, 0xE6, 0xE7, 0xE8, 0xE9, 0xEA, 0xEB, 0xEC, 0xED, 0xEE, 0xEF, 0xF0, 0xF1, 0xF2, 0xF3, 0xF4, 0xF5], 1) = {
            waveform: u8 as WaveformType = RW 0..=1,
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

/// 1 bit
#[repr(u8)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, IntoPrimitive, TryFromPrimitive)]
pub enum TremoloDepth {
    /// 1.0dB
    Low = 0b0,
    /// 4.8dB
    High = 0b1,
}

/// 1 bit
#[repr(u8)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, IntoPrimitive, TryFromPrimitive)]
pub enum VibratoDepth {
    /// 7 cents
    Low = 0b0,
    /// 14 cents
    High = 0b1,
}

/// 1 bit
#[repr(u8)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, IntoPrimitive, TryFromPrimitive)]
pub enum InstrumentMode {
    Melodic = 0b0,
    Percussion = 0b1,
}

/// 1 bit
#[repr(u8)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, IntoPrimitive, TryFromPrimitive)]
pub enum SynthesisType {
    FrequencyModulation = 0b0,
    AdditiveSynthesis = 0b1,
}

/// 2 bits
#[repr(u8)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, IntoPrimitive, TryFromPrimitive)]
pub enum WaveformType {
    /// ```txt
    ///   ---         ---         ---         ---         ---         ---
    ///  /   \       /   \       /   \       /   \       /   \       /   
    /// /     \     /     \     /     \     /     \     /     \     /    
    ///        \   /       \   /       \   /       \   /       \   /     
    ///         ---         ---         ---         ---         ---      
    /// ```
    Sine = 0b00,
    /// ```txt
    ///   ---         ---         ---         ---         ---         ---
    ///  /   \       /   \       /   \       /   \       /   \       /   
    /// /     \-----/     \-----/     \-----/     \-----/     \-----/    
    ///
    ///
    /// ```
    HalfSine = 0b01,
    /// ```txt
    ///   ---   ---   ---   ---   ---   ---   ---   ---   ---   ---   ---
    ///  /   \ /   \ /   \ /   \ /   \ /   \ /   \ /   \ /   \ /   \ /   
    /// Y     Y     Y     Y     Y     Y     Y     Y     Y     Y     Y    
    ///     
    ///     
    /// ```
    AbsSine = 0b10,
    /// ```txt
    ///   /|           /|           /|           /|           /|         
    ///  / |          / |          / |          / |          / |         
    /// /  |---------/  |---------/  |---------/  |---------/  |---------
    ///     
    ///     
    /// ```
    PulseSine = 0b11,
}
