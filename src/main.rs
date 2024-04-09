use std::sync::Arc;
use tokio::sync::Mutex;
use futures::stream::{self, StreamExt};
use warp::Filter;
use tracing_subscriber;
use tracing::{info, debug, trace};

use lndbalancer::{calculate_htlc_max, calculate_fee_target};

mod config;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    info!("Starting LND Balancer");
    let loaded_config = config::Config::new();
    loaded_config.make_current();

    let config = config::Config::current();

    info!("Config: {:?}", config);

    let health_route = warp::path!("health").map(|| warp::reply::json(&"OK"));
    let port = config.application_port;

    tokio::spawn(warp::serve(health_route).run(([127, 0, 0, 1], port)));

    loop {

        process_lnd().await?;
        tokio::time::sleep(tokio::time::Duration::from_secs(config.dynamic_fee_update_frequency)).await;
    }

}

async fn process_lnd() -> Result<(), Box<dyn std::error::Error>> {
    let settings = config::Config::current();

    let sources: Vec<config::Source> = settings.sources.clone();
    if sources.is_empty() {
        panic!("No sources found in the configuration");
    }
    let lnd = sources.first().unwrap().clone();
    let endpoint = lnd.endpoint.to_string();
    let macaroon = lnd.macaroon.to_string();
    let cert = lnd.cert.to_string();
    let client_result = tonic_lnd::connect(endpoint, cert, macaroon).await?;

    let client = Arc::new(Mutex::new(client_result)); // client_result is directly used with `?` above

    let mut client_guard = client.lock().await; // Lock the client to avoid deadlock
    let channels = client_guard
        .lightning()
        .list_channels(tonic_lnd::lnrpc::ListChannelsRequest {
            active_only: false,
            ..Default::default()
        })
        .await
        .expect("failed to list channels").into_inner();

    drop(client_guard); // Drop the client to avoid deadlock (since we are not using it anymore


    process_channels(client, channels.channels).await;
    Ok(())
}



async fn process_channels(client: Arc<Mutex<tonic_lnd::Client>>, channels: Vec<tonic_lnd::lnrpc::Channel>) {
    debug!("Channels: {:?}", channels.len());
    let client = Arc::new(client); // Wrap the client in an Arc

    // Convert the iterator to a stream
    let channels_stream = stream::iter(channels);

    // Asynchronously process each channel
    channels_stream.for_each_concurrent(None, move |c| {
        let client_clone = Arc::clone(&client); // Clone the Arc, not the client
        async move {
            debug!("Processing Channel: {:?}", c.channel_point);
            trace!("{:?}", c);

            let config = lndbalancer::config::Config::current();
            let fee_rate = calculate_fee_target(&c, &config).await.unwrap();
            let htlc_max = calculate_htlc_max(c.clone(), &config).await.unwrap();

            debug!("Target: {:?}", fee_rate);
            debug!("HTLC Max: {:?}", htlc_max);
            let split_channel_point: Vec<&str> = c.channel_point.split(':').collect();

            // Directly accessing the elements since we know the format
            let funding_txid_str = split_channel_point[0]; // This is already &str, no need to clone and unwrap
            let output_index_str = split_channel_point[1]; // This is also &str

            let channel_point = tonic_lnd::lnrpc::ChannelPoint {
                output_index: output_index_str.parse().expect("Output index should be a valid u32 number"),
                funding_txid: Some(tonic_lnd::lnrpc::channel_point::FundingTxid::FundingTxidStr(funding_txid_str.to_string())),
            };
            // let channel_point = tonic_lnd::lnrpc::ChannelPoint::from(&c.channel_point).expect("failed to parse channel point");
            let request = tonic_lnd::lnrpc::PolicyUpdateRequest {
                scope: Some(tonic_lnd::lnrpc::policy_update_request::Scope::ChanPoint(channel_point)),
                base_fee_msat: 1000,
                fee_rate,
                time_lock_delta: 144,
                max_htlc_msat: htlc_max,

                ..Default::default()
            };

            let result = client_clone.lock().await.lightning().update_channel_policy(request).await.expect("failed to update channel policy");
            debug!("{:?}", result);
        }

    }).await; // Wait for all asynchronous operations to complete
    info!("All channels processed");
}