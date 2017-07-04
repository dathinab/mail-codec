use chrono;
use ascii::AsciiStr;

use error::*;
use codec::{ SmtpDataEncoder, SmtpDataEncodable };

pub struct DateTime( chrono::DateTime<chrono::Utc> );

impl DateTime {
    fn new<TZ: chrono::TimeZone>( date_time: chrono::DateTime<TZ>) -> DateTime {
        DateTime( date_time.with_timezone( &chrono::Utc ) )
    }
}

impl SmtpDataEncodable for DateTime {
    fn encode( &self, encoder: &mut SmtpDataEncoder ) -> Result<()> {
        let as_str = self.0.to_rfc2822();
        let ascii = unsafe { AsciiStr::from_ascii_unchecked( &*as_str ) };
        encoder.write_str( ascii );
        Ok( () )
    }
}