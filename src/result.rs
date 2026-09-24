use core::convert;
use core::ops::{ControlFlow, FromResidual, Residual, Try};
use wdk_sys::NTSTATUS;

#[repr(C)]
pub enum Result<T> {
    Ok(T),
    Status(NTSTATUS),
}

impl<T> Try for Result<T> {
    type Output = T;
    type Residual = Result<convert::Infallible>;

    #[inline]
    fn from_output(output: Self::Output) -> Self {
        Result::Ok(output)
    }
    #[inline]
    fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
        match self {
            Self::Ok(v) => ControlFlow::Continue(v),
            Self::Status(status) => ControlFlow::Break(Result::Status(status)),
        }
    }
}

impl<T> FromResidual<Result<convert::Infallible>> for Result<T> {
    #[inline]
    fn from_residual(r: Result<convert::Infallible>) -> Self {
        match r {
            Result::Status(s) => Result::Status(From::from(s)),
        }
    }
}

impl<T> Residual<T> for Result<convert::Infallible> {
    type TryType = Result<T>;
}
