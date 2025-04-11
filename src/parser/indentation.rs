use std::cmp::Ordering;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Indentation {
    Smaller,
    Same,
    Larger,
}

impl Indentation {
    pub fn check(current_indentation: usize, next_indentation: usize) -> Option<Self> {
        if next_indentation % 4 != 0 {
            return None;
        }

        Some(match current_indentation.cmp(&next_indentation) {
            Ordering::Less => Self::Smaller,
            Ordering::Equal => Self::Same,
            Ordering::Greater => Self::Larger,
        })
    }
}
