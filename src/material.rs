//! Material definitions, physical phases, thermal properties, and color palettes.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Phase {
    Void = 0,
    Powder = 1,
    Liquid = 2,
    Gas = 3,
    Solid = 4,
    Energy = 5,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Material {
    Empty = 0,
    Sand = 1,
    Gunpowder = 2,
    Virus = 3,
    Oil = 4,
    Water = 5,
    Saltwater = 6,
    Acid = 7,
    Nitro = 8,
    Lava = 9,
    Steam = 10,
    Smoke = 11,
    Methane = 12,
    ToxicGas = 13,
    Ice = 14,
    Wood = 15,
    Plant = 16,
    Stone = 17,
    Obsidian = 18,
    Metal = 19,
    Fire = 20,
    Spark = 21,
}

pub const MATERIAL_COUNT: usize = 22;

#[derive(Clone, Copy, Debug)]
pub struct MaterialProps {
    pub phase: Phase,
    pub density: i16,
    pub dispersion: u8,
    pub conductivity: f32,
    pub default_temp: f32,
    pub base_color: [u8; 3],
    pub color_jitter: u8,
    pub flammable: bool,
    pub ignition_temp: f32,
    /// 0 = acid-proof, higher = faster dissolution in Acid (out of 255).
    pub acid_dissolve: u8,
    pub conductive: bool,
}

impl Material {
    #[inline]
    pub const fn from_u8(id: u8) -> Self {
        match id {
            0 => Self::Empty,
            1 => Self::Sand,
            2 => Self::Gunpowder,
            3 => Self::Virus,
            4 => Self::Oil,
            5 => Self::Water,
            6 => Self::Saltwater,
            7 => Self::Acid,
            8 => Self::Nitro,
            9 => Self::Lava,
            10 => Self::Steam,
            11 => Self::Smoke,
            12 => Self::Methane,
            13 => Self::ToxicGas,
            14 => Self::Ice,
            15 => Self::Wood,
            16 => Self::Plant,
            17 => Self::Stone,
            18 => Self::Obsidian,
            19 => Self::Metal,
            20 => Self::Fire,
            21 => Self::Spark,
            _ => Self::Empty,
        }
    }

    #[inline]
    pub const fn props(self) -> &'static MaterialProps {
        &PROPS[self as usize]
    }

    #[inline]
    pub const fn phase(self) -> Phase {
        self.props().phase
    }

    #[inline]
    pub const fn density(self) -> i16 {
        self.props().density
    }
}

