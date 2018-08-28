//! traveller generates worlds for trevaller

#[macro_use]
extern crate bitflags;
extern crate rand;

use rand::Rng;
use std::{fmt, cmp};

fn safer_add(a: u8, b: i8) -> u8 {
    cmp::max(a as i8 + b, 0) as u8
    // if - b > a as i8 {
    //     0
    // } else {
    //     (a as i8 + b) as u8
    // }
}

/// roll a standard d3
fn rolld3() -> u8 {
    rand::thread_rng().gen_range(1, 4)
}

fn rolld3dm(dm: i8) -> u8 {
    safer_add(rolld3(), dm)
}

/// roll a standard d6
fn rolld6() -> u8 {
    rand::thread_rng().gen_range(1, 7)
}

fn rolld6dm(dm: i8) -> u8 {
    safer_add(rolld6(), dm)
}

/// roll some number of d6 and get the result
fn rollnd6(n: u8) -> u8 {
    let mut total = 0;
    for _ in 0..n {
        total += rolld6();
    }
    total
}

fn rollnd6dm(n: u8, dm: i8) -> u8 {
    safer_add(rollnd6(n), dm)
}

/// roll one d6 as the tens place and one d6 as the ones place. some number
/// can't be represented with this, meaning there are essentially only 36
/// outcomes, not 66.
// fn rolld66() -> u8 {
//     rolld6() * 10 + rolld6()
// }

#[derive(Clone, Debug, Default)]
struct Subsector {
    grid: Vec<Vec<Option<World>>>,
}

#[derive(Clone, Debug)]
enum Density {
    Rift,
    Sparse,
    Spiral,
    Dense,
}

impl Subsector {
    fn generate(density: Density) -> Subsector {
        // a subsector is an 8x10 grid of possible world locations. the
        // likelyhood of a hex containing a world starts at 50/50 and is
        // modified by the region of the galaxy it's in. the region is a
        // configuration parameter.
        let dm = match density {
            Density::Rift => -2,
            Density::Sparse => -1,
            Density::Spiral => 0,
            Density::Dense => 1,
        };

        let mut subsector = Subsector{grid: vec![]};
        for _ in 0..8 {
            let mut col = vec![];
            for _ in 0..10 {
                if rolld6() as i16 + dm >= 4 {
                    col.push(Some(World::generate()));
                } else {
                    col.push(None);
                }
            }
            subsector.grid.push(col);
        }
        subsector
    }
}

#[derive(Clone, Debug)]
struct World {
    size: Size,
    atmosphere: Atmosphere,
    temperature: Temperature,
    starport: Starport,
    hydrographics: Hydrographics,
    population: Population,
    government: Government,
    factions: Vec<Faction>,
    law: Law,
    tech: Tech,
    bases: Bases,
    codes: Codes,
    zone: Zone,
}

impl fmt::Display for World {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}{}{}{}{}{}-{} {} {} {}",
               self.starport,
               self.size,
               self.atmosphere,
               self.hydrographics,
               self.population,
               self.government,
               self.law,
               self.tech,
               self.bases,
               self.codes,
               self.zone
        )
    }
}

