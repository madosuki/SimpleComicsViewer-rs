use std::str::FromStr;

#[repr(i64)]
#[derive(Default, Clone, Copy, Debug)]
pub enum PageDirection {
    #[default]
    RightToLeft = 0,
    LeftToRight = 1,
}

impl TryFrom<i64> for PageDirection {
    type Error = ();

    fn try_from(val: i64) -> Result<Self, Self::Error> {
        match val {
            0 => Ok(PageDirection::RightToLeft),
            1 => Ok(PageDirection::LeftToRight),
            _ => Err(()),
        }
    }
}

impl FromStr for PageDirection {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "rtl" => Ok(PageDirection::RightToLeft),
            "ltr" => Ok(PageDirection::LeftToRight),
            _ => Err(()),
        }
    }
}

impl PageDirection {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::RightToLeft => "rtl",
            Self::LeftToRight => "ltr",
        }
    }
}
