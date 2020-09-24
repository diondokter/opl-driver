use crate::ll::registers::channel_settings2;
use crate::ll::registers::operator_settings0;
use crate::ll::registers::operator_settings1;
use crate::ll::registers::operator_settings2;
use crate::ll::registers::operator_settings3;
use crate::ll::registers::operator_settings4;

#[derive(Debug, Copy, Clone)]
pub struct OperatorSettings {
    pub operator_settings0: operator_settings0::W,
    pub operator_settings1: operator_settings1::W,
    pub operator_settings2: operator_settings2::W,
    pub operator_settings3: operator_settings3::W,
    pub operator_settings4: operator_settings4::W,
}

impl OperatorSettings {
    pub const fn new(
        operator_settings0: operator_settings0::W,
        operator_settings1: operator_settings1::W,
        operator_settings2: operator_settings2::W,
        operator_settings3: operator_settings3::W,
        operator_settings4: operator_settings4::W,
    ) -> Self {
        Self {
            operator_settings0,
            operator_settings1,
            operator_settings2,
            operator_settings3,
            operator_settings4,
        }
    }

    pub const fn from_bytes(bytes: [u8; 5]) -> Self {
        Self::new(
            operator_settings0::W::from_raw([bytes[0]]),
            operator_settings1::W::from_raw([bytes[1]]),
            operator_settings2::W::from_raw([bytes[2]]),
            operator_settings3::W::from_raw([bytes[3]]),
            operator_settings4::W::from_raw([bytes[4]]),
        )
    }
}

#[derive(Debug, Copy, Clone)]
pub struct MelodyInstrument {
    pub operator_0: OperatorSettings,
    pub channel_settings2: channel_settings2::W,
    pub operator_1: OperatorSettings,
}
impl MelodyInstrument {
    pub const fn new(
        operator_0: OperatorSettings,
        channel_settings2: channel_settings2::W,
        operator_1: OperatorSettings,
    ) -> Self {
        Self {
            operator_0,
            channel_settings2,
            operator_1,
        }
    }

    pub const fn from_bytes(bytes: [u8; 11]) -> Self {
        Self::new(
            OperatorSettings::from_bytes([bytes[0], bytes[1], bytes[2], bytes[3], bytes[4]]),
            channel_settings2::W::from_raw([bytes[5]]),
            OperatorSettings::from_bytes([bytes[6], bytes[7], bytes[8], bytes[9], bytes[10]]),
        )
    }
}

#[derive(Debug, Copy, Clone)]
pub struct BassDrum {
    pub operator_0: OperatorSettings,
    pub channel_settings2: channel_settings2::W,
    pub operator_1: OperatorSettings,
}
impl BassDrum {
    pub const CHANNEL: usize = 6;

    pub const fn new(
        operator_0: OperatorSettings,
        channel_settings2: channel_settings2::W,
        operator_1: OperatorSettings,
    ) -> Self {
        Self {
            operator_0,
            channel_settings2,
            operator_1,
        }
    }

    pub const fn from_bytes(bytes: [u8; 11]) -> Self {
        Self::new(
            OperatorSettings::from_bytes([bytes[0], bytes[1], bytes[2], bytes[3], bytes[4]]),
            channel_settings2::W::from_raw([bytes[5]]),
            OperatorSettings::from_bytes([bytes[6], bytes[7], bytes[8], bytes[9], bytes[10]]),
        )
    }
}

#[derive(Debug, Copy, Clone)]
pub struct SnareDrum {
    pub operator: OperatorSettings,
}
impl SnareDrum {
    pub const CHANNEL: usize = 7;
    pub const OPERATOR: usize = 1;

    pub const fn new(operator: OperatorSettings) -> Self {
        Self { operator }
    }

    pub const fn from_bytes(bytes: [u8; 5]) -> Self {
        Self::new(OperatorSettings::from_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4],
        ]))
    }
}

