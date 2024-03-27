// config file is at config/config.toml

use std::fmt;
use serde::{Deserialize, Serialize};
use config::{Config, Source};

mod config;


#[tokio::main]
async fn main() {
    // Load the configuration
    let config = Config::new();

    let sources: Vec<Source> = config.sources;

    // // Connecting to LND requires only address, cert file, and macaroon file
    let lnd = sources.get(0).unwrap().clone();
    let endpoint = lnd.endpoint.to_string();
    let macaroon = lnd.macaroon.to_string();
    let cert = lnd.cert.to_string();
    println!("endpoint: {:?}", endpoint);
    println!("cert: {:?}", cert);

    let mut client = tonic_lnd::connect(endpoint, cert, macaroon)
        .await
        .expect("failed to connect");

    let info = client
        .lightning()
        // All calls require at least empty parameter
        .get_info(tonic_lnd::lnrpc::GetInfoRequest {})
        .await
        .expect("failed to get info");

    // We only print it here, note that in real-life code you may want to call `.into_inner()` on
    // the response to get the message.
    println!("{:#?}", info);
}