impl World {
    fn generate() -> World {
        // size is 2d6 - 2
        let size = rollnd6dm(2, -2);

        // atmosphere is 2d6 + size - 7, min 0
        let atmosphere = rollnd6dm(2, size as i8 - 7);

        // temperature has a dm based on atmosphere
        let dm = match atmosphere {
            0  | 1       =>  0,
            2  | 3       => -2,
            4  | 5  | 14 => -1,
            6  | 7       =>  0,
            8  | 9       =>  1,
            10 | 13 | 15 =>  2,
            11 | 12      =>  6,
            _            =>  0,
        };
        let temperature = match rollnd6dm(2, dm) {
            0 | 1 | 2 => Temperature::Frozen,
            3 | 4 => Temperature::Cold,
            5 | 6 | 7 | 8 | 9 => Temperature::Temperate,
            10 | 11 => Temperature::Hot,
            _ => Temperature::Roasting,
        };

        // hydrographics percentage
        let hydrographics = if size == 1 || size == 1 {
            0
        } else {
            let mut dm = match atmosphere {
                0 | 1 | 10 | 11 | 12 => -4,
                _ => 0,
            };
            dm += match temperature {
                Temperature::Hot => -2,
                Temperature::Roasting => -6,
                _ => 0,
            };
            dm += size as i8;
            dm += -7;
            rollnd6dm(2, dm)
        };

        // population is 2d6 - 2
        let population = rollnd6dm(2, -2);

        // government type is 2d6 - 7 + pop
        let government = cmp::min(rollnd6dm(2, population as i8 - 7), 13);

        // factions
        let dm = match government {
            0 | 7 => 1,
            10 | 11 | 12 | 13 => -1,
            _ => 0,
        };
        let mut factions = vec![];
        for _ in 0..(rolld3dm(dm)) {
            // the rule book doesn't specify how to determine the government
            // type of a faction. however, the "strength" attribute of a faction
            // seems like a good approximation of the "population" of a faction.
            let strength = rollnd6(2);
            factions.push(Faction {
                government: cmp::min(rollnd6dm(2, strength as i8 - 7), 13),
                strength: match strength {
                    2  | 3  => FactionStrength::Obscure,
                    4  | 5  => FactionStrength::Fringe,
                    6  | 7  => FactionStrength::Minor,
                    8  | 9  => FactionStrength::Notable,
                    10 | 11 => FactionStrength::Significant,
                    12      => FactionStrength::Overwhelming,
                    // how did we end up with this if we rolled 2d6?
                    _       => FactionStrength::Obscure,
                },
            })
        }

        // the generator skips cultural differences, since it's recommended that
        // they be a combination of random and reasoned extrapolation. that
        // means TODO: print out a reminder to manually generate cultural
        // differences.

        // law level is 2d6 - 7 + government with a maximum of 9
        let law = cmp::min(rollnd6dm(2, government as i8 - 7), 9);

        // starports are rated A-E, or X if there is none, determined with a 2d6
        // roll and converted with a table.
        let class = match rollnd6(2) {
            2 => StarportClass::X,
            3 | 4 => StarportClass::E,
            5 | 6 => StarportClass::D,
            7 | 8 => StarportClass::C,
            9 | 10 => StarportClass::B,
            11 | 12 => StarportClass::A,
            _ => StarportClass::X,
        };
        let starport = Starport {
            class,
            berthing: match class {
                StarportClass::A => 1000 * rolld6() as u32,
                StarportClass::B => 500  * rolld6() as u32,
                StarportClass::C => 100  * rolld6() as u32,
                StarportClass::D => 10   * rolld6() as u32,
                StarportClass::E => 0,
                StarportClass::X => 0,
            },
        };

        // bases in the system are determined by the class of the starport and
        // a roll against a value,
        let mut bases = Bases::default();
        macro_rules! base {
            ( $base:path, $dc:expr ) => {
                if rollnd6(2) >= $dc {
                    bases |= $base;
                }
            };
        }
        match class {
            StarportClass::A => {
                base!(Bases::NAVAL, 8);
                base!(Bases::SCOUT, 10);
                base!(Bases::RESEARCH, 8);
                base!(Bases::TAS, 4);
                base!(Bases::CONSULATE, 6);
            },
            StarportClass::B => {
                base!(Bases::NAVAL, 8);
                base!(Bases::SCOUT, 8);
                base!(Bases::RESEARCH, 10);
                base!(Bases::TAS, 6);
                base!(Bases::CONSULATE, 8);
                base!(Bases::PIRATE, 12);
            },
            StarportClass::C => {
                base!(Bases::SCOUT, 8);
                base!(Bases::RESEARCH, 10);
                base!(Bases::TAS, 10);
                base!(Bases::CONSULATE, 10);
                base!(Bases::PIRATE, 10);
            },
            StarportClass::D => {
                base!(Bases::SCOUT, 7);
                base!(Bases::PIRATE, 12);
            },
            StarportClass::E => {
                base!(Bases::PIRATE, 12);
            },
            StarportClass::X => {},
        }

        // tech level is 1d6, plus a complex system of dms based on starport
        // class, size, atmosphere, hydrographics, population, and government.
        // there is also a possibility that the tech level doesn't meet the
        // minimum requirements for the atmospheric conditions on the planet, in
        // which case the planet is likely to die out. this is shown with a
        // warning when outputing the planets.
        let mut dm = 0;
        // starport class
        dm += match class {
            StarportClass::A => 6,
            StarportClass::B => 4,
            StarportClass::C => 2,
            StarportClass::D => 0,
            StarportClass::E => 0,
            StarportClass::X => -4,
        };
        // planet size
        dm += match size {
            0 | 1 => 2,
            2 | 3 | 4 => 1,
            _ => 0,
        };
        // atmosphere
        dm += match atmosphere {
            0 | 1 | 2 | 3 | 10 | 11 | 12 | 13 | 14 | 15 => 1,
            _ => 0,
        };
        // hydrographics
        dm += match hydrographics {
            0 | 9 => 1,
            10 => 2,
            _ => 0,
        };
        // population
        dm += match population {
            1 | 2 | 3 | 4 | 5 | 9 => 1,
            10 => 2,
            11 => 3,
            12 => 4,
            _ => 0,
        };
        // government
        dm += match government {
            0 | 5 => 1,
            7 => 2,
            13 | 14 => -2,
            _ => 0,
        };
        // tech levels have a minimum of 0, but no theoretical maximum. the
        // descriptions of tech levels only go up to 15, but there are some
        // weapons with higher tl ratings.
        let tech = cmp::max(0, rolld6dm(dm));

        // travel codes are mostly green, or amber with some specific
        // circumstances. red codes are at referee discretion and are not
        // generated by us. amber zone planets should be audited for their
        // status by the referee.
        let zone = if atmosphere >= 10 || government == 0 || government == 7
            || government == 10 || law == 0 || law >= 9 {
            Zone::Amber
        } else {
            Zone::Green
        };

        // trade codes are given based on a variety of requirements. a planet
        // can have as many trade codes as it qualifies for.
        let mut codes = Codes::default();
        // code_help! generates the boolean expression evaluated by the if
        // statement in the main code! macro. this became a lot more complicated
        // than I originally intended it to. it was fun though!
        macro_rules! code_help {
            // base case for enumerating all possible values.
            // (variable; number) => variable == number
            (($stat:ident; $v:expr)) =>
                ($stat == $v);
            // case for lower bound only.
            // (lower bound, variable) => variable >= lower bound
            (($low:expr, $stat:ident)) =>
                ($low <= $stat);
            // case for upper bound only.
            // (variable, upper bound) => variable <= upper bound
            (($stat:ident, $high:expr)) =>
                ($stat <= $high);
            // case for both lower and upper bound. decomposes into one lower
            // and one upper bound call composed with &&.
            // (lower bound, variable, upper bound) =>
            //     code_help!((lower bound, variable)) &&
            //     code_help!((variable, upper bound))
            (($low:expr, $stat:ident, $high:expr)) =>
                (code_help!(($low, $stat)) && code_help!(($stat, $high)));
            // recursive case for value enumeration.
            // (variable; value, values...) =>
            //     code_help!((variable; value)) ||
            //     code_help!((variable; values...))
            (($stat:ident; $v:expr, $($vs:tt)*)) =>
                (code_help!(($stat; $v)) || code_help!(($stat; $($vs)*)));
            // recursive case for a list of requirements.
            // requirement, requirements... =>
            //     code_help!(requirement) && code_help!(requirements...)
            ($car:tt, $($cdr:tt)*) =>
                (code_help!($car) && code_help!($($cdr)*));
        }
        // code! generates the if statement that checks if one of the trade
        // codes apply to this world. it takes a code and a list of requirements
        // for that code to apply to this world, with the general syntax of
        // code!(CODE; (requirement), ...);. the requirements are parsed by
        // code_help!. requirements can be one:
        // lower bound: (low, variable)
        // upper bound: (variable, high)
        // both bounds: (low, variable, high)
        // enumeration: (variable; values...)
        macro_rules! code {
            ($code:path; $($reqs:tt)*) => {
                if code_help!($($reqs)*) {
                    codes |= $code;
                }
            }
        }

        code!(Codes::AGRICULTURAL; (4, atmosphere, 9)
                                 , (4, hydrographics, 8)
                                 , (5, population, 7));
        code!(Codes::ASTEROID; (size; 0)
                             , (atmosphere; 0)
                             , (hydrographics; 0));
        code!(Codes::BARREN; (population; 0)
                           , (government; 0)
                           , (law; 0));
        code!(Codes::DESERT; (2, atmosphere)
                           , (hydrographics; 0));
        code!(Codes::FLUID_OCEANS; (10, atmosphere)
                                 , (1, hydrographics));
        code!(Codes::GARDEN; (5, size)
                           , (4, atmosphere, 9)
                           , (4, hydrographics, 8));
        code!(Codes::HIGH_POP; (9, population));
        code!(Codes::HIGH_TECH; (12, tech));
        code!(Codes::ICE_CAPPED; (atmosphere, 1)
                               , (1, hydrographics));
        code!(Codes::INDUSTRIAL; (atmosphere; 0, 1, 2, 4, 7, 9)
                               , (9, population));
        code!(Codes::LOW_POP; (1, population, 3));
        code!(Codes::LOW_TECH; (tech, 5));
        code!(Codes::NON_AGRICULTURAL; (atmosphere, 3)
                                     , (hydrographics, 3)
                                     , (6, population));
        code!(Codes::NON_INDUSTRIAL; (4, population, 6));
        code!(Codes::POOR; (2, atmosphere, 5)
                         , (hydrographics, 3));
        code!(Codes::RICH; (atmosphere; 6, 8)
                         , (6, population, 8));
        code!(Codes::VACUUM; (atmosphere; 0));
        code!(Codes::WATER_WORLD; (hydrographics; 10));

        World {
            size,
            atmosphere,
            temperature,
            hydrographics,
            population,
            government,
            factions,
            law,
            tech,
            starport,
            bases,
            codes,
            zone,
        }
    }
}

