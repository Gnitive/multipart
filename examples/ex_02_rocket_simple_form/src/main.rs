//! Read `multipart/form-data` content from POST form (see `static/index.html`) and parse into struct `Test`

#![feature(plugin, decl_macro, try_from, attr_literals, match_default_bindings)]
#![plugin(rocket_codegen)]
#![allow(unused_imports)]

extern crate rocket;
#[macro_use]
extern crate gnitive_multipart_derive;
extern crate gnitive_multipart;
use gnitive_multipart::multipart_parser::{MultipartParser};
use gnitive_multipart::gnitive_multipart::{MultipartParserTarget};


use rocket::{Data};
use rocket::response::{NamedFile};
use std::cell::{RefCell};
use std::io::{Cursor, Result};
use std::rc::{Rc};

mod req;
use req::{Req};


#[get("/")]
fn index() -> Result<NamedFile>
{
    NamedFile::open("static/index.html")
}

#[post("/form", data = "<data>")]
fn form<'a>(data: Data, request: Req)  -> rocket::Response<'a> {
    let s =
            match request.boundary
                {
                    Some(boundary) =>
                        {
                            let test = Test::new();
                            let target: Rc<RefCell<Test>> = Rc::new(RefCell::new(test));
                            let clone = target.clone();

                            let mut multipart_parser: MultipartParser<Test> = MultipartParser::new_from_str(boundary.as_str(), &target);
                            data.stream_to(&mut multipart_parser).unwrap();

                            let test = clone.borrow();
                            test.to_html()
                        }
                    None =>
                        {
                            "<html><body><p>Cannot found boundary</p></body></html>".to_string()
                        }
                };

    let mut result = rocket::Response::new();
    result.set_sized_body(Cursor::new(s));
    result
}



#[derive(MultipartDerive)]
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

    #[multipart(name="text1")]
    pub s: String,
}


impl Test{
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

    /// dump all fields to html document
    pub fn to_html(&self) -> String
    {
        let value_i: String = match self.i
            {
                Some(i) => i.to_string(),
                None => "<no data>".to_string()
            };

        let value_file1: String = format!("{:?}", &self.file1 );
        let value_file2: String = format!("{:?}", &self.file2 );
        let value_file3: String = self.file3.clone();
        let value_s: String = self.s.clone();

        format!(
            r#"
<!DOCTYPE HTML>
<html>
<head>
	<meta charset="utf-8">
    <title>ex_02_rocket_simple_form</title>
</head>
<body>
    <table rules="all">
        <tr><th>Field</th> <th>Value</th></tr>
        <tr><td>i</td>     <td>{value_i}</td></tr>
        <tr><td>file1</td> <td>{value_file1}</td></tr>
        <tr><td>file2</td> <td>{value_file2}</td></tr>
        <tr><td>file3</td> <td>{value_file3}</td></tr>
        <tr><td>s</td>     <td>{value_s}</td></tr>
    </table>
</body>
</html>"#,
            value_i = value_i,
            value_file1 = value_file1,
            value_file2 = value_file2,
            value_file3 = value_file3,
            value_s = value_s)
    }
}


impl MultipartParserTarget for Test { }


fn main() {
    rocket::ignite().mount("/", routes![index, form]).launch();
}