#[derive(Debug, Copy, Clone)]
pub struct TomTom {
    pub operator: OperatorSettings,
}
impl TomTom {
    pub const CHANNEL: usize = 8;
    pub const OPERATOR: usize = 0;

    pub const fn new(operator: OperatorSettings) -> Self {
        Self { operator }
    }

    pub const fn from_bytes(bytes: [u8; 5]) -> Self {
        Self::new(OperatorSettings::from_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4],
        ]))
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Cymbal {
    pub operator: OperatorSettings,
}
impl Cymbal {
    pub const CHANNEL: usize = 8;
    pub const OPERATOR: usize = 1;

    pub const fn new(operator: OperatorSettings) -> Self {
        Self { operator }
    }

    pub const fn from_bytes(bytes: [u8; 5]) -> Self {
        Self::new(OperatorSettings::from_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4],
        ]))
    }
}

#[derive(Debug, Copy, Clone)]
pub struct HiHat {
    pub operator: OperatorSettings,
}
impl HiHat {
    pub const CHANNEL: usize = 7;
    pub const OPERATOR: usize = 0;

    pub const fn new(operator: OperatorSettings) -> Self {
        Self { operator }
    }

    pub const fn from_bytes(bytes: [u8; 5]) -> Self {
        Self::new(OperatorSettings::from_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4],
        ]))
    }
}

pub mod presets {
    use super::*;

    pub const ELPIANO1: MelodyInstrument = MelodyInstrument::from_bytes([
        0x01, 0x4F, 0xF1, 0x50, 0x00, 0x06, 0x01, 0x04, 0xD2, 0x7C, 0x00
    ]);
    pub const GUITAR1: MelodyInstrument = MelodyInstrument::from_bytes([
        0x01, 0x11, 0xF2, 0x1F, 0x00, 0x0A, 0x01, 0x00, 0xF5, 0x88, 0x00
    ]);
    pub const STRINGS1: MelodyInstrument = MelodyInstrument::from_bytes([
        0xB1, 0x8B, 0x71, 0x11, 0x00, 0x06, 0x61, 0x40, 0x42, 0x15, 0x01
    ]);

    pub mod drums {
        use super::*;

        pub const BDRUM1: BassDrum = BassDrum::from_bytes([
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0B, 0xA8, 0x4C, 0x00
        ]);
        pub const CYMBAL1: Cymbal = Cymbal::from_bytes([
            0x01, 0x00, 0xF5, 0xB5, 0x00
        ]);
        pub const HIHAT1: HiHat = HiHat::from_bytes([
            0x01, 0x00, 0xF7, 0xB5, 0x00
        ]);
        pub const HIHAT2: HiHat = HiHat::from_bytes([
            0x01, 0x03, 0xDA, 0x18, 0x00
        ]);
        pub const LASER: Cymbal = Cymbal::from_bytes([
            0xE6, 0x00, 0x25, 0xB5, 0x00
        ]);
        pub const MLTRDRUM: SnareDrum = SnareDrum::from_bytes([
            0x0C, 0x00, 0xC8, 0xB6, 0x01
        ]);
        pub const RKSNARE: SnareDrum = SnareDrum::from_bytes([
            0x0C, 0x00, 0xC7, 0xB4, 0x00
        ]);
        pub const SNARE1: SnareDrum = SnareDrum::from_bytes([
            0x0C, 0x00, 0xF8, 0xB5, 0x00
        ]);
        pub const TOM1: TomTom = TomTom::from_bytes([
            0x04, 0x00, 0xF7, 0xB5, 0x00
        ]);
        pub const TOM2: TomTom = TomTom::from_bytes([
            0x02, 0x00, 0xC8, 0x97, 0x00
        ]);
        pub const XYLO2: BassDrum = BassDrum::from_bytes([
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x2E, 0x00, 0xFF, 0x0F, 0x00
        ]);
    }
}
