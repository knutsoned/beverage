//! A simple command line client that allows issuing queries to a remote Bevy
//! app via the BRP.

use std::process;

use anyhow::Result as AnyhowResult;
use argh::FromArgs;
use bevy::prelude::default;
use bevy::remote::builtin_verbs::{BrpQuery, BrpQueryRequest};
use bevy::remote::{BrpRequest, DEFAULT_PORT};
use http_body_util::BodyExt as _;
use hyper::client::conn::http1;
use hyper::header::HOST;
use hyper::Request;
use macro_rules_attribute::apply;
use serde_json::Value;
use smol::{net::TcpStream, Executor};
use smol_hyper::rt::FuturesIo;
use smol_macros::main;

/// TODO
#[derive(FromArgs)]
struct Args {
    /// the host to connect to
    #[argh(option, default = "\"127.0.0.1\".to_owned()")]
    host: String,
    /// the port to connect to
    #[argh(option, default = "DEFAULT_PORT")]
    port: u16,
    /// the full type names of the components to query for
    #[argh(positional, greedy)]
    components: Vec<String>,
}

/// The application entry point.
#[apply(main!)]
async fn main(executor: &Executor<'_>) -> AnyhowResult<()> {
    // Parse the arguments.
    let args: Args = argh::from_env();

    // Create the URL. We're going to need it to issue the HTTP request.
    let host_part = format!("{}:{}", args.host, args.port);
    let url = format!("https://{}/", host_part)
        .parse::<hyper::Uri>()
        .unwrap();

    // Create our `smol` TCP stream.
    let stream = TcpStream::connect(host_part).await.unwrap();

    // Create a HTTP 1.x connection.
    let (mut sender, connection) = http1::handshake::<_, String>(FuturesIo::new(stream))
        .await
        .unwrap();

    // Build the parameters to our BRP request. Include the full type names of
    // all the components, as specified on the command line.
    let brp_query_params = BrpQueryRequest {
        data: BrpQuery {
            components: args.components,
            ..default()
        },
        filter: default(),
    };

    let brp_query_params = match serde_json::to_value(&brp_query_params) {
        Ok(brp_query_params) => brp_query_params,
        Err(error) => die(&format!(
            "Failed to serialize request parameters: {}",
            error
        )),
    };

    // Build the request.
    let brp_request = BrpRequest {
        request: "QUERY".to_owned(),
        id: (0).into(),
        params: brp_query_params,
    };

    // Serialize the request.
    let brp_request = match serde_json::to_string(&brp_request) {
        Ok(brp_request) => brp_request,
        Err(error) => die(&format!("Failed to serialize request: {}", error)),
    };

    // Connect.
    executor
        .spawn(async move {
            if let Err(error) = connection.await {
                die(&format!("Failed to connect: {}", error));
            }
        })
        .detach();

    // We're connected, so build the HTTP request.
    let authority = url.authority().unwrap();
    let http_request = Request::builder()
        .uri(&url)
        .header(HOST, authority.as_str())
        .body(brp_request)
        .unwrap();

    let response = match sender.send_request(http_request).await {
        Ok(response) => response,
        Err(error) => die(&format!("Failed to send request: {}", error)),
    };

    let body = match response.collect().await {
        Ok(body) => body.to_bytes(),
        Err(error) => die(&format!("Failed to receive data: {}", error)),
    };

    let response: Value = match serde_json::from_slice(&body) {
        Ok(response) => response,
        Err(error) => die(&format!("Failed to parse response as JSON: {}", error)),
    };

    // Just print the JSON to stdout.
    println!("{}", serde_json::to_string_pretty(&response).unwrap());

    Ok(())
}

/// Exits with an error message.
fn die(message: &str) -> ! {
    eprintln!("{}", message);
    process::exit(1);
}
