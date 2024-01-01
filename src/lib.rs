use std::{fmt::Display, marker::PhantomData};

use acril::{Handler, Service};
use futures::{AsyncBufRead, AsyncWrite};
use pingu::{http::StatusCode, read_request, write_response, Request, Response};

pub trait ResponseError: Display {
    fn status_code(&self) -> StatusCode;

    fn to_response(&self) -> pingu::http::Result<pingu::Response> {
        pingu::Response::builder()
            .status(self.status_code())
            .body(pingu::Body::full(self.to_string().into()))
    }
}

impl ResponseError for std::io::Error {
    fn status_code(&self) -> StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}

pub struct Server<Io, H> {
    handler: H,
    io: Io,
}

impl<Io, H> Server<Io, H> {
    pub fn new(io: Io, handler: H) -> Self {
        Self { io, handler }
    }
}

impl<
        Io: AsyncBufRead + AsyncWrite + Unpin,
        H: Handler<Request, Response = Response, Context = HttpContext<H>>,
    > Server<Io, H>
{
    pub async fn run(&mut self) -> Result<(), H::Error>
    where
        H::Error: ResponseError + From<std::io::Error>,
    {
        loop {
            #[cfg(feature = "tracing")]
            tracing::debug!("reading request");
            let request = read_request(&mut self.io).await?;
            #[cfg(feature = "tracing")]
            tracing::debug!(?request);

            let close = request
                .headers()
                .get("Connection")
                .is_some_and(|conn| conn == "close");

            let mut cx = HttpContext::<H>(PhantomData);

            match self.handler.call(request, &mut cx).await {
                Ok(response) => {
                    #[cfg(feature = "tracing")]
                    tracing::debug!(?response, "Handler answered");
                    write_response(response, &mut self.io).await?;
                }
                Err(error) => {
                    if let Ok(response) = error.to_response() {
                        write_response(response, &mut self.io).await?;
                    }

                    return Err(error);
                }
            }

            if close {
                #[cfg(feature = "tracing")]
                tracing::debug!("Connection: closed");
                break;
            }
        }

        Ok(())
    }
}

pub struct HttpContext<H: Service<Context = Self>>(PhantomData<H>);
