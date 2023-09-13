use paste::paste;
use std::marker::Destruct;

pub trait OrdExtra: Ord {
    fn clamp_opt(self, min: Self, max: Self) -> Option<Self>
    where
        Self: Sized,
        Self: ~const Destruct,
        Self: PartialOrd,
    {
        if self < min || self > max {
            None
        } else {
            Some(self)
        }
    }
}

impl<T> OrdExtra for T where T: Ord + Sized + ~const Destruct + PartialOrd {}

macro_rules! cloneable_fn {
    ($($arg:ident),* $ (,)?) => {


        // TODO: try the paste macro next time: https://sl.bing.net/sxhvKk23uC
        // count!($($xs)*)
        paste! {
            pub trait [<CloneableFn $($arg)*>]<$($arg,)* O>: Fn($($arg,)*) -> O {
                fn clone_box<'a>(&self) -> Box<dyn 'a + [<CloneableFn $($arg)*>]<$($arg,)* O>>
                where
                    Self: 'a;
            }

            impl<$($arg,)* O, FN: Fn($($arg,)*) -> O + Clone> [<CloneableFn $($arg)*>]<$($arg,)* O> for FN
            {
                fn clone_box<'a>(&self) -> Box<dyn 'a + [<CloneableFn $($arg)*>]<$($arg,)* O>>
                where
                    Self: 'a,
                {
                    Box::new(self.clone())
                }
            }

            impl<'a, $($arg: 'a,)* O: 'a> Clone for Box<dyn 'a + [<CloneableFn $($arg)*>]<$($arg,)* O>> {
                fn clone(&self) -> Self {
                    (**self).clone_box()
                }
            }

        }
    };
}

//cloneable_fn!(); // paste! may not work with 0 args
cloneable_fn!(A);
cloneable_fn!(A, B);
cloneable_fn!(A, B, C);
cloneable_fn!(A, B, C, D);
cloneable_fn!(A, B, C, D, E);
cloneable_fn!(A, B, C, D, E, F);
cloneable_fn!(A, B, C, D, E, F, G);

fn foo(aa: i32, bb: i32) -> i32 {
    aa + bb
}

#[test]
fn cloneable_fn_test_fn() -> () {
    let _foo2 = foo.clone();
    let _foo3 = Box::new(foo).clone_box();
}

// const traits and fns seem to be in flux right now; workaround:
#[inline]
pub const fn max_usize(aa: usize, bb: usize) -> usize {
    if aa > bb {
        aa
    } else {
        bb
    }
}
//
#[inline]
pub const fn min_usize(aa: usize, bb: usize) -> usize {
    if aa < bb {
        aa
    } else {
        bb
    }
}
