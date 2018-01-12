use std::cell::{RefCell};
use std::rc::{Rc};
use header::{HeadersBuilder};
use boundary_builder::{BoundaryBuilder};
use std::io::{Write, Error};
use ::gnitive_multipart::{MultipartParserTarget, MultipartParserTargetGenerated, MultipartParseError, ProcessContent, Headers, OnError};

#[derive(Debug)]
#[derive(PartialEq)]
/// Internal state for `MultipartParser`
pub enum MultipartParserState
{
    /// Read first boundary
    BoundaryFirst,

    /// Read header line (`Content-Type`, `Content-Disposition` etc)
    Header,

    /// read next line of metadata or content?
    PostHeader,

    /// Read content (up to boundary)
    Content,

    /// Read next metadata or end of data?
    PostBoundary,

    /// Edn of data reached
    Finished,
}

pub struct MultipartParser<T: MultipartParserTarget + MultipartParserTargetGenerated>
{
    /// First boundary in body - without `\r\n` in head
    ///
    /// ```text
    /// --<boundary>\r\n
    /// ```
    boundary_first: Rc<RefCell<Vec<u8>>>,

    /// All next boundaries
    ///
    /// ```text
    /// \r\n--<boundary>
    /// ```
    boundary_middle: Rc<RefCell<Vec<u8>>>,

    ///  Byte sequence between boundary and metadata
    ///
    /// ```text
    /// \r\n
    /// ```
    divider: Rc<RefCell<Vec<u8>>>,

    /// Finish byte sequence - end of data marker
    ///
    /// ```text
    /// --\r\n
    /// ```
    epilogue: Rc<RefCell<Vec<u8>>>,

    /// CR LF
    ///
    /// ```text
    /// \r\n
    /// ```
    empty_string: Rc<RefCell<Vec<u8>>>,

    /// State
    state: MultipartParserState,

    /// Headers for current field
    headers: Option<Headers>,

    /// Builder for current headers
    headers_builder: HeadersBuilder,

    /// Current data processor
    process_content: Option<Box<ProcessContent>>,

    compare_pos: usize,
    content_start: usize,
    content_end: usize,
    content_size: usize,
    content_size_max: Option<usize>,
    buf_pos: usize,
    unprocessed: Vec<String>,
    on_error: OnError,
    error_fired: bool,

    /// Target struct
    target: Rc<RefCell<T>>
}

impl <T>Write for MultipartParser<T>
    where T: MultipartParserTarget + MultipartParserTargetGenerated
{
    fn write(&mut self, buf: &[u8]) -> Result<usize, Error>
    {
        self.content_start = 0;

        for pos in 0..buf.len()
            {
                self.buf_pos = pos;
                let c = buf[pos];

                match self.state
                    {
                        MultipartParserState::BoundaryFirst => self.process_boundary_first(c),
                        MultipartParserState::Header => self.process_header(c),
                        MultipartParserState::PostHeader => self.process_post_header(c),
                        MultipartParserState::Content => {
                            // Only in this state `MultipartParserTarget::error` function might be called
                            match self.process_content(c, buf)
                                {
                                    Ok(_) => (),
                                    Err(io_error) => return Err(io_error)
                                }
                        },
                        MultipartParserState::PostBoundary => self.process_post_boundary(c),
                        MultipartParserState::Finished => ()
                    };
            }
        Ok(buf.len())
    }

    fn flush(&mut self) -> Result<(), Error>
    {
        if !self.unprocessed.is_empty()
            {
                // All data processed - result of `error` can be ignored.
                match self.target.borrow_mut().error( &MultipartParseError::RequiredMissing(&self.unprocessed) )
                    {
                        Ok(_) => (),
                        Err(_) => ()
                    }
            }

        self.target.borrow_mut().finish();
        Ok(())
    }
}


