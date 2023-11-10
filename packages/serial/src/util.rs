pub fn hex_fmt<T: std::fmt::Debug + AsRef<[u8]>>(
    n: &T,
    f: &mut std::fmt::Formatter,
) -> std::fmt::Result {
    write!(f, "0x{}", hex::encode(n))
}
