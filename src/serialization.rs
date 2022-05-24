use derive_try_from_primitive::TryFromPrimitive;

use crate::{err::UnexpectedGenericType, DataToken};

#[derive(Copy, Clone, Debug)]
pub enum GenericValue<ArrType, StrType> {
    Vu8(u8),
    Vi8(i8),
    Vu16(u16),
    Vi16(i16),
    Vu32(u32),
    Vi32(i32),
    Vu64(u64),
    Vi64(i64),
    Vf32(f32),
    Vf64(f64),
    Vusize(usize),
    Visize(isize),
    Vbool(bool),
    Vstring(StrType),
    Vbytes(ArrType),
    Token(DataToken),

    Marker(u8),
}

impl<'a> From<&'a GenericValueBoxed> for GenericValueRef<'a> {
    fn from(boxed: &'a GenericValueBoxed) -> Self {
        match boxed {
            GenericValueBoxed::Vu8(v) => GenericValueRef::Vu8(*v),
            GenericValueBoxed::Vi8(v) => GenericValueRef::Vi8(*v),
            GenericValueBoxed::Vu16(v) => GenericValueRef::Vu16(*v),
            GenericValueBoxed::Vi16(v) => GenericValueRef::Vi16(*v),
            GenericValueBoxed::Vu32(v) => GenericValueRef::Vu32(*v),
            GenericValueBoxed::Vi32(v) => GenericValueRef::Vi32(*v),
            GenericValueBoxed::Vu64(v) => GenericValueRef::Vu64(*v),
            GenericValueBoxed::Vi64(v) => GenericValueRef::Vi64(*v),
            GenericValueBoxed::Vf32(v) => GenericValueRef::Vf32(*v),
            GenericValueBoxed::Vf64(v) => GenericValueRef::Vf64(*v),
            GenericValueBoxed::Vusize(v) => GenericValueRef::Vusize(*v),
            GenericValueBoxed::Visize(v) => GenericValueRef::Visize(*v),
            GenericValueBoxed::Vbool(v) => GenericValueRef::Vbool(*v),
            GenericValueBoxed::Vstring(v) => GenericValueRef::Vstring(v.as_str()),
            GenericValueBoxed::Vbytes(v) => GenericValueRef::Vbytes(v.as_slice()),
            GenericValueBoxed::Token(v) => GenericValueRef::Token(*v),
            GenericValueBoxed::Marker(v) => GenericValueRef::Marker(*v),
        }
    }
}

pub type GenericValueRef<'a> = GenericValue<&'a [u8], &'a str>;
pub type GenericValueBoxed = GenericValue<Vec<u8>, String>;

macro_rules! from_generic_type {
    ($t:ty, $cons:ident) => {
        impl<ArrType, StrType> From<$t> for GenericValue<ArrType, StrType> {
            fn from(value: $t) -> Self {
                GenericValue::$cons(value)
            }
        }
    }
}

from_generic_type!(u8, Vu8);
from_generic_type!(i8, Vi8);
from_generic_type!(u16, Vu16);
from_generic_type!(i16, Vi16);
from_generic_type!(u32, Vu32);
from_generic_type!(i32, Vi32);
from_generic_type!(u64, Vu64);
from_generic_type!(i64, Vi64);
from_generic_type!(f32, Vf32);
from_generic_type!(f64, Vf64);
from_generic_type!(usize, Vusize);
from_generic_type!(isize, Visize);
from_generic_type!(bool, Vbool);
from_generic_type!(DataToken, Token);

impl<'a, StrType> From<&'a [u8]> for GenericValue<&'a [u8], StrType> {
    fn from(value: &'a [u8]) -> Self {
        GenericValue::Vbytes(value)
    }
}

impl<StrType> From<Vec<u8>> for GenericValue<Vec<u8>, StrType> {
    fn from(value: Vec<u8>) -> Self {
        GenericValue::Vbytes(value)
    }
}

impl<'a, ArrType> From<&'a str> for GenericValue<ArrType, &'a str> {
    fn from(value: &'a str) -> Self {
        GenericValue::Vstring(value)
    }
}

