use super::Encoder;
use bytes::BytesMut;

pub trait List {
    fn write_all(&self, output: &mut BytesMut);
}

pub fn tuple<L>(tuple: L) -> impl Encoder
where
    L: List
{
    move |output: &mut BytesMut| {
        tuple.write_all(output)
    }
}


macro_rules! impl_list_for_tuple {
    ($($idx:literal),+) => {
        paste::paste! {
            impl<$([<E $idx>]),+> List for ($([<E $idx>]),+,)
            where
            $(
                [<E $idx>]: Encoder,
            )+
            {
                fn write_all(&self, output: &mut BytesMut) {
                    $(
                        self.$idx.write(output);
                    )+
                }
            }
        }
    };
}

impl_list_for_tuple!(0);
impl_list_for_tuple!(0, 1);
impl_list_for_tuple!(0, 1, 2);
impl_list_for_tuple!(0, 1, 2, 3);
impl_list_for_tuple!(0, 1, 2, 3, 4);
impl_list_for_tuple!(0, 1, 2, 3, 4, 5);
impl_list_for_tuple!(0, 1, 2, 3, 4, 5, 6);
impl_list_for_tuple!(0, 1, 2, 3, 4, 5, 6, 7);
impl_list_for_tuple!(0, 1, 2, 3, 4, 5, 6, 7, 8);
impl_list_for_tuple!(0, 1, 2, 3, 4, 5, 6, 7, 8, 9);
impl_list_for_tuple!(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10);
impl_list_for_tuple!(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11);
impl_list_for_tuple!(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12);
impl_list_for_tuple!(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13);
impl_list_for_tuple!(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14);
impl_list_for_tuple!(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15);
impl_list_for_tuple!(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16);

