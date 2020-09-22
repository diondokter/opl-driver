use crate::ll;
use core::marker::PhantomData;
use device_driver::ll::LowLevelDevice;

pub struct Uninitialized;
pub struct Melody;
pub struct Rythm;

pub trait Initialized {}

impl Initialized for Melody {}
impl Initialized for Rythm {}

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

impl<I: ll::HardwareInterface, INITIALIZED: Initialized> Opl2<I, INITIALIZED> {
    pub fn ll(&mut self) -> ll::registers::RegisterSet<I> {
        self.ll.registers()
    }
}
