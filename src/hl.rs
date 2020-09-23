use crate::ll;
use core::marker::PhantomData;
use device_driver::{Bit, ll::LowLevelDevice};
use ll::InstrumentMode;

pub struct Uninitialized;
pub struct Melody;
pub struct Rhythm;

pub trait Initialized {
    const CHANNEL_COUNT: usize;
}

impl Initialized for Melody {
    const CHANNEL_COUNT: usize = 9;
}
impl Initialized for Rhythm {
    const CHANNEL_COUNT: usize = 6;
}

#[derive(Debug)]
pub enum Opl2Error {
    LowLevelError(ll::LowLevelError),
}

impl<LLE: Into<ll::LowLevelError>> From<LLE> for Opl2Error {
    fn from(low_level_error: LLE) -> Self {
        Opl2Error::LowLevelError(low_level_error.into())
    }
}

pub struct Opl2<I: ll::HardwareInterface, STATE> {
    ll: ll::Opl2LL<I>,
    phantom: PhantomData<STATE>,
}

impl<I: ll::HardwareInterface> Opl2<I, Uninitialized> {
    pub fn new(interface: I) -> Self {
        Self {
            ll: ll::Opl2LL::new(interface),
            phantom: PhantomData::default(),
        }
    }

    pub fn initialize(mut self) -> Result<Opl2<I, Melody>, Opl2Error> {
        self.ll.interface().reset()?;

        Ok(Opl2 {
            ll: self.ll,
            phantom: PhantomData::default(),
        })
    }
}

impl<I: ll::HardwareInterface, INIT: Initialized> Opl2<I, INIT> {
    // Map that gives the two operator indices for each channel
    const OPERATOR_MAP: [(usize, usize); 9] = [
        (0x00, 0x03),
        (0x01, 0x04),
        (0x02, 0x05),
        (0x08, 0x0B),
        (0x09, 0x0C),
        (0x0A, 0x0D),
        (0x10, 0x13),
        (0x11, 0x14),
        (0x12, 0x15),
    ];

    pub fn ll(&mut self) -> ll::registers::RegisterSet<I> {
        self.ll.registers()
    }
}

impl<I: ll::HardwareInterface> Opl2<I, Melody> {
    pub fn into_rhythm_mode(mut self) -> Result<Opl2<I, Rhythm>, Opl2Error> {
        // KEY-ON registers for channels 06, 07, and 08 must be OFF in order to use the rhythm section. Other parameters such as attack/decay/sustain/release must also be set appropriately.
        for i in 6..=8 {
            self.ll().channel_settings1().modify_index(i, |_, w| w.key_on(Bit::Cleared))?;
            self.ll().operator_settings2().write_index(Self::OPERATOR_MAP[i].0, |w| w.attack_rate(0xF).decay_rate(0))?;
            self.ll().operator_settings2().write_index(Self::OPERATOR_MAP[i].1, |w| w.attack_rate(0xF).decay_rate(0))?;
            self.ll().operator_settings3().write_index(Self::OPERATOR_MAP[i].0, |w| w.sustain_level(0).release_rate(0xF))?;
            self.ll().operator_settings3().write_index(Self::OPERATOR_MAP[i].1, |w| w.sustain_level(0).release_rate(0xF))?;
        }

        self.ll().rhythm_settings().modify(|_, w| w.instrument_mode(InstrumentMode::Percussion))?;

        Ok(Opl2 { ll: self.ll, phantom: PhantomData::default()})
    }
}

impl<I: ll::HardwareInterface> Opl2<I, Rhythm> {
    pub fn into_melody_mode(mut self) -> Result<Opl2<I, Melody>, Opl2Error> {
        self.ll().rhythm_settings().modify(|_, w| w.instrument_mode(InstrumentMode::Melodic))?;
        Ok(Opl2 { ll: self.ll, phantom: PhantomData::default()})
    }

    pub fn bass_drum(&mut self, value: bool) -> Result<(), Opl2Error> {
        self.ll().rhythm_settings().modify(|_, w| w.bass_drum_on(value.into()))?;
        Ok(())
    }

    pub fn snare_drum(&mut self, value: bool) -> Result<(), Opl2Error> {
        self.ll().rhythm_settings().modify(|_, w| w.snare_drum_on(value.into()))?;
        Ok(())
    }

    pub fn tom_tom(&mut self, value: bool) -> Result<(), Opl2Error> {
        self.ll().rhythm_settings().modify(|_, w| w.tom_tom_on(value.into()))?;
        Ok(())
    }

    pub fn cymbal(&mut self, value: bool) -> Result<(), Opl2Error> {
        self.ll().rhythm_settings().modify(|_, w| w.cymbal_on(value.into()))?;
        Ok(())
    }

    pub fn hi_hat(&mut self, value: bool) -> Result<(), Opl2Error> {
        self.ll().rhythm_settings().modify(|_, w| w.hi_hat_on(value.into()))?;
        Ok(())
    }
}