const PROPS: [MaterialProps; MATERIAL_COUNT] = [
    // 0: Empty
    MaterialProps {
        phase: Phase::Void,
        density: 0,
        dispersion: 0,
        conductivity: 0.004,
        default_temp: 22.0,
        base_color: [11, 14, 20],
        color_jitter: 0,
        flammable: false,
        ignition_temp: 9999.0,
        acid_dissolve: 0,
        conductive: false,
    },
    // 1: Sand
    MaterialProps {
        phase: Phase::Powder,
        density: 180,
        dispersion: 0,
        conductivity: 0.08,
        default_temp: 22.0,
        base_color: [216, 186, 114],
        color_jitter: 22,
        flammable: false,
        ignition_temp: 9999.0,
        acid_dissolve: 0,
        conductive: false,
    },
    // 2: Gunpowder
    MaterialProps {
        phase: Phase::Powder,
        density: 150,
        dispersion: 0,
        conductivity: 0.06,
        default_temp: 22.0,
        base_color: [68, 66, 74],
        color_jitter: 16,
        flammable: true,
        ignition_temp: 220.0,
        acid_dissolve: 60,
        conductive: false,
    },
    // 3: Virus (biological infectious spore)
    MaterialProps {
        phase: Phase::Powder,
        density: 130,
        dispersion: 0,
        conductivity: 0.08,
        default_temp: 28.0,
        base_color: [168, 56, 214],
        color_jitter: 28,
        flammable: true,
        ignition_temp: 120.0,
        acid_dissolve: 140,
        conductive: false,
    },
    // 4: Oil
    MaterialProps {
        phase: Phase::Liquid,
        density: 75,
        dispersion: 4,
        conductivity: 0.10,
        default_temp: 22.0,
        base_color: [110, 74, 34],
        color_jitter: 14,
        flammable: true,
        ignition_temp: 240.0,
        acid_dissolve: 0,
        conductive: false,
    },
    // 5: Water
    MaterialProps {
        phase: Phase::Liquid,
        density: 100,
        dispersion: 5,
        conductivity: 0.22,
        default_temp: 20.0,
        base_color: [42, 134, 232],
        color_jitter: 14,
        flammable: false,
        ignition_temp: 9999.0,
        acid_dissolve: 0,
        conductive: true,
    },
    // 6: Saltwater
    MaterialProps {
        phase: Phase::Liquid,
        density: 114,
        dispersion: 5,
        conductivity: 0.24,
        default_temp: 18.0,
        base_color: [62, 168, 212],
        color_jitter: 14,
        flammable: false,
        ignition_temp: 9999.0,
        acid_dissolve: 0,
        conductive: true,
    },
    // 7: Acid
    MaterialProps {
        phase: Phase::Liquid,
        density: 124,
        dispersion: 4,
        conductivity: 0.20,
        default_temp: 26.0,
        base_color: [116, 242, 48],
        color_jitter: 18,
        flammable: false,
        ignition_temp: 9999.0,
        acid_dissolve: 0,
        conductive: true,
    },
    // 8: Nitro
    MaterialProps {
        phase: Phase::Liquid,
        density: 138,
        dispersion: 3,
        conductivity: 0.14,
        default_temp: 22.0,
        base_color: [228, 72, 108],
        color_jitter: 16,
        flammable: true,
        ignition_temp: 110.0,
        acid_dissolve: 0,
        conductive: false,
    },
    // 9: Lava
    MaterialProps {
        phase: Phase::Liquid,
        density: 220,
        dispersion: 2,
        conductivity: 0.28,
        default_temp: 1150.0,
        base_color: [255, 102, 18],
        color_jitter: 28,
        flammable: false,
        ignition_temp: 9999.0,
        acid_dissolve: 0,
        conductive: false,
    },
    // 10: Steam
    MaterialProps {
        phase: Phase::Gas,
        density: -20,
        dispersion: 3,
        conductivity: 0.14,
        default_temp: 165.0,
        base_color: [208, 226, 242],
        color_jitter: 16,
        flammable: false,
        ignition_temp: 9999.0,
        acid_dissolve: 0,
        conductive: false,
    },
    // 11: Smoke
    MaterialProps {
        phase: Phase::Gas,
        density: -14,
        dispersion: 3,
        conductivity: 0.06,
        default_temp: 120.0,
        base_color: [88, 92, 102],
        color_jitter: 18,
        flammable: false,
        ignition_temp: 9999.0,
        acid_dissolve: 0,
        conductive: false,
    },
    // 12: Methane
    MaterialProps {
        phase: Phase::Gas,
        density: -28,
        dispersion: 4,
        conductivity: 0.06,
        default_temp: 22.0,
        base_color: [176, 134, 212],
        color_jitter: 16,
        flammable: true,
        ignition_temp: 200.0,
        acid_dissolve: 0,
        conductive: false,
    },
    // 13: ToxicGas
    MaterialProps {
        phase: Phase::Gas,
        density: -10,
        dispersion: 3,
        conductivity: 0.07,
        default_temp: 38.0,
        base_color: [148, 214, 72],
        color_jitter: 20,
        flammable: false,
        ignition_temp: 9999.0,
        acid_dissolve: 0,
        conductive: false,
    },
    // 14: Ice
    MaterialProps {
        phase: Phase::Solid,
        density: 900,
        dispersion: 0,
        conductivity: 0.24,
        default_temp: -20.0,
        base_color: [152, 220, 248],
        color_jitter: 14,
        flammable: false,
        ignition_temp: 9999.0,
        acid_dissolve: 45,
        conductive: false,
    },
    // 15: Wood
    MaterialProps {
        phase: Phase::Solid,
        density: 1000,
        dispersion: 0,
        conductivity: 0.06,
        default_temp: 22.0,
        base_color: [128, 80, 44],
        color_jitter: 16,
        flammable: true,
        ignition_temp: 260.0,
        acid_dissolve: 70,
        conductive: false,
    },
    // 16: Plant
    MaterialProps {
        phase: Phase::Solid,
        density: 1000,
        dispersion: 0,
        conductivity: 0.07,
        default_temp: 22.0,
        base_color: [56, 174, 76],
        color_jitter: 24,
        flammable: true,
        ignition_temp: 180.0,
        acid_dissolve: 110,
        conductive: false,
    },
    // 17: Stone
    MaterialProps {
        phase: Phase::Solid,
        density: 1000,
        dispersion: 0,
        conductivity: 0.14,
        default_temp: 22.0,
        base_color: [114, 118, 128],
        color_jitter: 16,
        flammable: false,
        ignition_temp: 9999.0,
        acid_dissolve: 18,
        conductive: false,
    },
    // 18: Obsidian
    MaterialProps {
        phase: Phase::Solid,
        density: 1000,
        dispersion: 0,
        conductivity: 0.09,
        default_temp: 45.0,
        base_color: [44, 34, 62],
        color_jitter: 12,
        flammable: false,
        ignition_temp: 9999.0,
        acid_dissolve: 0,
        conductive: false,
    },
    // 19: Metal (electrical conductor)
    MaterialProps {
        phase: Phase::Solid,
        density: 1000,
        dispersion: 0,
        conductivity: 0.42,
        default_temp: 22.0,
        base_color: [164, 174, 188],
        color_jitter: 10,
        flammable: false,
        ignition_temp: 9999.0,
        acid_dissolve: 28,
        conductive: true,
    },
    // 20: Fire
    MaterialProps {
        phase: Phase::Energy,
        density: -30,
        dispersion: 1,
        conductivity: 0.18,
        default_temp: 650.0,
        base_color: [255, 148, 28],
        color_jitter: 36,
        flammable: false,
        ignition_temp: 9999.0,
        acid_dissolve: 0,
        conductive: false,
    },
    // 21: Spark
    MaterialProps {
        phase: Phase::Energy,
        density: 0,
        dispersion: 0,
        conductivity: 0.15,
        default_temp: 420.0,
        base_color: [130, 232, 255],
        color_jitter: 24,
        flammable: false,
        ignition_temp: 9999.0,
        acid_dissolve: 0,
        conductive: true,
    },
];
