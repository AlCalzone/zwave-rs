use bytes::Bytes;
use cookie_factory as cf;
use custom_debug_derive::Debug;
use zwave_core::encoding::{encoders::empty, Serializable};
use zwave_core::munch::{
    bytes::{be_u8, rest},
    combinators::map,
};
use zwave_core::prelude::*;

#[derive(Debug, Clone, PartialEq)]
pub struct CCRaw {
    pub cc_id: CommandClasses,
    pub cc_command: Option<u8>,
    // #[debug(with = "hex_fmt")]
    pub payload: Bytes,
}

impl Parsable for CCRaw {
    fn parse(i: &mut Bytes) -> zwave_core::munch::ParseResult<Self> {
        let cc_id = CommandClasses::parse(i)?;

        // All CCs except NoOperation have a CC command
        let cc_command = match cc_id {
            CommandClasses::NoOperation => None,
            _ => map(be_u8, Some).parse(i)?,
        };
        let payload = rest(i)?;

        Ok(Self {
            cc_id,
            cc_command,
            payload,
        })
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
            slice(&self.payload),
        ))
    }
}
