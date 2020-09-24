use crate::{
    instrument::BassDrum, instrument::Cymbal, instrument::HiHat, instrument::MelodyInstrument,
    instrument::OperatorSettings, instrument::SnareDrum, instrument::TomTom, ll,
};
use core::marker::PhantomData;
use device_driver::{ll::LowLevelDevice, Bit};
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
    InvalidChannel,
}

impl<LLE: Into<ll::LowLevelError>> From<LLE> for Opl2Error {
    fn from(low_level_error: LLE) -> Self {
        Opl2Error::LowLevelError(low_level_error.into())
    }
}

pub enum Note {
    C(u8),
    Cs(u8),
    D(u8),
    Eb(u8),
    E(u8),
    F(u8),
    Fs(u8),
    G(u8),
    Ab(u8),
    A(u8),
    Bb(u8),
    B(u8),
}

impl Note {
    pub fn get_frequency(&self) -> u16 {
        match self {
            Note::C(_) => 0x157,
            Note::Cs(_) => 0x16B,
            Note::D(_) => 0x181,
            Note::Eb(_) => 0x198,
            Note::E(_) => 0x1B0,
            Note::F(_) => 0x1CA,
            Note::Fs(_) => 0x1E5,
            Note::G(_) => 0x202,
            Note::Ab(_) => 0x220,
            Note::A(_) => 0x241,
            Note::Bb(_) => 0x263,
            Note::B(_) => 0x287,
        }
    }

    pub fn get_octave(&self) -> u8 {
        match self {
            Note::C(o) => *o,
            Note::Cs(o) => *o,
            Note::D(o) => *o,
            Note::Eb(o) => *o,
            Note::E(o) => *o,
            Note::F(o) => *o,
            Note::Fs(o) => *o,
            Note::G(o) => *o,
            Note::Ab(o) => *o,
            Note::A(o) => *o,
            Note::Bb(o) => *o,
            Note::B(o) => *o,
        }
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

    fn set_operator_settings(
        &mut self,
        channel: usize,
        operator: usize,
        settings: OperatorSettings,
    ) -> Result<(), Opl2Error> {
        let operator = match (operator, Self::OPERATOR_MAP[channel]) {
            (0, (operator, _)) => operator,
            (1, (_, operator)) => operator,
            _ => unreachable!(),
        };

        self.ll()
            .operator_settings0()
            .write_index(operator, |_| settings.operator_settings0)?;
        self.ll()
            .operator_settings1()
            .write_index(operator, |_| settings.operator_settings1)?;
        self.ll()
            .operator_settings2()
            .write_index(operator, |_| settings.operator_settings2)?;
        self.ll()
            .operator_settings3()
            .write_index(operator, |_| settings.operator_settings3)?;
        self.ll()
            .operator_settings4()
            .write_index(operator, |_| settings.operator_settings4)?;

        Ok(())
    }

    pub fn setup_melody_instrument(
        &mut self,
        channel: usize,
        value: MelodyInstrument,
    ) -> Result<(), Opl2Error> {
        if channel > INIT::CHANNEL_COUNT {
            return Err(Opl2Error::InvalidChannel);
        }

        self.set_operator_settings(channel, 0, value.operator_0)?;
        self.ll()
            .channel_settings2()
            .write_index(channel, |_| value.channel_settings2)?;
        self.set_operator_settings(channel, 1, value.operator_1)?;

        Ok(())
    }

    pub fn start_channel(&mut self, channel: usize, note: Note) -> Result<(), Opl2Error> {
        if channel > INIT::CHANNEL_COUNT {
            return Err(Opl2Error::InvalidChannel);
        }

        let frequency = note.get_frequency();
        let octave = note.get_octave();

        self.ll().channel_settings0().write_index(channel, |w| {
            w.frequency_number_low((frequency & 0xFF) as u8)
        })?;
        self.ll().channel_settings1().write_index(channel, |w| {
            w.frequency_number_high(((frequency & 0x300) >> 8) as u8)
                .block_number(octave)
                .key_on(Bit::Set)
        })?;

        Ok(())
    }

    pub fn stop_channel(&mut self, channel: usize) -> Result<(), Opl2Error> {
        if channel > INIT::CHANNEL_COUNT {
            return Err(Opl2Error::InvalidChannel);
        }

        self.ll()
            .channel_settings1()
            .write_index(channel, |w| w.key_on(Bit::Cleared))?;

        Ok(())
    }
}

impl<I: ll::HardwareInterface> Opl2<I, Melody> {
    pub fn into_rhythm_mode(mut self) -> Result<Opl2<I, Rhythm>, Opl2Error> {
        // KEY-ON registers for channels 06, 07, and 08 must be OFF in order to use the rhythm section.
        for i in 6..=8 {
            self.ll()
                .channel_settings1()
                .modify_index(i, |_, w| w.key_on(Bit::Cleared))?;
        }

        self.ll()
            .rhythm_settings()
            .modify(|_, w| w.instrument_mode(InstrumentMode::Percussion))?;

        Ok(Opl2 {
            ll: self.ll,
            phantom: PhantomData::default(),
        })
    }
}

impl<I: ll::HardwareInterface> Opl2<I, Rhythm> {
    pub fn into_melody_mode(mut self) -> Result<Opl2<I, Melody>, Opl2Error> {
        self.ll()
            .rhythm_settings()
            .modify(|_, w| w.instrument_mode(InstrumentMode::Melodic))?;
        Ok(Opl2 {
            ll: self.ll,
            phantom: PhantomData::default(),
        })
    }

