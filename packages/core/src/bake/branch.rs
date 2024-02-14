use super::Encoder;
use bytes::BytesMut;

macro_rules! impl_encoder_for_tuple {
    ($($idx:literal),+) => {
        paste::paste! {
            impl<$([<E $idx>]),+> Encoder for ($([<E $idx>]),+,)
            where
            $(
                [<E $idx>]: Encoder,
            )+
            {
                fn write(&self, output: &mut BytesMut) {
                    $(
                        self.$idx.write(output);
                    )+
                }
            }
        }
    };
}

impl_encoder_for_tuple!(0);
impl_encoder_for_tuple!(0, 1);
impl_encoder_for_tuple!(0, 1, 2);
impl_encoder_for_tuple!(0, 1, 2, 3);
impl_encoder_for_tuple!(0, 1, 2, 3, 4);
impl_encoder_for_tuple!(0, 1, 2, 3, 4, 5);
impl_encoder_for_tuple!(0, 1, 2, 3, 4, 5, 6);
impl_encoder_for_tuple!(0, 1, 2, 3, 4, 5, 6, 7);
impl_encoder_for_tuple!(0, 1, 2, 3, 4, 5, 6, 7, 8);
impl_encoder_for_tuple!(0, 1, 2, 3, 4, 5, 6, 7, 8, 9);
impl_encoder_for_tuple!(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10);
impl_encoder_for_tuple!(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11);
impl_encoder_for_tuple!(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12);
impl_encoder_for_tuple!(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13);
impl_encoder_for_tuple!(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14);
impl_encoder_for_tuple!(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15);
impl_encoder_for_tuple!(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16);
