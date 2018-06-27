//! Read `multipart/form-data` content from POST form (see `static/index.html`),
//! save all files to `/tmp/` (see `impl ProcessContent` and `impl MultipartParserTarget`),
//! and parse rest of data into struct `Test`.

#![feature(plugin, decl_macro, try_from, attr_literals, match_default_bindings)]
#![plugin(rocket_codegen)]
#![allow(unused_imports)]

extern crate rocket;
#[macro_use]
extern crate gnitive_multipart_derive;
extern crate gnitive_multipart;
use gnitive_multipart::multipart_parser::{MultipartParser};
use gnitive_multipart::gnitive_multipart::{MultipartParserTarget, ProcessContent, Headers};

use rocket::response::{NamedFile};
use rocket::{Data};
use std::cell::{RefCell};
use std::io::{Cursor, Result};
use std::rc::{Rc};

mod req;
use req::{Req};

mod file_writer;
use file_writer::{FileWriter};

#[get("/")]
fn index() -> Result<NamedFile> {
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

    #[multipart(name="text1")]
    pub s: String,

    writers: Vec<Rc<RefCell<FileWriter>>>
}


impl Test{
    pub fn new() -> Test
    {
        Test
            {
                i: None,
                s: String::new(),
                writers: vec![]
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
        let value_s: String = self.s.clone();

        let mut writers = String::new();
        for writer in &self.writers
            {
                writers.push_str(writer.borrow().to_html().as_str() );
            }

        format!(
            r#"
<!DOCTYPE HTML>
<html>
<head>
	<meta charset="utf-8">
    <title>ex_03_rocket_stream_redirection</title>
</head>
<body>
    <table rules="all">
        <tr><th>Field</th> <th>Value</th></tr>
        <tr><td>i</td>     <td>{value_i}</td></tr>
        <tr><td>s</td>     <td>{value_s}</td></tr>
        {writers}
    </table>
</body>
</html>"#,
            value_i = value_i,
            value_s = value_s,
            writers = writers)
    }
}




impl MultipartParserTarget for Test
{

    /// This method will be called when `MultipartParser` found header with `name` not listed in struct
    fn content_parser(&mut self, _self_: &Rc<RefCell<Self>>, headers: &Headers) -> Option<Rc<RefCell<ProcessContent>>>
    {
        if let Some(name) = headers.get_name()
            {
                match name.as_str()
                    {
                        "file1" |
                        "file2" |
                        "file3" =>
                            {
                                let file_writer = FileWriter::new(name);
                                let rc = Rc::new(RefCell::new(file_writer));
                                self.writers.push(rc.clone());
                                Some (rc)
                            }
                        _ => None
                    }
            }
        else
        {
            // return `None` - skip unknown field
            None
        }
    }
}


fn main() {
    rocket::ignite().mount("/", routes![index, form]).launch();
}