type Size = u8;
type Atmosphere = u8;
type Hydrographics = u8;
type Population = u8;
type Government = u8;
type Law = u8;
type Tech = u8;
type BerthingCost = u32;

#[derive(Copy, Clone, Debug)]
struct Starport {
    class: StarportClass,
    berthing: BerthingCost,
}

impl fmt::Display for Starport {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.class)
    }
}

#[derive(Copy, Clone, Debug)]
enum StarportClass {
    A, // excellent
    B, // good
    C, // routine
    D, // poor
    E, // frontier
    X, // none
}

impl fmt::Display for StarportClass {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", match self {
            StarportClass::A => "A",
            StarportClass::B => "B",
            StarportClass::C => "C",
            StarportClass::D => "D",
            StarportClass::E => "E",
            StarportClass::X => "X",
        })
    }
}

#[derive(Copy, Clone, Debug)]
struct Faction {
    government: Government,
    strength: FactionStrength,
}

#[derive(Copy, Clone, Debug)]
enum FactionStrength {
    Obscure,
    Fringe,
    Minor,
    Notable,
    Significant,
    Overwhelming,
}

#[derive(Copy, Clone, Debug)]
enum Temperature {
    Frozen,
    Cold,
    Temperate,
    Hot,
    Roasting,
}

// lol
bitflags! {
    #[derive(Default)]
    struct Bases: u8 {
        const PIRATE    = 0b000001;
        const SCOUT     = 0b000010;
        const NAVAL     = 0b000100;
        const RESEARCH  = 0b001000;
        const CONSULATE = 0b010000;
        const TAS       = 0b100000;
    }
}

