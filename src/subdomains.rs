use crate::{
    errors::Error,
    model::{CrtShEntry, Subdomain},
    url::{Protocol, Url},
};
use futures::{stream, StreamExt};
use reqwest::Client;
use std::{collections::HashSet, time::Duration};
use trust_dns_resolver::{
    config::{ResolverConfig, ResolverOpts},
    name_server::{GenericConnection, GenericConnectionProvider, TokioRuntime},
    AsyncResolver,
};

type DnsRsolver = AsyncResolver<GenericConnection, GenericConnectionProvider<TokioRuntime>>;

pub async fn enumerate(http_client: &Client, target: &str) -> Result<Vec<Subdomain>, Error> {
    let url = Url::new()
        .set_protocol(Protocol::Https)
        .set_domain("api.certspotter.com")
        .set_path("/v1/issuances")
        .add_param("include_domains", "true")
        .add_param("expand", "dns_names")
        .add_param("expand", "issuer")
        .add_param("expand", "revocation")
        .add_param("expand", "problem_reporting")
        .add_param("expand", "cert_der")
        .add_param("domain", target)
        .add_param("include_subdomains", "true")
        .build();

    println!("Attempting scan on {}", target);

    let entries: Vec<CrtShEntry> = http_client.get(url).send().await?.json().await?;

    let mut opts = ResolverOpts::default();
    opts.timeout = Duration::from_secs(4);

    let dns_resolver = AsyncResolver::tokio(ResolverConfig::default(), opts)
        .expect("subdomain resolver: building DNS client");

    let mut subdomains: HashSet<String> = entries
        .into_iter()
        .flat_map(|entry| entry.dns_names)
        .filter(|subdomain: &String| subdomain != target)
        .filter(|subdomain: &String| !subdomain.contains('*'))
        .collect();

    subdomains.insert(target.to_string());

    let subdomains: Vec<Subdomain> = stream::iter(subdomains.into_iter())
        .map(|domain| Subdomain {
            domain,
            open_ports: Vec::new(),
        })
        .filter_map(|subdomain| {
            let dns_resolver: DnsRsolver = dns_resolver.clone();
            async move {
                if resolves(&dns_resolver, &subdomain).await {
                    Some(subdomain)
                } else {
                    None
                }
            }
        })
        .collect()
        .await;

    Ok(subdomains)
}

pub async fn resolves(dns_resolver: &DnsRsolver, domain: &Subdomain) -> bool {
    dns_resolver.lookup_ip(domain.domain.as_str()).await.is_ok()
}
