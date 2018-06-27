//! Implement `ProcessContent` trait
//!
//! * `NullProcessor` - Empty processor,
//!
//! * `DefaultProcessor` - buferize all incoming data, convert data to any simple type


use std::convert::{TryFrom};
use std::str::{FromStr, ParseBoolError};
use std::num::{ParseIntError, ParseFloatError};
use std::string::{FromUtf8Error};
use ::gnitive_multipart::{ProcessContent, ProcessParams, Headers};

/// Empty processor - dont process any data
pub struct NullProcessor
{
    /// Only for trait `ProcessContent`
    params: ProcessParams
}

impl NullProcessor
{
    pub fn new() -> NullProcessor
    {
        NullProcessor
            {
                params: ProcessParams::new("", None )
            }
    }
}

impl ProcessContent for NullProcessor
{
    fn open(&mut self, _headers: &Headers) -> () {}

    fn write(&mut self, _headers: &Headers, _data: &Vec<u8>) -> (){}

    fn flush(&mut self, _headers: &Headers) -> (){}

    fn get_process_params(&self) -> &ProcessParams
    {
        &self.params
    }
}


/// Store all data in `raw_data`, can convert to any simple type (see `impl TryFrom` bellow)
pub struct DefaultProcessor
{
    /// Processor parameters, used in `ProcessContent` trait.
    params: ProcessParams,

    /// Buffer to store data in `write` function
    raw_data: Vec<u8>,

    /// `true` after `flush`, `false` otherwise
    is_done: bool
}


impl DefaultProcessor
{
    pub fn new(params: ProcessParams) -> DefaultProcessor
    {
        DefaultProcessor
            {
                params,
                raw_data: vec![],
                is_done: false
            }
    }

    /// Return `true` if all data collected (i.e. `flush` called)
    pub fn is_done(&self) -> bool
    {
        self.is_done
    }

    /// Get access to internal buffer
    pub fn raw_data(&self) -> &Vec<u8>
    {
        &self.raw_data
    }
}


impl ProcessContent for DefaultProcessor
{
    fn open(&mut self, _headers: &Headers) -> ()
    {
        if self.is_done
            {
                self.raw_data.clear();
                self.is_done = false;
            }
    }

    fn write(&mut self, _headers: &Headers, data: &Vec<u8>) -> ()
    {
        if !self.is_done
            {
                self.raw_data.extend(data);
            }
        else
            {
                panic!("'write' called after 'flush' for field '{}'", self.params.name);
            }
    }

    fn flush(&mut self, _headers: &Headers) -> ()
    {
        self.is_done = true;
    }

    fn get_process_params(&self) -> &ProcessParams
    {
        &self.params
    }
}


/* -------- Vec<u8>  -------- */
impl <'a>TryFrom<&'a DefaultProcessor> for Vec<u8>
{
    type Error = !;

    fn try_from(default_processor: &DefaultProcessor) -> Result<Self, Self::Error>
    {
        Ok(default_processor.raw_data.clone())
    }
}


impl TryFrom<DefaultProcessor> for Vec<u8>
{
    type Error = !;

    fn try_from(default_processor: DefaultProcessor) -> Result<Self, Self::Error>
    {
        Ok(default_processor.raw_data.clone())
    }
}


impl <'a>TryFrom<&'a DefaultProcessor> for Option<Vec<u8>>
{
    type Error = !;

    fn try_from(default_processor: &DefaultProcessor) -> Result<Self, Self::Error>
    {
        Ok(Some(default_processor.raw_data.clone()))
    }
}


impl TryFrom<DefaultProcessor> for Option<Vec<u8>>
{
    type Error = !;

    fn try_from(default_processor: DefaultProcessor) -> Result<Self, Self::Error>
    {
        Ok(Some(default_processor.raw_data.clone()))
    }
}


/* -------- String  -------- */


impl <'a>TryFrom<&'a DefaultProcessor> for String
{
    type Error = FromUtf8Error;

