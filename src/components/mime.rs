use std::error::Error;
use std::fmt;

use mime;
use ascii::AsciiStr;

use error::*;
use codec::{ MailEncoder, MailEncodable };

pub use mime::Mime;


impl MailEncodable for mime::Mime {

    fn encode<E>( &self, encoder:  &mut E ) -> Result<()>
        where E: MailEncoder
    {
        let res = self.to_string();
        //TODO can mime be non ascii??, e.g. utf8 file names?
        encoder.write_str( AsciiStr::from_ascii( &*res ).unwrap() );
        //OPTIMIZE: as far as I know mime can not be non-ascii
        //encoder.write_str( unsafe { AsciiStr::from_ascii_unchecked( &*res ) } );
        Ok( () )
    }
}


//UPSTREAM(mime): open an issue that FromStrError does not implement Error
#[derive(Debug)]
pub struct MimeFromStrError( pub mime::FromStrError );

impl fmt::Display for MimeFromStrError {
    fn fmt( &self, fter: &mut fmt::Formatter ) -> fmt::Result {
        <MimeFromStrError as fmt::Debug>::fmt( self, fter )
    }
}
impl Error for MimeFromStrError {
    fn description(&self) -> &str {
        "parsing mime from str failed"
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use codec::test_utils::*;

    ec_test!{simple,{
        let mime: Mime = "text/wtf;charset=utf8;random=alot".parse().unwrap();
        mime
    } => ascii => [
        LinePart("text/wtf;charset=utf8;random=alot")
    ]}

    //TODO test international extension:
    // 0. check if relevant for mime (it's relevant for some other part based on the _same grammar_)
    // 1. splitting parameters over multiple lines
    // 2. non ascii parameters encoded with charset and language information
}