    pub fn bass_drum(&mut self, value: bool) -> Result<(), Opl2Error> {
        self.ll()
            .rhythm_settings()
            .modify(|_, w| w.bass_drum_on(value.into()))?;
        Ok(())
    }

    pub fn setup_bass_drum(&mut self, value: BassDrum) -> Result<(), Opl2Error> {
        self.set_operator_settings(BassDrum::CHANNEL, 0, value.operator_0)?;
        self.ll()
            .channel_settings2()
            .write_index(BassDrum::CHANNEL, |_| value.channel_settings2)?;
        self.set_operator_settings(BassDrum::CHANNEL, 1, value.operator_1)?;

        Ok(())
    }

    pub fn snare_drum(&mut self, value: bool) -> Result<(), Opl2Error> {
        self.ll()
            .rhythm_settings()
            .modify(|_, w| w.snare_drum_on(value.into()))?;
        Ok(())
    }

    pub fn setup_snare_drum(&mut self, value: SnareDrum) -> Result<(), Opl2Error> {
        self.set_operator_settings(SnareDrum::CHANNEL, SnareDrum::OPERATOR, value.operator)?;

        Ok(())
    }

    pub fn tom_tom(&mut self, value: bool) -> Result<(), Opl2Error> {
        self.ll()
            .rhythm_settings()
            .modify(|_, w| w.tom_tom_on(value.into()))?;
        Ok(())
    }

    pub fn setup_tom_tom(&mut self, value: TomTom) -> Result<(), Opl2Error> {
        self.set_operator_settings(TomTom::CHANNEL, TomTom::OPERATOR, value.operator)?;

        Ok(())
    }

    pub fn cymbal(&mut self, value: bool) -> Result<(), Opl2Error> {
        self.ll()
            .rhythm_settings()
            .modify(|_, w| w.cymbal_on(value.into()))?;
        Ok(())
    }

    pub fn setup_cymbal(&mut self, value: Cymbal) -> Result<(), Opl2Error> {
        self.set_operator_settings(Cymbal::CHANNEL, Cymbal::OPERATOR, value.operator)?;

        Ok(())
    }

    pub fn hi_hat(&mut self, value: bool) -> Result<(), Opl2Error> {
        self.ll()
            .rhythm_settings()
            .modify(|_, w| w.hi_hat_on(value.into()))?;
        Ok(())
    }

    pub fn setup_hi_hat(&mut self, value: HiHat) -> Result<(), Opl2Error> {
        self.set_operator_settings(HiHat::CHANNEL, HiHat::OPERATOR, value.operator)?;

        Ok(())
    }
}