    fn try_from(default_processor: &DefaultProcessor) -> Result<Self, Self::Error>
    {
        String::from_utf8(default_processor.raw_data.clone())
    }
}


impl TryFrom<DefaultProcessor> for String
{
    type Error = FromUtf8Error;

    fn try_from(default_processor: DefaultProcessor) -> Result<Self, Self::Error>
    {
        String::from_utf8(default_processor.raw_data.clone())
    }
}


impl <'a>TryFrom<&'a DefaultProcessor> for Option<String>
{
    type Error = FromUtf8Error;

    fn try_from(default_processor: &DefaultProcessor) -> Result<Self, Self::Error>
    {
        match String::from_utf8(default_processor.raw_data.clone())
            {
                Ok(s) => Ok(Some(s)),
                Err(e) => Err(e)
            }
    }
}


impl TryFrom<DefaultProcessor> for Option<String>
{
    type Error = FromUtf8Error;

    fn try_from(default_processor: DefaultProcessor) -> Result<Self, Self::Error>
    {
        match String::from_utf8(default_processor.raw_data.clone())
            {
                Ok(s) => Ok(Some(s)),
                Err(e) => Err(e)
            }
    }
}


/* -------- bool  -------- */

impl <'a>TryFrom<&'a DefaultProcessor> for bool
{
    type Error = ParseBoolError;

    fn try_from(default_processor: &DefaultProcessor) -> Result<Self, Self::Error>
    {
        match String::try_from(default_processor)
            {
                Ok(s) => bool::from_str(s.as_str()),
                Err(e) => bool::from_str(e.to_string().as_str())

            }
    }
}

impl TryFrom<DefaultProcessor> for bool
{
    type Error = ParseBoolError;

    fn try_from(default_processor: DefaultProcessor) -> Result<Self, Self::Error>
    {
        match String::try_from(default_processor)
            {
                Ok(s) => bool::from_str(s.as_str()),
                Err(e) => bool::from_str(e.to_string().as_str())

            }
    }
}

impl <'a>TryFrom<&'a DefaultProcessor> for Option<bool>
{
    type Error = ParseBoolError;

    fn try_from(default_processor: &DefaultProcessor) -> Result<Self, Self::Error>
    {
        match String::try_from(default_processor)
            {
                Ok(s) =>
                    {
                        match bool::from_str(s.as_str())
                            {
                                Ok(b) => Ok(Some(b)),
                                Err(e) => Err(e)
                            }
                    },
                Err(e) =>
                    {
                        match bool::from_str(e.to_string().as_str())
                            {
                                Ok(b) => Ok(Some(b)),
                                Err(e) => Err(e)
                            }

                    }
            }
    }
}


impl TryFrom<DefaultProcessor> for Option<bool>
{
    type Error = ParseBoolError;

    fn try_from(default_processor: DefaultProcessor) -> Result<Self, Self::Error>
    {
        match String::try_from(default_processor)
            {
                Ok(s) =>
                    {
                        match bool::from_str(s.as_str())
                            {
                                Ok(b) => Ok(Some(b)),
                                Err(e) => Err(e)
                            }
                    },
                Err(e) =>
                    {
                        match bool::from_str(e.to_string().as_str())
                            {
                                Ok(b) => Ok(Some(b)),
                                Err(e) => Err(e)
                            }

                    }
            }
    }
}


/* -------- i8  -------- */
impl <'a>TryFrom<&'a DefaultProcessor> for i8
{
    type Error = ParseIntError;

    fn try_from(default_processor: &DefaultProcessor) -> Result<Self, Self::Error>
    {
        match String::try_from(default_processor)
            {
                Ok(s) => s.parse::<i8>(),
                Err(e) => e.to_string().parse::<i8>()
            }
    }
}

impl TryFrom<DefaultProcessor> for i8
{
    type Error = ParseIntError;

