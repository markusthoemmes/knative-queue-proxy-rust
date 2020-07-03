use envconfig::Envconfig;
use envconfig_derive::Envconfig;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Response, Server};
use lazy_static::lazy_static;
use std::convert::Infallible;
use std::net::SocketAddr;
use futures_intrusive::sync::Semaphore;

#[derive(Envconfig)]
pub struct Config {
    #[envconfig(from = "CONTAINER_CONCURRENCY", default = "0")]
    pub container_concurrency: usize,

    #[envconfig(from = "QUEUE_SERVING_PORT", default = "8012")]
    pub queue_serving_port: u16,
}

#[tokio::main]
async fn main() {
    lazy_static! {
        static ref CONFIG: Config = Config::init().expect("Failed to parse environment");
        static ref QUEUE: Semaphore = Semaphore::new(false, 1);
    }

    let make_svc = make_service_fn(|_| async {
        Ok::<_, Infallible>(service_fn(|_| async {
            QUEUE.acquire(1).await;

            Ok::<_, Infallible>(Response::new(Body::from("Hello world!")))
        }))
    });

    let addr = SocketAddr::from(([127, 0, 0, 1], CONFIG.queue_serving_port));
    let server = Server::bind(&addr).serve(make_svc);

    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}
