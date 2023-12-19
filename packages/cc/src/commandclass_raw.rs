use zwave_core::prelude::*;

use cookie_factory as cf;
use custom_debug_derive::Debug;
use nom::{
    combinator::{map, rest},
    number::complete::be_u8,
};
use zwave_core::{
    definitions::CommandClasses,
    encoding::{encoders::empty, Parsable, Serializable},
};

#[derive(Debug, Clone, PartialEq)]
pub struct CCRaw {
    pub cc_id: CommandClasses,
    pub cc_command: Option<u8>,
    // #[debug(with = "hex_fmt")]
    pub payload: Vec<u8>,
}

impl Parsable for CCRaw {
    fn parse(i: zwave_core::encoding::Input) -> ParseResult<Self> {
        let (i, cc_id) = CommandClasses::parse(i)?;

        // All CCs except NoOperation have a CC command
        let (i, cc_command) = match cc_id {
            CommandClasses::NoOperation => (i, None),
            _ => map(be_u8, Some)(i)?,
        };
        let (i, payload) = rest(i)?;

        Ok((
            i,
            Self {
                cc_id,
                cc_command,
                payload: payload.to_vec(),
            },
        ))
    }
}

impl Serializable for CCRaw {
    fn serialize<'a, W: std::io::Write + 'a>(&'a self) -> impl cookie_factory::SerializeFn<W> + 'a {
        use cf::{bytes::be_u8, combinator::slice, sequence::tuple};
        tuple((
            self.cc_id.serialize(),
            move |out| match self.cc_command {
                Some(cc_command) => be_u8(cc_command)(out),
                None => empty()(out),
            },
            slice(self.payload.as_slice()),
        ))
    }
}