    fn try_from(default_processor: DefaultProcessor) -> Result<Self, Self::Error>
    {
        match String::try_from(default_processor)
            {
                Ok(s) => s.parse::<i8>(),
                Err(e) => e.to_string().parse::<i8>()
            }
    }
}

impl <'a>TryFrom<&'a DefaultProcessor> for Option<i8>
{
    type Error = ParseIntError;

    fn try_from(default_processor: &DefaultProcessor) -> Result<Self, Self::Error>
    {
        match String::try_from(default_processor)
            {
                Ok(s) =>
                    match s.parse::<i8>()
                        {
                            Ok(i) => Ok(Some(i)),
                            Err(e) => Err(e)
                        },
                Err(e) =>
                    {
                        match e.to_string().parse::<i8>()
                            {
                                Ok(i) => Ok(Some(i)),
                                Err(e) => Err(e)
                            }
                    }
            }
    }
}


impl TryFrom<DefaultProcessor> for Option<i8>
{
    type Error = ParseIntError;

    fn try_from(default_processor: DefaultProcessor) -> Result<Self, Self::Error>
    {
        match String::try_from(default_processor)
            {
                Ok(s) =>
                    match s.parse::<i8>()
                        {
                            Ok(i) => Ok(Some(i)),
                            Err(e) => Err(e)
                        },
                Err(e) =>
                    {
                        match e.to_string().parse::<i8>()
                            {
                                Ok(i) => Ok(Some(i)),
                                Err(e) => Err(e)
                            }
                    }
            }
    }
}


/* -------- i16  -------- */

impl <'a>TryFrom<&'a DefaultProcessor> for i16
{
    type Error = ParseIntError;

    fn try_from(default_processor: &DefaultProcessor) -> Result<Self, Self::Error>
    {
        match String::try_from(default_processor)
            {
                Ok(s) => s.parse::<i16>(),
                Err(e) => e.to_string().parse::<i16>()
            }
    }
}

impl TryFrom<DefaultProcessor> for i16
{
    type Error = ParseIntError;

    fn try_from(default_processor: DefaultProcessor) -> Result<Self, Self::Error>
    {
        match String::try_from(default_processor)
            {
                Ok(s) => s.parse::<i16>(),
                Err(e) => e.to_string().parse::<i16>()
            }
    }
}

impl <'a>TryFrom<&'a DefaultProcessor> for Option<i16>
{
    type Error = ParseIntError;

    fn try_from(default_processor: &DefaultProcessor) -> Result<Self, Self::Error>
    {
        match String::try_from(default_processor)
            {
                Ok(s) =>
                    match s.parse::<i16>()
                        {
                            Ok(i) => Ok(Some(i)),
                            Err(e) => Err(e)
                        },
                Err(e) =>
                    {
                        match e.to_string().parse::<i16>()
                            {
                                Ok(i) => Ok(Some(i)),
                                Err(e) => Err(e)
                            }
                    }
            }
    }
}

impl TryFrom<DefaultProcessor> for Option<i16>
{
    type Error = ParseIntError;

    fn try_from(default_processor: DefaultProcessor) -> Result<Self, Self::Error>
    {
        match String::try_from(default_processor)
            {
                Ok(s) =>
                    match s.parse::<i16>()
                        {
                            Ok(i) => Ok(Some(i)),
                            Err(e) => Err(e)
                        },
                Err(e) =>
                    {
                        match e.to_string().parse::<i16>()
                            {
                                Ok(i) => Ok(Some(i)),
                                Err(e) => Err(e)
                            }
                    }
            }
    }
}


/* -------- i32  -------- */

impl <'a>TryFrom<&'a DefaultProcessor> for i32
{
    type Error = ParseIntError;

    fn try_from(default_processor: &DefaultProcessor) -> Result<Self, Self::Error>
    {
        match String::try_from(default_processor)
            {
                Ok(s) => s.parse::<i32>(),
                Err(e) => e.to_string().parse::<i32>()
            }
    }
}

impl TryFrom<DefaultProcessor> for i32
{
    type Error = ParseIntError;

