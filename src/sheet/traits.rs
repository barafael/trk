pub trait HasTEX {
    fn to_tex(&self) -> String;
}

pub trait HasHTML {
    fn to_html(&self) -> String;
}
