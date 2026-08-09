use core::mem::MaybeUninit;
use core::cell::UnsafeCell;

use na::SMatrix;
use spin::Mutex;




pub type ToSafe<T> = Mutex<MaybeUninit<T>>;

pub type SafeMatrix<T, const R: usize, const C: usize> = ToSafe<SMatrix<T, R, C>>;

