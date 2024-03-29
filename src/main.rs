// config file is at config/config.toml

use std::fmt;
use serde::{Deserialize, Serialize};
use config::{Config, Source};
use tonic_lnd;

use lndbalancer::{calculate_htlc_max, calculate_fee_target};
use lndbalancer;
mod config;

use futures::stream::{self, StreamExt};


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


    let balancer_config = lndbalancer::Config {
        dynamic_fees: true,
        dynamic_fee_update_frequency: 100,
        dynamic_fee_intervals: 5,
        dynamic_fee_min: 100,
        dynamic_fee_max: 500,
    };


    let channels = client
        .lightning()
        .list_channels(tonic_lnd::lnrpc::ListChannelsRequest {
            active_only: false,
            ..Default::default()
        })
        .await
        .expect("failed to list channels").into_inner();

        println!("Before calling process_channels");
        process_channels(channels.channels, balancer_config).await;
        println!("After calling process_channels");

    // Print the response



    // We only print it here, note that in real-life code you may want to call `.into_inner()` on
    // the response to get the message.
    println!("{:#?}", info);
}

async fn process_channels(channels: Vec<tonic_lnd::lnrpc::Channel>, balancer_config: lndbalancer::Config) {
    // let channels = channels.into_inner().channels;
    println!("Channels: {:?}", channels.len());
    // Convert the iterator to a stream
    let channels_stream = stream::iter(channels);

    // Asynchronously process each channel
    channels_stream.for_each_concurrent(None, |c| async move {
        println!("{:?}", c);
        println!("-----------------");
        let fee_rate = calculate_fee_target(&c, &balancer_config).await.unwrap();
        let htlc_max = calculate_htlc_max(c, &balancer_config).await.unwrap();
        println!("Target: {:?}", target);
        println!("HTLC Max: {:?}", htlc_max);
        let channel_point = tonic_lnd::lnrpc::ChannelPoint::from(c.channel_point);
        tonic_lnd::lnrpc::PolicyUpdateRequest {
            scope: Some(tonic_lnd::lnrpc::policy_update_request::Scope::ChanPoint(channel_point)),
            base_fee_msat: Some(1000),
            fee_rate: fee_rate,
            time_lock_delta: Some(144),
            max_htlc_msat: Some(htlc_max),

            ..Default::default()
        };

        tonic_lnd::lnrpc::SetChannel

    }).await; // Wait for all asynchronous operations to complete
}