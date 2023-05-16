use std::{
    env,
    time::{Duration, Instant},
};

use anyhow::Ok;
use errors::Error;
use futures::{stream, StreamExt};
use model::Subdomain;
use reqwest::{redirect, Client};

use crate::{ports::scan_ports, subdomains::enumerate};

mod common_ports;
mod errors;
mod model;
mod ports;
mod subdomains;
mod url;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        return Err(Error::CliUsage.into());
    }

    let target = args[1].as_str().trim();

    let http_timeout = Duration::from_secs(10);
    let http_client = Client::builder()
        .redirect(redirect::Policy::limited(4))
        .timeout(http_timeout)
        .build()?;

    let ports_concurrency = 200;
    let subdomains_concurrency = 100;

    let scan_start = Instant::now();
    let subdomains = enumerate(&http_client, target).await?;

    let scan_result = stream::iter(subdomains.into_iter())
        .map(|subdomain| scan_ports(ports_concurrency, subdomain))
        .buffer_unordered(subdomains_concurrency)
        .collect::<Vec<Subdomain>>()
        .await;

    let scan_duration = scan_start.elapsed();

    println!("Scan took {:?}", scan_duration);

    for subdomain in scan_result {
        println!("{}:", &subdomain.domain);
        for port in &subdomain.open_ports {
            println!("    {}", port.port);
        }
    }

    println!("Finished scan");

    Ok(())
}
