use std::{env, time::Duration};

use anyhow::Ok;
use errors::Error;
use model::Subdomain;
use reqwest::{blocking::Client, redirect};

mod common_ports;
mod errors;
mod model;
mod ports;
mod subdomains;

fn main() -> Result<(), anyhow::Error> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        return Err(Error::CliUsage.into());
    }

    let target = args[1].as_str();

    let http_timeout = Duration::from_secs(5);
    let http_client = Client::builder()
        .redirect(redirect::Policy::limited(4))
        .timeout(http_timeout)
        .build()?;

    let scan_result: Vec<Subdomain> = subdomains::enumerate(&http_client, target)
        .unwrap()
        .into_iter()
        .map(ports::scan_ports)
        .collect();

    for subdomain in scan_result {
        println!("{}:", &subdomain.domain);
        for port in &subdomain.open_ports {
            println!("    {}", port.port);
        }

        println!();
    }

    Ok(())
}
