#[derive(Debug, Clone, PartialEq)]
pub enum PeelType {
    Int,
    Float,
    String,
    Bool,
    Void,
    Option(Box<PeelType>),
    Result(Box<PeelType>, Box<PeelType>),
    #[allow(dead_code)]
    List(Box<PeelType>),
    #[allow(dead_code)]
    Map(Box<PeelType>, Box<PeelType>),
    Func {
        params: Vec<PeelType>,
        ret: Box<PeelType>,
        is_async: bool,
    },
    #[allow(dead_code)]
    Future(Box<PeelType>),
    Object(String), // For named structs/objects
    Unknown,
}

impl PeelType {
    pub fn matches(&self, other: &PeelType) -> bool {
        if self == other || *self == PeelType::Unknown || *other == PeelType::Unknown {
            return true;
        }

        match (self, other) {
            (PeelType::Option(a), PeelType::Option(b)) => a.matches(b),
            (PeelType::Result(a_ok, a_err), PeelType::Result(b_ok, b_err)) => {
                a_ok.matches(b_ok) && a_err.matches(b_err)
            }
            (PeelType::List(a), PeelType::List(b)) => a.matches(b),
            (PeelType::Map(a_k, a_v), PeelType::Map(b_k, b_v)) => {
                a_k.matches(b_k) && a_v.matches(b_v)
            }
            (PeelType::Func { params: a_p, ret: a_r, is_async: a_a },
             PeelType::Func { params: b_p, ret: b_r, is_async: b_a }) => {
                if a_p.len() != b_p.len() || a_a != b_a { return false; }
                a_p.iter().enumerate().all(|(i, p)| p.matches(&b_p[i])) && a_r.matches(b_r)
            }
            _ => false,
        }
    }
}