    fn try_from(default_processor: DefaultProcessor) -> Result<Self, Self::Error>
    {
        match String::try_from(default_processor)
            {
                Ok(s) => s.parse::<i32>(),
                Err(e) => e.to_string().parse::<i32>()
            }
    }
}

impl <'a>TryFrom<&'a DefaultProcessor> for Option<i32>
{
    type Error = ParseIntError;

    fn try_from(default_processor: &DefaultProcessor) -> Result<Self, Self::Error>
    {
        match String::try_from(default_processor)
            {
                Ok(s) =>
                    match s.parse::<i32>()
                        {
                            Ok(i) => Ok(Some(i)),
                            Err(e) => Err(e)
                        },
                Err(e) =>
                    {
                        match e.to_string().parse::<i32>()
                            {
                                Ok(i) => Ok(Some(i)),
                                Err(e) => Err(e)
                            }
                    }
            }
    }
}

impl TryFrom<DefaultProcessor> for Option<i32>
{
    type Error = ParseIntError;

    fn try_from(default_processor: DefaultProcessor) -> Result<Self, Self::Error>
    {
        match String::try_from(default_processor)
            {
                Ok(s) =>
                    match s.parse::<i32>()
                        {
                            Ok(i) => Ok(Some(i)),
                            Err(e) => Err(e)
                        },
                Err(e) =>
                    {
                        match e.to_string().parse::<i32>()
                            {
                                Ok(i) => Ok(Some(i)),
                                Err(e) => Err(e)
                            }
                    }
            }
    }
}


/* -------- i64  -------- */
impl <'a>TryFrom<&'a DefaultProcessor> for i64
{
    type Error = ParseIntError;

    fn try_from(default_processor: &DefaultProcessor) -> Result<Self, Self::Error>
    {
        match String::try_from(default_processor)
            {
                Ok(s) => s.parse::<i64>(),
                Err(e) => e.to_string().parse::<i64>()
            }
    }
}

impl TryFrom<DefaultProcessor> for i64
{
    type Error = ParseIntError;

    fn try_from(default_processor: DefaultProcessor) -> Result<Self, Self::Error>
    {
        match String::try_from(default_processor)
            {
                Ok(s) => s.parse::<i64>(),
                Err(e) => e.to_string().parse::<i64>()
            }
    }
}

impl <'a>TryFrom<&'a DefaultProcessor> for Option<i64>
{
    type Error = ParseIntError;

    fn try_from(default_processor: &DefaultProcessor) -> Result<Self, Self::Error>
    {
        match String::try_from(default_processor)
            {
                Ok(s) =>
                    match s.parse::<i64>()
                        {
                            Ok(i) => Ok(Some(i)),
                            Err(e) => Err(e)
                        },
                Err(e) =>
                    {
                        match e.to_string().parse::<i64>()
                            {
                                Ok(i) => Ok(Some(i)),
                                Err(e) => Err(e)
                            }
                    }
            }
    }
}

impl TryFrom<DefaultProcessor> for Option<i64>
{
    type Error = ParseIntError;

    fn try_from(default_processor: DefaultProcessor) -> Result<Self, Self::Error>
    {
        match String::try_from(default_processor)
            {
                Ok(s) =>
                    match s.parse::<i64>()
                        {
                            Ok(i) => Ok(Some(i)),
                            Err(e) => Err(e)
                        },
                Err(e) =>
                    {
                        match e.to_string().parse::<i64>()
                            {
                                Ok(i) => Ok(Some(i)),
                                Err(e) => Err(e)
                            }
                    }
            }
    }
}


/* -------- u8  -------- */
impl <'a>TryFrom<&'a DefaultProcessor> for u8
{
    type Error = ParseIntError;

    fn try_from(default_processor: &DefaultProcessor) -> Result<Self, Self::Error>
    {
        match String::try_from(default_processor)
            {
                Ok(s) => s.parse::<u8>(),
                Err(e) => e.to_string().parse::<u8>()
            }
    }
}

