use crate::{
    errors::Error,
    model::{CrtShEntry, Subdomain},
};
use reqwest::blocking::Client;
use std::{collections::HashSet, time::Duration};
use trust_dns_resolver::{
    config::{ResolverConfig, ResolverOpts},
    Resolver,
};

pub fn enumerate(http_client: &Client, target: &str) -> Result<Vec<Subdomain>, Error> {
    let url = format!("https://api.certspotter.com/v1/issuances?domain={}&include_subdomains=true&expand=dns_names&expand=issuer&expand=revocation&expand=problem_reporting&expand=cert_der", target);
    println!("Attempting scan on {}", target);
    let entries: Vec<CrtShEntry> = http_client.get(url).send()?.json()?;

    let mut subdomains: HashSet<String> = entries
        .into_iter()
        .flat_map(|entry| entry.dns_names)
        .filter(|subdomain: &String| subdomain != target)
        .filter(|subdomain: &String| !subdomain.contains('*'))
        .collect();

    subdomains.insert(target.to_string());

    let subdomains: Vec<Subdomain> = subdomains
        .into_iter()
        .map(|domain| Subdomain {
            domain,
            open_ports: Vec::new(),
        })
        .filter(resolves)
        .collect();

    Ok(subdomains)
}

pub fn resolves(domain: &Subdomain) -> bool {
    let mut opts = ResolverOpts::default();
    opts.timeout = Duration::from_secs(4);

    let dns_resolver = Resolver::new(ResolverConfig::default(), opts)
        .expect("subdomain resolver: building DNS client");
    dns_resolver.lookup_ip(domain.domain.as_str()).is_ok()
}
