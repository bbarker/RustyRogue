use std::marker::Destruct;

pub trait OrdExtra: Ord {
    fn clamp_opt(self, min: Self, max: Self) -> Option<Self>
    where
        Self: Sized,
        Self: ~const Destruct,
        Self: ~const PartialOrd,
    {
        if self < min || self > max {
            None
        } else {
            Some(self)
        }
    }
}

impl<T> OrdExtra for T where T: Ord + Sized + ~const Destruct + ~const PartialOrd {}