impl fmt::Display for Bases {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "TCRNSP")
    }
}

bitflags! {
    #[derive(Default)]
    struct Codes: u32 {
        const AGRICULTURAL     = 0b000000000000000001;
        const ASTEROID         = 0b000000000000000010;
        const BARREN           = 0b000000000000000100;
        const DESERT           = 0b000000000000001000;
        const FLUID_OCEANS     = 0b000000000000010000;
        const GARDEN           = 0b000000000000100000;
        const HIGH_POP         = 0b000000000001000000;
        const HIGH_TECH        = 0b000000000010000000;
        const ICE_CAPPED       = 0b000000000100000000;
        const INDUSTRIAL       = 0b000000001000000000;
        const LOW_POP          = 0b000000010000000000;
        const LOW_TECH         = 0b000000100000000000;
        const NON_AGRICULTURAL = 0b000001000000000000;
        const NON_INDUSTRIAL   = 0b000010000000000000;
        const POOR             = 0b000100000000000000;
        const RICH             = 0b001000000000000000;
        const VACUUM           = 0b010000000000000000;
        const WATER_WORLD      = 0b100000000000000000;
    }
}

impl fmt::Display for Codes {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Ag As Ba De Fl Ga Hi Ht IC In Lo Lt Na NI Po Ri Va Wa")
    }
}

#[derive(Copy, Clone, Debug)]
enum Zone {
    Unclassified,
    Green,
    Amber,
    Red,
}

impl Default for Zone {
    fn default() -> Zone {
        Zone::Unclassified
    }
}

impl fmt::Display for Zone {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", match self {
            Zone::Unclassified => " ",
            Zone::Green => " ",
            Zone::Amber => "A",
            Zone::Red => "R",
        })
    }
}

fn main() {
    for _ in 0..10 {
        println!("{:#?}", World::generate());
    }
}