impl TryFrom<DefaultProcessor> for u8
{
    type Error = ParseIntError;

    fn try_from(default_processor: DefaultProcessor) -> Result<Self, Self::Error>
    {
        match String::try_from(default_processor)
            {
                Ok(s) => s.parse::<u8>(),
                Err(e) => e.to_string().parse::<u8>()
            }
    }
}

impl <'a>TryFrom<&'a DefaultProcessor> for Option<u8>
{
    type Error = ParseIntError;

    fn try_from(default_processor: &DefaultProcessor) -> Result<Self, Self::Error>
    {
        match String::try_from(default_processor)
            {
                Ok(s) =>
                    match s.parse::<u8>()
                        {
                            Ok(i) => Ok(Some(i)),
                            Err(e) => Err(e)
                        },
                Err(e) =>
                    {
                        match e.to_string().parse::<u8>()
                            {
                                Ok(i) => Ok(Some(i)),
                                Err(e) => Err(e)
                            }
                    }
            }
    }
}

impl TryFrom<DefaultProcessor> for Option<u8>
{
    type Error = ParseIntError;

    fn try_from(default_processor: DefaultProcessor) -> Result<Self, Self::Error>
    {
        match String::try_from(default_processor)
            {
                Ok(s) =>
                    match s.parse::<u8>()
                        {
                            Ok(i) => Ok(Some(i)),
                            Err(e) => Err(e)
                        },
                Err(e) =>
                    {
                        match e.to_string().parse::<u8>()
                            {
                                Ok(i) => Ok(Some(i)),
                                Err(e) => Err(e)
                            }
                    }
            }
    }
}


/* -------- u16 -------- */
impl <'a>TryFrom<&'a DefaultProcessor> for u16
{
    type Error = ParseIntError;

    fn try_from(default_processor: &DefaultProcessor) -> Result<Self, Self::Error>
    {
        match String::try_from(default_processor)
            {
                Ok(s) => s.parse::<u16>(),
                Err(e) => e.to_string().parse::<u16>()
            }
    }
}

impl TryFrom<DefaultProcessor> for u16
{
    type Error = ParseIntError;

    fn try_from(default_processor: DefaultProcessor) -> Result<Self, Self::Error>
    {
        match String::try_from(default_processor)
            {
                Ok(s) => s.parse::<u16>(),
                Err(e) => e.to_string().parse::<u16>()
            }
    }
}

impl <'a>TryFrom<&'a DefaultProcessor> for Option<u16>
{
    type Error = ParseIntError;

    fn try_from(default_processor: &DefaultProcessor) -> Result<Self, Self::Error>
    {
        match String::try_from(default_processor)
            {
                Ok(s) =>
                    match s.parse::<u16>()
                        {
                            Ok(i) => Ok(Some(i)),
                            Err(e) => Err(e)
                        },
                Err(e) =>
                    {
                        match e.to_string().parse::<u16>()
                            {
                                Ok(i) => Ok(Some(i)),
                                Err(e) => Err(e)
                            }
                    }
            }
    }
}

impl TryFrom<DefaultProcessor> for Option<u16>
{
    type Error = ParseIntError;

    fn try_from(default_processor: DefaultProcessor) -> Result<Self, Self::Error>
    {
        match String::try_from(default_processor)
            {
                Ok(s) =>
                    match s.parse::<u16>()
                        {
                            Ok(i) => Ok(Some(i)),
                            Err(e) => Err(e)
                        },
                Err(e) =>
                    {
                        match e.to_string().parse::<u16>()
                            {
                                Ok(i) => Ok(Some(i)),
                                Err(e) => Err(e)
                            }
                    }
            }
    }
}


/* -------- u32 -------- */
impl <'a>TryFrom<&'a DefaultProcessor> for u32
{
    type Error = ParseIntError;

