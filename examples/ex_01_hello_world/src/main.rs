//! Read `multipart/form-data` content from file (`data/form_data.txt`) in `Vec<u8>` and parse into struct `Test`

#![feature(attr_literals, try_from)]
#[macro_use]
extern crate gnitive_multipart_derive;
extern crate gnitive_multipart;

use std::cell::RefCell;
use std::convert::TryFrom;
use std::rc::Rc;

use gnitive_multipart::multipart_parser::MultipartParser;
use gnitive_multipart::gnitive_multipart::{Headers, MultipartParserTarget, ProcessContent, ToMultipartParseError};


#[derive(MultipartDerive)]
#[derive(Debug)]
#[multipart]
struct Test
{
    #[multipart]
    pub i: Option<i32>,

    #[multipart]
    pub file1: Option<Vec<u8>>,

    #[multipart]
    pub file2: Vec<u8>,

    #[multipart]
    pub file3: String,

    #[multipart(name = "text1")]
    pub s: String,
}


impl Test {
    pub fn new() -> Test
    {
        Test
            {
                i: None,
                file1: None,
                file2: vec![],
                file3: String::new(),
                s: String::new(),
            }
    }

    pub fn dump(&self) -> ()
    {
        println!("{:?}", self)
    }
}


/// default implementation `gnitive_multipart::MultipartParserTarget`
impl MultipartParserTarget for Test {}


fn main() {
    use std::io::prelude::*;
    let boundary = "---------------------------735323031399963166993862150";

    // path to form_data.txt
    let path =
        {
            use std::env;
            // /.../ex_01_hello_world/target/debug/ex_01_hello_world
            let mut path = env::current_exe().unwrap();
            path.pop();
            path.pop();
            path.pop();

            // /.../ex_01_hello_world/data/form_data.txt
            path.push("data/form_data.txt");
            path
        };


    // read all content form_data.txt to buf
    let buf: Vec<u8> =
        {
            use std::fs::File;
            use std::io::BufReader;

            let file = File::open(path).unwrap();
            let mut buf_reader = BufReader::new(file);

            let mut buf = Vec::new();
            buf_reader.read_to_end(&mut buf).unwrap();
            buf
        };


    let test = Test::new();
    let target: Rc<RefCell<Test>> = Rc::new(RefCell::new(test));

    let mut multipart_parser: MultipartParser<Test> = MultipartParser::new_from_str(boundary, &target);
    multipart_parser.write(buf.as_ref()).unwrap();
    multipart_parser.flush().unwrap();

    target.borrow().dump();
}
