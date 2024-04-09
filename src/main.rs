use std::sync::Arc;
use tokio::sync::Mutex;
use futures::stream::{self, StreamExt};
use warp::Filter;

use lndbalancer::{calculate_htlc_max, calculate_fee_target};

mod config;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = config::Config::current();

    let health_route = warp::path!("health").map(|| warp::reply::json(&"OK"));
    let port = config.application_port;

    tokio::spawn(warp::serve(health_route).run(([127, 0, 0, 1], port)));

    loop {
        process_lnd().await?;
        tokio::time::sleep(tokio::time::Duration::from_secs(config.dynamic_fee_update_frequency)).await;
    }

}

async fn process_lnd() -> Result<(), Box<dyn std::error::Error>> {
    let settings = lndbalancer::config::Config::new();
    let settings_arced = Arc::new(settings.clone());
    let sources: Vec<lndbalancer::config::Source> = settings.sources;
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


    process_channels(client, channels.channels, settings_arced).await;
    Ok(())
}



async fn process_channels(client: Arc<Mutex<tonic_lnd::Client>>, channels: Vec<tonic_lnd::lnrpc::Channel>, balancer_config: Arc<lndbalancer::config::Config>) {
    // let channels = channels.into_inner().channels;
    println!("Channels: {:?}", channels.len());
    let client = Arc::new(client); // Wrap the client in an Arc

    // Convert the iterator to a stream
    let channels_stream = stream::iter(channels);

    // Asynchronously process each channel
    channels_stream.for_each_concurrent(None, move |c| {
        let client_clone = Arc::clone(&client); // Clone the Arc, not the client
        let config_clone = Arc::clone(&balancer_config); // Clone the Arc, not the config
        async move {
            println!("{:?}", c);
            println!("-----------------");

            let config = config_clone.as_ref();
            let fee_rate = calculate_fee_target(&c, &config).await.unwrap();
            let htlc_max = calculate_htlc_max(c.clone(), &config).await.unwrap();


            println!("Target: {:?}", fee_rate);
            println!("HTLC Max: {:?}", htlc_max);
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
            println!("{:?}", result);
        }

    }).await; // Wait for all asynchronous operations to complete

}