    fn try_from(default_processor: &DefaultProcessor) -> Result<Self, Self::Error>
    {
        match String::try_from(default_processor)
            {
                Ok(s) => s.parse::<u32>(),
                Err(e) => e.to_string().parse::<u32>()
            }
    }
}

impl TryFrom<DefaultProcessor> for u32
{
    type Error = ParseIntError;

    fn try_from(default_processor: DefaultProcessor) -> Result<Self, Self::Error>
    {
        match String::try_from(default_processor)
            {
                Ok(s) => s.parse::<u32>(),
                Err(e) => e.to_string().parse::<u32>()
            }
    }
}

impl <'a>TryFrom<&'a DefaultProcessor> for Option<u32>
{
    type Error = ParseIntError;

    fn try_from(default_processor: &DefaultProcessor) -> Result<Self, Self::Error>
    {
        match String::try_from(default_processor)
            {
                Ok(s) =>
                    match s.parse::<u32>()
                        {
                            Ok(i) => Ok(Some(i)),
                            Err(e) => Err(e)
                        },
                Err(e) =>
                    {
                        match e.to_string().parse::<u32>()
                            {
                                Ok(i) => Ok(Some(i)),
                                Err(e) => Err(e)
                            }
                    }
            }
    }
}


impl TryFrom<DefaultProcessor> for Option<u32>
{
    type Error = ParseIntError;

    fn try_from(default_processor: DefaultProcessor) -> Result<Self, Self::Error>
    {
        match String::try_from(default_processor)
            {
                Ok(s) =>
                    match s.parse::<u32>()
                        {
                            Ok(i) => Ok(Some(i)),
                            Err(e) => Err(e)
                        },
                Err(e) =>
                    {
                        match e.to_string().parse::<u32>()
                            {
                                Ok(i) => Ok(Some(i)),
                                Err(e) => Err(e)
                            }
                    }
            }
    }
}


/* -------- u64 -------- */
impl <'a>TryFrom<&'a DefaultProcessor> for u64
{
    type Error = ParseIntError;

    fn try_from(default_processor: &DefaultProcessor) -> Result<Self, Self::Error>
    {
        match String::try_from(default_processor)
            {
                Ok(s) => s.parse::<u64>(),
                Err(e) => e.to_string().parse::<u64>()
            }
    }
}

impl TryFrom<DefaultProcessor> for u64
{
    type Error = ParseIntError;

    fn try_from(default_processor: DefaultProcessor) -> Result<Self, Self::Error>
    {
        match String::try_from(default_processor)
            {
                Ok(s) => s.parse::<u64>(),
                Err(e) => e.to_string().parse::<u64>()
            }
    }
}

impl <'a>TryFrom<&'a DefaultProcessor> for Option<u64>
{
    type Error = ParseIntError;

    fn try_from(default_processor: &DefaultProcessor) -> Result<Self, Self::Error>
    {
        match String::try_from(default_processor)
            {
                Ok(s) =>
                    match s.parse::<u64>()
                        {
                            Ok(i) => Ok(Some(i)),
                            Err(e) => Err(e)
                        },
                Err(e) =>
                    {
                        match e.to_string().parse::<u64>()
                            {
                                Ok(i) => Ok(Some(i)),
                                Err(e) => Err(e)
                            }
                    }
            }
    }
}

impl TryFrom<DefaultProcessor> for Option<u64>
{
    type Error = ParseIntError;

    fn try_from(default_processor: DefaultProcessor) -> Result<Self, Self::Error>
    {
        match String::try_from(default_processor)
            {
                Ok(s) =>
                    match s.parse::<u64>()
                        {
                            Ok(i) => Ok(Some(i)),
                            Err(e) => Err(e)
                        },
                Err(e) =>
                    {
                        match e.to_string().parse::<u64>()
                            {
                                Ok(i) => Ok(Some(i)),
                                Err(e) => Err(e)
                            }
                    }
            }
    }
}


/* -------- f32 -------- */
impl <'a>TryFrom<&'a DefaultProcessor> for f32
{
    type Error = ParseFloatError;