impl<ArrType> From<String> for GenericValue<ArrType, String> {
    fn from(value: String) -> Self {
        GenericValue::Vstring(value)
    }
}

macro_rules! expect_generic_type {
    ($name: ident, $cons: ident, $ex: ident, $t:ty) => {
        paste::paste! {
            pub fn [<expect_ $name>](&self) -> std::result::Result<&$t, UnexpectedGenericType> {
                match self {
                    GenericValue::$cons(v) => Ok(v),
                    g => Err(UnexpectedGenericType{ expected_type: SerializedType::$ex,  actual_type: g.type_of()})
                }
            }

            pub fn [<expect_ $name _into>](self) -> std::result::Result<$t, UnexpectedGenericType> {
                match self {
                    GenericValue::$cons(v) => Ok(v),
                    g => Err(UnexpectedGenericType{ expected_type: SerializedType::$ex,  actual_type: g.type_of()})
                }
            }
        }
    }
}

impl<ArrType, StrType> GenericValue<ArrType, StrType> {
    fn type_of(&self) -> SerializedType {
        match self {
            GenericValue::Vu8(_) => SerializedType::Su8,
            GenericValue::Vi8(_) => SerializedType::Si8,
            GenericValue::Vu16(_) => SerializedType::Su16,
            GenericValue::Vi16(_) => SerializedType::Si16,
            GenericValue::Vu32(_) => SerializedType::Su32,
            GenericValue::Vi32(_) => SerializedType::Si32,
            GenericValue::Vu64(_) => SerializedType::Su64,
            GenericValue::Vi64(_) => SerializedType::Si64,
            GenericValue::Vf32(_) => SerializedType::Sf32,
            GenericValue::Vf64(_) => SerializedType::Sf64,
            GenericValue::Vusize(_) => SerializedType::Susize,
            GenericValue::Visize(_) => SerializedType::Sisize,
            GenericValue::Vbool(_) => SerializedType::Sbool,
            GenericValue::Vstring(_) => SerializedType::Sstring,
            GenericValue::Vbytes(_) => SerializedType::Sbytes,
            GenericValue::Token(_) => SerializedType::Token,
            GenericValue::Marker(_) => SerializedType::Marker,
        }
    }

    expect_generic_type!(u8, Vu8, Su8, u8);
    expect_generic_type!(i8, Vi8, Si8, i8);
    expect_generic_type!(u16, Vu16, Su16, u16);
    expect_generic_type!(i16, Vi16, Si16, i16);
    expect_generic_type!(u32, Vu32, Su32, u32);
    expect_generic_type!(i32, Vi32, Si32, i32);
    expect_generic_type!(u64, Vu64, Su64, u64);
    expect_generic_type!(i64, Vi64, Si64, i64);
    expect_generic_type!(f32, Vf32, Sf32, f32);
    expect_generic_type!(f64, Vf64, Sf64, f64);
    expect_generic_type!(usize, Vusize, Susize, usize);
    expect_generic_type!(isize, Visize, Sisize, isize);
    expect_generic_type!(bool, Vbool, Sbool, bool);
    expect_generic_type!(string, Vstring, Sstring, StrType);
    expect_generic_type!(bytes, Vbytes, Sbytes, ArrType);
    expect_generic_type!(token, Token, Token, DataToken);
    expect_generic_type!(marker, Marker, Marker, u8);
}

#[repr(u8)]
#[derive(TryFromPrimitive, Copy, Clone, Debug, PartialEq, Eq)]
pub enum SerializedType {
    Su8,
    Si8,
    Su16,
    Si16,
    Su32,
    Si32,
    Su64,
    Si64,
    Sf32,
    Sf64,
    Susize,
    Sisize,
    Sbool,
    Sstring,
    Sbytes,
    Token,

    Marker,
}

pub(crate) mod sealed {
    use std::{
        convert::TryInto,
        io,
        io::{ErrorKind, Read, Write},
    };

    use super::{GenericValue, SerializedType};
    use crate::{endpoint::sealed::*, err::*, protocol::*};