impl <T>MultipartParser<T>
    where T: MultipartParserTarget + MultipartParserTargetGenerated
{

    /// Create `MultipartParser` for struct `target` with known string `boundary`
    pub fn new_from_str(boundary: &str, target: &Rc<RefCell<T>>) -> Self
    {
        let boundary: Vec<u8> = boundary.as_bytes().to_vec();
        MultipartParser::new_from_vec(boundary, target)
    }

    /// Create `MultipartParser` for struct `target` with known vector `boundary`
    pub fn new_from_vec(boundary: Vec<u8>, target: &Rc<RefCell<T>>) -> Self
    {
        let mut boundary_first: Vec<u8> = Vec::new();
        boundary_first
            .append_prelude()
            .append_boundary(&boundary)
            .append_crlf();

        let mut boundary_middle: Vec<u8> = Vec::new();
        boundary_middle
            .append_crlf()
            .append_prelude()
            .append_boundary(&boundary);

        let mut divider: Vec<u8> = Vec::new();
        divider
            .append_crlf();

        let mut epilogue: Vec<u8> = Vec::new();
        epilogue
            .append_prelude();

        let mut empty_string: Vec<u8> = Vec::new();
        empty_string
            .append_crlf();

        let unprocessed = target.borrow().get_all_required();


        MultipartParser
            {
                boundary_first: Rc::new(RefCell::new(boundary_first)),
                boundary_middle: Rc::new(RefCell::new(boundary_middle)),
                divider: Rc::new(RefCell::new(divider)),
                epilogue: Rc::new(RefCell::new(epilogue)),
                empty_string: Rc::new(RefCell::new(empty_string)),

                state: MultipartParserState::BoundaryFirst,
                headers: None,
                headers_builder: HeadersBuilder::new(),
                process_content: None,
                compare_pos: 0,
                content_start: 0,
                content_end: 0,
                content_size: 0,
                content_size_max: None,
                buf_pos: 0,
                unprocessed,
                on_error: OnError::ContinueWithError,
                error_fired: false,

                target: target.clone()
            }
    }


    /// Change internal state to `Header`
    fn to_header(&mut self) -> ()
    {
        self.compare_pos = 0;
        self.headers = None;
        self.state = MultipartParserState::Header;
    }

    /// Change internal state to `Header` (from `PostHeader`)
    fn to_header_continue(&mut self) -> ()
    {
        self.compare_pos = 0;
        self.state = MultipartParserState::Header;
    }

    /// Change internal state to `PostHeader`
    fn to_post_header(&mut self) -> ()
    {
        self.compare_pos = 0;
        self.headers_builder.flush();
        self.state = MultipartParserState::PostHeader;
    }


    /// Change internal state to `Content`
    fn to_content(&mut self) -> ()
    {
        self.content_start = self.buf_pos;
        self.content_size = 0;
        self.on_error = OnError::ContinueWithError;
        self.error_fired = false;


        let headers = self.headers_builder.build();

        {
            let target = self.target.borrow();
            self.process_content = target.content_parser_generated(&self.target, &headers);

            self.content_size_max = match &self.process_content
                {
                    &Some(ref process_content) => process_content.get_process_params().max_size.clone(),
                    &None => None
                };

            {
                let name = &headers.get_name();
                if let &Some(ref name) = name
                    {
                        self.unprocessed.remove_item(&name);
                    }
            }

            self.headers = Some(headers);
        }

        self.processor_open();

        self.compare_pos = 0;
        self.state = MultipartParserState::Content;
    }

    /// Change internal state to `PostBoundary`
    fn to_post_boundary(&mut self) -> ()
    {
        self.content_start = 0;
        self.compare_pos = 0;
        self.state = MultipartParserState::PostBoundary;
    }

    /// Change internal state to `Finished`
    fn to_finished(&mut self) -> ()
    {
        self.state = MultipartParserState::Finished;
    }



    /// Compare `boundary[compare_pos]` and `c`
    /// Return (`boundary[compare_pos]==c`, `compare_pos+1 = boundary.len()`)
    fn compare (&self, c: u8, boundary: &Rc<RefCell<Vec<u8>>>) -> (bool, bool)
    {
        self.compare_at(c, boundary, self.compare_pos)
    }

    /// Compare `boundary[pos]` and `c`
    /// Return (`boundary[pos]==c`, `pos+1 = boundary.len()`)
    fn compare_at (&self, c: u8, boundary: &Rc<RefCell<Vec<u8>>>, pos: usize) -> (bool, bool)
    {
        let vec = boundary.borrow();
        if c == vec[pos]
            {
                (true, pos +1 == vec.len())
            }
            else
            {
                (false, false)
            }
    }

    /// Read boundary from stream, switch to `Header` when boundary completed
    ///
    /// First boundary in multipart/form-data is different to other - without `\r\n` in head
    fn process_boundary_first(&mut self, c: u8) ->()
    {
        let (sym_equal, boundary_equal) = self.compare(c, &self.boundary_first);
        if boundary_equal
            {
                self.to_header();
                return;
            }

        if !sym_equal
            {
                panic!("Cannot parse first boundary, invalid symbol '{}' at position {}", c, &self.compare_pos);
            }
        self.compare_pos += 1;
    }

    /// Read header from stream, switch to `PostHeader` when `\r\n` readed
    fn process_header(&mut self, c: u8) ->()
    {
        let (sym_equal, boundary_equal) = self.compare(c, &self.empty_string);
        if boundary_equal
            {
                self.to_post_header();
                return;
            }

        if sym_equal
            {
                self.compare_pos+=1;
            }
            else
            {
                if self.compare_pos > 0
                    {
                        let tmp = &self.empty_string.clone();
                        let empty_string: &Vec<u8> = &tmp.borrow();
                        self.headers_builder.write(empty_string[0]);

                        let compare_pos_last = self.compare_pos;
                        self.compare_pos = 0;
                        for i in 1..compare_pos_last
                            {
                                self.process_header(empty_string[i]);
                            }
                    }
                    else
                    {
                        self.headers_builder.write(c);
                    }
            }
    }

    /// Read post header from stream, switch to `Content` when `\r\n` readed (i.e. 2x empty string) or returns to `Header` state if other synbos readed
    fn process_post_header(&mut self, c: u8) ->()
    {
        let (sym_equal, boundary_equal) = self.compare(c, &self.empty_string);
        if boundary_equal
            {
                self.to_content();
                return;
            }

        if sym_equal
            {
                self.compare_pos+=1;
            }
            else
            {
                let tmp = &self.empty_string.clone();
                let empty_string: &Vec<u8> = &tmp.borrow();
                self.headers_builder.write(empty_string[0]);

                let compare_pos_last = self.compare_pos;
                self.compare_pos = 0;
                for i in 1..compare_pos_last
                    {
                        self.process_header(empty_string[i]);
                    }
                self.to_header_continue();
            }
    }

    /// Read content from stream, until `boundary_middle` sequence readed
    fn process_content(&mut self, c: u8, buf: &[u8]) -> Result<(), Error>
    {
        let (sym_equal, boundary_equal) = self.compare(c, &self.boundary_middle);
        if boundary_equal
            {
                self.processor_flush();
                self.to_post_boundary();
                return Ok(())
            }

        if sym_equal
            {
                if self.compare_pos == 0
                    {
                        self.content_end = self.buf_pos;
                        self.processor_write(buf)?;
                    }
                self.compare_pos += 1;
            }
            else
            {
                if self.compare_pos > 0
                    {
                        let clone = self.boundary_middle.clone();
                        let vec = clone.borrow();

                        self.processor_write_sym(vec[0])?;

                        if self.compare_pos > 1
                            {
                                let to = self.compare_pos;
                                match  self.flow_content(1, to)
                                    {
                                        Ok(pos) => self.compare_pos = pos,
                                        Err(e) => return Err(e)
                                    };
                            }
                        else
                        {
                            self.compare_pos = 0;
                        }
                    }
            }
        Ok(())
    }

    /// Special case - part of `boundary_middle` readed from stream, but it is part of content body
    fn flow_content(&mut self, from: usize, to: usize) -> Result<usize, Error>
    {
        let clone = self.boundary_middle.clone();
        let vec = clone.borrow();

        let mut pos: usize = from;
        while pos < to
            {
                let c = vec[pos];
                let (sym_equal, _boundary_equal) = self.compare_at(c, &self.boundary_middle, pos);
                if sym_equal
                    {
                        if pos > from
                            {
                                match self.processor_write_from_to(vec.as_ref(), from, pos)
                                    {
                                        Ok(_) => (),
                                        Err(e) => return Err(e)
                                    }
                            }
                        break
                    }
                pos+=1;
            }

        Ok(to - pos)
    }

    /// `boundary_middle` successfully read - next may be `--` (end of data) or `\r\n` (header and content)
    fn process_post_boundary(&mut self, c: u8) ->()
    {
        let (_divider_sym_equal, divider_boundary_equal) = self.compare(c, &self.divider);
        let (_epilogue_sym_equal, epilogue_boundary_equal) = self.compare(c, &self.epilogue);

        if divider_boundary_equal
            {
                self.to_header();
            }
        if epilogue_boundary_equal
            {
                self.to_finished();
            }
        if !divider_boundary_equal && !epilogue_boundary_equal
            {
                self.compare_pos +=1;
            }
    }

    /// Call `open` for current processor
    fn processor_open(&mut self) -> ()
    {
        if let Some(ref mut process_content) = self.process_content
            {
                if let Some(ref headers ) = self.headers
                    {
                        process_content.open(headers);
                    }

            }
    }


    /// Write `buf` to current processor
    fn processor_write(&mut self, buf: &[u8]) -> Result<(), Error>
    {
        let from = self.content_start;
        let to = self.content_end;
        self.processor_write_from_to(buf, from, to)
    }

    /// Write `buf[from...to]` to current processor
    fn processor_write_from_to(&mut self, buf: &[u8], from: usize, to: usize) -> Result<(), Error>
    {
        if self.on_error == OnError::Skip
            {
                return Ok(());
            }

        if let Some(ref mut process_content) = self.process_content
            {
                if let Some(max_size) = self.content_size_max
                    {
                        self.content_size +=  to - from;

                        if self.content_size > max_size
                            {
                                match self.on_error
                                    {
                                        OnError::Skip => (),
                                        OnError::ContinueWithoutError => (),
                                        OnError::ContinueWithError =>
                                            {
                                                let name = &process_content.get_process_params().name;
                                                let on_error = self.target.borrow_mut().error( &MultipartParseError::SizeLimit(name.clone(), max_size ));
                                                match on_error
                                                    {
                                                        Ok(on_error) =>
                                                            {
                                                                self.on_error = on_error;

                                                            },
                                                        Err(e) => return Err(e)
                                                    }
                                                self.content_size_max = None;
                                            }
                                    }
                            }
                    }

                if let Some(ref headers ) = self.headers
                    {
                        let data: Vec<u8> = buf[from..to].to_vec();
                        process_content.write(&headers, &data);
                    }

            }
        Ok(())
    }


    /// Write one symbol `c` to current processor
    fn processor_write_sym(&mut self, c: u8) -> Result<(), Error>
    {
        let a = [c];
        self.processor_write_from_to(&a, 0, 1)
    }

    /// Call `flush` for current processor
    fn processor_flush(&mut self) -> ()
    {
        if let Some(ref mut process_content) = self.process_content
            {
                if let Some(ref headers ) = self.headers
                    {
                        process_content.flush(&headers);
                    }
            }
    }
}