    fn try_from(default_processor: &DefaultProcessor) -> Result<Self, Self::Error>
    {
        match String::try_from(default_processor)
            {
                Ok(s) => s.parse::<f32>(),
                Err(e) => e.to_string().parse::<f32>()
            }
    }
}

impl TryFrom<DefaultProcessor> for f32
{
    type Error = ParseFloatError;

    fn try_from(default_processor: DefaultProcessor) -> Result<Self, Self::Error>
    {
        match String::try_from(default_processor)
            {
                Ok(s) => s.parse::<f32>(),
                Err(e) => e.to_string().parse::<f32>()
            }
    }
}

impl <'a>TryFrom<&'a DefaultProcessor> for Option<f32>
{
    type Error = ParseFloatError;

    fn try_from(default_processor: &DefaultProcessor) -> Result<Self, Self::Error>
    {
        match String::try_from(default_processor)
            {
                Ok(s) =>
                    match s.parse::<f32>()
                        {
                            Ok(i) => Ok(Some(i)),
                            Err(e) => Err(e)
                        },
                Err(e) =>
                    {
                        match e.to_string().parse::<f32>()
                            {
                                Ok(i) => Ok(Some(i)),
                                Err(e) => Err(e)
                            }
                    }
            }
    }
}

impl TryFrom<DefaultProcessor> for Option<f32>
{
    type Error = ParseFloatError;

    fn try_from(default_processor: DefaultProcessor) -> Result<Self, Self::Error>
    {
        match String::try_from(default_processor)
            {
                Ok(s) =>
                    match s.parse::<f32>()
                        {
                            Ok(i) => Ok(Some(i)),
                            Err(e) => Err(e)
                        },
                Err(e) =>
                    {
                        match e.to_string().parse::<f32>()
                            {
                                Ok(i) => Ok(Some(i)),
                                Err(e) => Err(e)
                            }
                    }
            }
    }
}


/* -------- f64 -------- */
impl <'a>TryFrom<&'a DefaultProcessor> for f64
{
    type Error = ParseFloatError;

    fn try_from(default_processor: &DefaultProcessor) -> Result<Self, Self::Error>
    {
        match String::try_from(default_processor)
            {
                Ok(s) => s.parse::<f64>(),
                Err(e) => e.to_string().parse::<f64>()
            }
    }
}

impl TryFrom<DefaultProcessor> for f64
{
    type Error = ParseFloatError;

    fn try_from(default_processor: DefaultProcessor) -> Result<Self, Self::Error>
    {
        match String::try_from(default_processor)
            {
                Ok(s) => s.parse::<f64>(),
                Err(e) => e.to_string().parse::<f64>()
            }
    }
}

impl <'a>TryFrom<&'a DefaultProcessor> for Option<f64>
{
    type Error = ParseFloatError;

    fn try_from(default_processor: &DefaultProcessor) -> Result<Self, Self::Error>
    {
        match String::try_from(default_processor)
            {
                Ok(s) =>
                    match s.parse::<f64>()
                        {
                            Ok(i) => Ok(Some(i)),
                            Err(e) => Err(e)
                        },
                Err(e) =>
                    {
                        match e.to_string().parse::<f64>()
                            {
                                Ok(i) => Ok(Some(i)),
                                Err(e) => Err(e)
                            }
                    }
            }
    }
}

impl TryFrom<DefaultProcessor> for Option<f64>
{
    type Error = ParseFloatError;

    fn try_from(default_processor: DefaultProcessor) -> Result<Self, Self::Error>
    {
        match String::try_from(default_processor)
            {
                Ok(s) =>
                    match s.parse::<f64>()
                        {
                            Ok(i) => Ok(Some(i)),
                            Err(e) => Err(e)
                        },
                Err(e) =>
                    {
                        match e.to_string().parse::<f64>()
                            {
                                Ok(i) => Ok(Some(i)),
                                Err(e) => Err(e)
                            }
                    }
            }
    }
}
