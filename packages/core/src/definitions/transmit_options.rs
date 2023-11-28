
use crate::encoding::{
    self, encoders, BitParsable, BitSerializable, Parsable, Serializable,
};

use cookie_factory as cf;
use nom::{
    bits, bits::complete::bool, sequence::tuple,
};
use ux::{u1, u2};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TransmitOptions {
    ack: bool,
    auto_route: bool,
    no_route: bool,
    explore: bool,
}

impl Default for TransmitOptions {
    fn default() -> Self {
        Self {
            ack: true,
            auto_route: true,
            no_route: false,
            explore: true,
        }
    }
}

// impl Debug for TransmitOptions {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         f.debug_struct("TransmitOptions")
//             .field("ack", &self.ack)
//             .field("auto_route", &self.auto_route)
//             .field("no_route", &self.no_route)
//             .field("explore", &self.explore)
//             .finish()
//     }
// }

impl TransmitOptions {
    pub fn new() -> Self {
        Self {
            ack: false,
            auto_route: false,
            no_route: false,
            explore: false,
        }
    }

    pub fn default_no_ack() -> Self {
        Self {
            ack: false,
            auto_route: true,
            no_route: false,
            explore: true,
        }
    }

    pub fn ack(mut self, ack: bool) -> Self {
        self.ack = ack;
        self
    }

    pub fn auto_route(mut self, auto_route: bool) -> Self {
        self.auto_route = auto_route;
        self
    }

    pub fn no_route(mut self, no_route: bool) -> Self {
        self.no_route = no_route;
        self
    }

    pub fn explore(mut self, explore: bool) -> Self {
        self.explore = explore;
        self
    }
}

impl Parsable for TransmitOptions {
    fn parse(i: encoding::Input) -> encoding::ParseResult<Self> {
        let (i, (_reserved76, explore, no_route, _reserved3, auto_route, _reserved1, ack)) = bits(
            tuple((u2::parse, bool, bool, u1::parse, bool, u1::parse, bool)),
        )(
            i
        )?;

        Ok((
            i,
            Self::new()
                .ack(ack)
                .auto_route(auto_route)
                .no_route(no_route)
                .explore(explore),
        ))
    }
}

impl Serializable for TransmitOptions {
    fn serialize<'a, W: std::io::Write + 'a>(&'a self) -> impl cf::SerializeFn<W> + 'a {
        move |out| {
            encoders::bits(move |bo| {
                let reserved76 = u2::new(0);
                let reserved3 = u1::new(0);
                let reserved1 = u1::new(0);
                reserved76.write(bo);
                self.explore.write(bo);
                self.no_route.write(bo);
                reserved3.write(bo);
                self.auto_route.write(bo);
                reserved1.write(bo);
                self.ack.write(bo);
            })(out)
        }
    }
}

#[test]
fn test_parse() {
    let raw = vec![0b1111_1111];
    let (_, opts) = TransmitOptions::parse(&raw).unwrap();
    let expected = TransmitOptions::new()
        .ack(true)
        .auto_route(true)
        .no_route(true)
        .explore(true);
    assert_eq!(opts, expected);
}

#[test]
fn test_serialize() {
    let opts = TransmitOptions::default();
    let raw = cookie_factory::gen_simple(opts.serialize(), Vec::new()).unwrap();
    assert_eq!(raw, vec![0b0010_0101]);

    let opts = TransmitOptions::new().ack(true);
    let raw = cookie_factory::gen_simple(opts.serialize(), Vec::new()).unwrap();
    assert_eq!(raw, vec![0b0000_0001]);
}
