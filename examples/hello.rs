use acril::{Handler, Service};
use acril_web::{HttpContext, Server};
use async_compat::CompatExt;
use pingu::{Body, Request, Response};
use tokio::net::TcpListener;

pub struct HelloHandler;

impl Service for HelloHandler {
    type Error = std::io::Error;
    type Context = HttpContext<Self>;
}

impl Handler<Request> for HelloHandler {
    type Response = Response;
    async fn call(
        &mut self,
        _request: Request,
        _cx: &mut Self::Context,
    ) -> Result<Self::Response, Self::Error> {
        Ok(Response::builder()
            .status(200)
            .header("Content-Type", "text/html")
            .body(Body::full(String::from("<!doctype html><html><head><title>Hello world!</title></head><body><h1>Modify <code>examples/hello.rs</code> to add more stuff!</h1></body></html>").into()))
            .unwrap())
    }
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    #[cfg(feature = "tracing")]
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let listener = TcpListener::bind("0.0.0.0:3000").await?;

    #[cfg(feature = "tracing")]
    tracing::info!("Listening at :3000");

    loop {
        let (connection, addr) = listener.accept().await?;

        println!("{addr} connected");

        tokio::spawn(async move {
            if let Err(e) = Server::new(
                futures::io::BufReader::new(connection.compat()),
                HelloHandler,
            )
            .run()
            .await
            {
                #[cfg(feature = "tracing")]
                tracing::error!(error = ?e);
            }
        });
    }
}
