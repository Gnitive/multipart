/// Use for easy construct boundary inside `MultipartParser`
pub trait BoundaryBuilder
{
    fn append_crlf<'a>(&'a mut self) -> &'a mut Self;
    fn append_prelude<'a>(&'a mut self) -> &'a mut Self;
    fn append_boundary<'a>(&'a mut self, boundary: &Vec<u8>) -> &'a mut Self;
}

impl BoundaryBuilder for Vec<u8>
{
    fn append_crlf<'a>(&'a mut self) -> &'a mut Vec<u8>
    {
        self.push('\r' as u8);
        self.push('\n' as u8);
        self
    }

    fn append_prelude<'a>(&'a mut self) -> &'a mut Vec<u8>
    {
        let dash =  '-' as u8;
        self.push(dash);
        self.push(dash);
        self
    }

    fn append_boundary<'a>(&'a mut self, boundary: &Vec<u8>) -> &'a mut Vec<u8>
    {
        self.extend(boundary);
        self
    }
}