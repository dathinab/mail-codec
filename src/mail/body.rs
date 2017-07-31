use std::fmt;

use futures::future::BoxFuture;
use futures::Async;

use error::*;
use codec::transfer_encoding::TransferEncodedFileBuffer;


pub type FutureBuf = BoxFuture<TransferEncodedFileBuffer, Error>;

#[derive(Debug)]
pub struct Body {
    body: InnerBody
}


enum InnerBody {
    /// a futures resolving to a buffer
    Future(FutureBuf),
    /// store the value the FileBufferFuture resolved to
    Value(TransferEncodedFileBuffer),
    /// if the FileBufferFuture failed, we don't have anything
    /// to store, but have not jet dropped the mail it is
    /// contained within, so we still need a value for InnerBody
    ///
    /// this variations should only ever occure between
    /// a call to a BodyFuture in `MailFuture::poll` resolved to
    /// an Error and the Body/Mail being dropped (before `MailFuture::poll`
    /// exists)
    Failed
}

impl fmt::Debug for InnerBody {
    fn fmt( &self, fter: &mut fmt::Formatter ) -> fmt::Result {
        use self::InnerBody::*;
        match *self {
            Future( .. ) => {
                write!( fter, "Future(..)" )
            },
            Value( ref buf ) => {
                fter.debug_tuple("Value")
                    .field(buf)
                    .finish()
            },
            Failed => {
                write!( fter, "Failed" )
            }
        }
    }
}



impl Body {

    /// creates a new body based on a already transfer-encoded buffer
    pub fn new_future(data: FutureBuf) -> Body {
        Body {
            body: InnerBody::Future( data )
        }
    }

    /// creates a new body based on a already transfer-encoded buffer
    pub fn new_resolved( data: TransferEncodedFileBuffer ) -> Body {
        Body {
            body: InnerBody::Value( data )
        }
    }

    /// returns a reference to the buffer if
    /// the buffer is directly contained in the Body,
    /// i.e. the Futures was resolved _and_ the body
    /// is aware of it
    ///
    pub fn file_buffer_ref(&self ) -> Option<&TransferEncodedFileBuffer> {
        use self::InnerBody::*;
        match self.body {
            Value( ref value ) => Some( value ),
            _ => None
        }
    }

    /// polls the body for completation by calling `Futures::poll` on the
    /// contained future
    ///
    /// returns:
    /// - Ok(Some),  if the future was already completed in the past
    /// - Ok(Some),* if polll results in Ready, also the contained future
    ///              will be replaced by the value it resolved to
    /// - Ok(None),  if the future is not ready yet
    /// - Err(),     if the future resolved to a err in a previous call to
    ///              poll_body, note that the error the future resolved to
    ///              is no longer available
    /// - Err(),*    if the future resolves to an Error, the contained future
    ///              will be removed, `chain_err` will be used to include
    ///              the error in the error_chain
    pub fn poll_body( &mut self ) -> Result<Option<&TransferEncodedFileBuffer>> {
        use self::InnerBody::*;
        let new_body;
        match self.body {
            Failed =>
                bail!( ErrorKind::BodyFutureResolvedToAnError ),
            Value( ref buffer ) =>
                return Ok( Some( buffer ) ),
            Future( ref mut future ) => {
                match future.poll() {
                    Ok( Async::NotReady ) =>
                        return Ok( None ),
                    Ok( Async::Ready( buffer ) ) =>
                        new_body = Ok( Some( buffer ) ),
                    Err( e ) =>
                        new_body = Err( e )
                }
            },
        }

        match new_body {
            Ok( None ) => Ok( None ),
            Ok( Some( buffer ) ) => {
                self.body = Value( buffer );
                Ok( self.file_buffer_ref() )
            }
            Err( e ) => {
                self.body = Failed;
                Err( e )
            }
        }
    }
}


impl From<FutureBuf> for Body {
    fn from(fut: FutureBuf) -> Self {
        Self::new_future( fut )
    }
}

impl From<TransferEncodedFileBuffer> for Body {
    fn from(data: TransferEncodedFileBuffer) -> Self {
        Self::new_resolved( data )
    }
}