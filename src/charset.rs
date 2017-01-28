#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CharSet {
    SingleChar(char),

    Range { lo: char, hi: char },

    AnyChar,

    Diff {
        include: Box<CharSet>,
        exclude: Box<CharSet>,
    },

    Union(Vec<CharSet>),

    Epsilon,
}

impl CharSet {
    pub fn test(&self, c: char) -> bool {
        match self {

            &CharSet::SingleChar(c1) => c1 == c,

            &CharSet::Range { lo, hi } => c >= lo && c <= hi,

            &CharSet::AnyChar => true,

            &CharSet::Diff { ref include, ref exclude } => include.test(c) && !exclude.test(c),

            &CharSet::Union(ref css) => {
                for cs in css {
                    if cs.test(c) {
                        return true;
                    }
                }
                false
            }

            &CharSet::Epsilon => true,
        }
    }
}
