pub struct FourCC([u8; 4]);

impl From<&[u8]> for FourCC {
    fn from(value: &[u8]) -> Self {
        let [a, b, c, d] = value else {
            todo!();
        };

        Self([*a, *b, *c, *d])
    }
}

impl std::fmt::Debug for FourCC {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "FourCC({}{}{}{})",
            self.0[0] as char, self.0[1] as char, self.0[2] as char, self.0[3] as char
        ))
    }
}
