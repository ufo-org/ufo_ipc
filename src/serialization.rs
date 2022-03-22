use std::{
    convert::TryInto,
    io::{Error, ErrorKind, Read, Result, Write},
};

use crate::protocol::*;

use paste::paste;

macro_rules! prim_rw {
    ($name: ident, $t:ty) => {
        paste! {
            fn [<read_ $name>](&mut self) -> Result<$t> {
                let mut v = $t::default();
                self.read_exact(bytemuck::bytes_of_mut(&mut v))?;
                Ok(v)
            }

            fn [<write_ $name>](&mut self, v: $t) -> Result<&mut Self>{
                self.write_all(bytemuck::bytes_of(&v))?;
                Ok(self)
            }
        }
    };
}

pub trait SerializationEndpoint
where
    Self: Sized,
{
    fn write_all(&mut self, data: &[u8]) -> Result<&mut Self>;

    fn read_exact(&mut self, data: &mut [u8]) -> Result<&mut Self>;

    //
    fn read_bytes(&mut self, read_to: &mut [u8]) -> Result<&mut Self> {
        self.read_exact(read_to)?;
        Ok(self)
    }

    fn write_bytes(&mut self, value: &[u8]) -> Result<&mut Self> {
        self.write_all(value)?;
        Ok(self)
    }

    //
    fn read_bool(&mut self) -> Result<bool> {
        Ok(!matches!(self.read_u8()?, 0))
    }

    fn write_bool(&mut self, value: bool) -> Result<&mut Self> {
        self.write_u8(match value {
            false => 0,
            true => 1,
        })?;
        Ok(self)
    }

    //
    fn read_string(&mut self) -> Result<String> {
        let size = self.read_usize()?;

        let mut utf8 = vec![0; size];
        self.read_bytes(utf8.as_mut_slice())?;

        String::from_utf8(utf8).map_err(|str_err| Error::new(ErrorKind::Other, str_err))
    }

    fn write_string(&mut self, value: &str) -> Result<&mut Self> {
        let size = value.len();
        let bytes = value.as_bytes();

        self.write_usize(size)?.write_bytes(bytes)
    }

    //
    fn read_protocol(&mut self) -> Result<ProtocolConstant> {
        self.read_u8()?.try_into().map_err(|str_err| {
            Error::new(
                ErrorKind::Other,
                ProtocolError::UnknownProtocolConstant(str_err),
            )
        })
    }

    fn write_protocol(&mut self, p: ProtocolConstant) -> Result<&mut Self>{
        self.write_u8(p as u8)
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

    prim_rw!(usize, usize);
    prim_rw!(isize, isize);
}

impl<X> SerializationEndpoint for X
where
    X: Endpoint + Sized,
{
    fn write_all(&mut self, data: &[u8]) -> Result<&mut Self> {
        self.pipes().writer.write_all(data)?;
        Ok(self)
    }

    fn read_exact(&mut self, data: &mut [u8]) -> Result<&mut Self> {
        self.pipes().reader.read_exact(data)?;
        Ok(self)
    }
}