    macro_rules! prim_rw {
        ($name: ident, $t:ty) => {
            paste::paste! {
                fn [<read_ $name>](&mut self) -> io::Result<$t> {
                    let mut v = $t::default();
                    self.read_exact(bytemuck::bytes_of_mut(&mut v))?;
                    Ok(v)
                }

                fn [<write_ $name>](&mut self, v: $t) -> io::Result<&mut Self>{
                    self.write_all(bytemuck::bytes_of(&v))?;
                    Ok(self)
                }
            }
        };
    }

    macro_rules! rw_prim_enum {
        ($name: ident, $t: ty, $err: ident ) => {
            paste::paste! {
                fn [<read_ $name>](&mut self) -> io::Result<$t> {
                    self.read_u8()?.try_into().map_err(|invalid_u8| {
                        io::Error::new(
                            ErrorKind::Other,
                            ProtocolError::$err(invalid_u8),
                        )
                    })
                }

                fn [<write_ $name>](&mut self, v: $t) -> io::Result<&mut Self> {
                    self.write_u8(v as u8)
                }
            }
        };
    }

    pub trait SerializationEndpoint
    where
        Self: Sized,
    {
        fn write_all(&mut self, data: &[u8]) -> io::Result<&mut Self>;

        fn read_exact(&mut self, data: &mut [u8]) -> io::Result<&mut Self>;

        //
        fn read_bytes(&mut self) -> io::Result<Vec<u8>> {
            let size = self.read_usize()?;
            let mut vec = vec![0; size];
            self.read_exact(vec.as_mut_slice())?;
            Ok(vec)
        }

        fn write_bytes(&mut self, value: &[u8]) -> io::Result<&mut Self> {
            self.write_usize(value.len())?;
            self.write_all(value)?;
            Ok(self)
        }

        //
        fn read_bool(&mut self) -> io::Result<bool> {
            Ok(!matches!(self.read_u8()?, 0))
        }

        fn write_bool(&mut self, value: bool) -> io::Result<&mut Self> {
            self.write_u8(match value {
                false => 0,
                true => 1,
            })?;
            Ok(self)
        }

        //
        fn read_string(&mut self) -> io::Result<String> {
            let utf8 = self.read_bytes()?;
            String::from_utf8(utf8).map_err(|str_err| io::Error::new(ErrorKind::Other, str_err))
        }

        fn write_string(&mut self, value: &str) -> io::Result<&mut Self> {
            self.write_bytes(value.as_bytes())
        }

        //
        rw_prim_enum!(protocol, ProtocolConstant, UnknownProtocolConstant);
        rw_prim_enum!(err_type, RemoteErrorType, UnknownErrorType);
        rw_prim_enum!(log_type, LogType, UnknownLogType);

        //
        fn write_gtype(&mut self, p: SerializedType) -> io::Result<&mut Self> {
            self.write_u8(p as u8)
        }

        fn read_generic(&mut self) -> io::Result<GenericValue<Vec<u8>, String>> {
            let g_type: SerializedType = SerializedType::try_from(self.read_u8()?)
                .map_err(ProtocolError::UnknownGenericType)?;

            Ok(match g_type {
                SerializedType::Su8 => GenericValue::Vu8(self.read_u8()?),
                SerializedType::Si8 => GenericValue::Vi8(self.read_i8()?),
                SerializedType::Su16 => GenericValue::Vu16(self.read_u16()?),
                SerializedType::Si16 => GenericValue::Vi16(self.read_i16()?),
                SerializedType::Su32 => GenericValue::Vu32(self.read_u32()?),
                SerializedType::Si32 => GenericValue::Vi32(self.read_i32()?),
                SerializedType::Su64 => GenericValue::Vu64(self.read_u64()?),
                SerializedType::Si64 => GenericValue::Vi64(self.read_i64()?),
                SerializedType::Sf32 => GenericValue::Vf32(self.read_f32()?),
                SerializedType::Sf64 => GenericValue::Vf64(self.read_f64()?),
                SerializedType::Susize => GenericValue::Vusize(self.read_usize()?),
                SerializedType::Sisize => GenericValue::Visize(self.read_isize()?),
                SerializedType::Sbool => GenericValue::Vbool(self.read_bool()?),
                SerializedType::Sstring => GenericValue::Vstring(self.read_string()?),
                SerializedType::Marker => GenericValue::Marker(self.read_u8()?),
                SerializedType::Sbytes => GenericValue::Vbytes(self.read_bytes()?),
                SerializedType::Token => GenericValue::Token(DataToken(self.read_u64()?)),
            })
        }

        fn read_generic_vec(&mut self) -> io::Result<Vec<GenericValue<Vec<u8>, String>>> {
            let length = self.read_usize()?;
            let mut vec = Vec::with_capacity(length);
            for _ in 0..length {
                vec.push(self.read_generic()?);
            }

            Ok(vec)
        }

        fn write_generic(&mut self, value: GenericValue<&[u8], &str>) -> io::Result<&mut Self> {
            match value {
                GenericValue::Vu8(v) => self.write_gtype(SerializedType::Su8)?.write_u8(v)?,
                GenericValue::Vi8(v) => self.write_gtype(SerializedType::Si8)?.write_i8(v)?,
                GenericValue::Vu16(v) => self.write_gtype(SerializedType::Su16)?.write_u16(v)?,
                GenericValue::Vi16(v) => self.write_gtype(SerializedType::Si16)?.write_i16(v)?,
                GenericValue::Vu32(v) => self.write_gtype(SerializedType::Su32)?.write_u32(v)?,
                GenericValue::Vi32(v) => self.write_gtype(SerializedType::Si32)?.write_i32(v)?,
                GenericValue::Vu64(v) => self.write_gtype(SerializedType::Su64)?.write_u64(v)?,
                GenericValue::Vi64(v) => self.write_gtype(SerializedType::Si64)?.write_i64(v)?,
                GenericValue::Vf32(v) => self.write_gtype(SerializedType::Sf32)?.write_f32(v)?,
                GenericValue::Vf64(v) => self.write_gtype(SerializedType::Sf64)?.write_f64(v)?,
                GenericValue::Vusize(v) => {
                    self.write_gtype(SerializedType::Susize)?.write_usize(v)?
                }
                GenericValue::Visize(v) => {
                    self.write_gtype(SerializedType::Sisize)?.write_isize(v)?
                }
                GenericValue::Vbool(v) => self.write_gtype(SerializedType::Sbool)?.write_bool(v)?,

                GenericValue::Vstring(v) => {
                    self.write_gtype(SerializedType::Sstring)?.write_string(v)?
                }
                GenericValue::Vbytes(v) => {
                    self.write_gtype(SerializedType::Sbytes)?.write_bytes(v)?
                }
                GenericValue::Token(DataToken(v)) => {
                    self.write_gtype(SerializedType::Token)?.write_u64(v)?
                }
                GenericValue::Marker(v) => self.write_gtype(SerializedType::Marker)?.write_u8(v)?,
            };
            Ok(self)
        }

        fn write_generic_vec(
            &mut self,
            values: &[GenericValue<&[u8], &str>],
        ) -> io::Result<&mut Self> {
            self.write_usize(values.len())?;
            for v in values {
                self.write_generic(*v)?;
            }

            Ok(self)
        }

        //
        prim_rw!(u8, u8);
        prim_rw!(u16, u16);
        prim_rw!(u32, u32);
        prim_rw!(u64, u64);
        prim_rw!(i8, i8);
        prim_rw!(i16, i16);
        prim_rw!(i32, i32);
        prim_rw!(i64, i64);

        prim_rw!(f32, f32);
        prim_rw!(f64, f64);

        prim_rw!(usize, usize);
        prim_rw!(isize, isize);
    }

    impl<X> SerializationEndpoint for X
    where
        X: Endpoint + Sized,
    {
        fn write_all(&mut self, data: &[u8]) -> io::Result<&mut Self> {
            self.pipes().writer.write_all(data)?;
            Ok(self)
        }

        fn read_exact(&mut self, data: &mut [u8]) -> io::Result<&mut Self> {
            self.pipes().reader.read_exact(data)?;
            Ok(self)
        }
    }
}

