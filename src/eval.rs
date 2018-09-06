pub enum Arg<'str> {
    Int(usize),
    Float(f64),
    Str(&'s str)
}