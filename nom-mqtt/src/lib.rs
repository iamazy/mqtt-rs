#[macro_use]
extern crate nom;

use nom::error::{ErrorKind, VerboseError};
use nom::Err as NomErr;

type IResult<I, O, E = (I, ErrorKind)> = Result<(I, O), NomErr<E>>;
type Res<T, U> = IResult<T, U, VerboseError<T>>;

#[cfg(test)]
mod tests;
mod bytes;