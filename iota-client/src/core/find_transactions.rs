use crate::error::Result;
use bee_ternary::T3B1Buf;
use bee_transaction::bundled::{Address, BundledTransactionField, Tag};
use crypto::hashes::ternary::Hash;

use crate::response::{FindTransactionsResponse, FindTransactionsResponseBuilder};
use crate::Client;

/// Builder to construct findTransactions API
#[derive(Debug)]
pub struct FindTransactionsBuilder<'a> {
    client: &'a Client,
    bundles: Option<Vec<String>>,
    addresses: Option<Vec<String>>,
    tags: Option<Vec<String>>,
    approvees: Option<Vec<String>>,
}

impl<'a> FindTransactionsBuilder<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self {
            client,
            bundles: Default::default(),
            addresses: Default::default(),
            tags: Default::default(),
            approvees: Default::default(),
        }
    }

    /// Add bundle hashes to search for
    pub fn bundles(mut self, bundles: &[Hash]) -> Self {
        self.bundles = Some(
            bundles
                .iter()
                .map(|h| {
                    (*h).encode::<T3B1Buf>()
                        .iter_trytes()
                        .map(char::from)
                        .collect::<String>()
                })
                .collect(),
        );
        self
    }

    /// Add tags to search for
    pub fn tags(mut self, tags: &[Tag]) -> Self {
        self.tags = Some(
            tags.iter()
                .map(|h| {
                    h.to_inner()
                        .encode::<T3B1Buf>()
                        .iter_trytes()
                        .map(char::from)
                        .collect::<String>()
                })
                .collect(),
        );
        self
    }

    /// Add child transactions to search for
    pub fn approvees(mut self, approvees: &[Hash]) -> Self {
        self.approvees = Some(
            approvees
                .iter()
                .map(|h| {
                    h.to_inner()
                        .encode::<T3B1Buf>()
                        .iter_trytes()
                        .map(char::from)
                        .collect::<String>()
                })
                .collect(),
        );
        self
    }

    /// Add addresses to search for (do not include the checksum)
    pub fn addresses(mut self, addresses: &[Address]) -> Self {
        self.addresses = Some(
            addresses
                .iter()
                .map(|h| {
                    h.to_inner()
                        .encode::<T3B1Buf>()
                        .iter_trytes()
                        .map(char::from)
                        .collect::<String>()
                })
                .collect(),
        );
        self
    }

    /// Send findTransactions request
    pub async fn send(self) -> Result<FindTransactionsResponse> {
        let mut body = json!({
            "command": "findTransactions",
        });

        if let Some(bundles) = self.bundles {
            body["bundles"] = json!(bundles);
        }

        if let Some(addresses) = self.addresses {
            body["addresses"] = json!(addresses);
        }

        if let Some(tags) = self.tags {
            body["tags"] = json!(tags);
        }

        if let Some(approvees) = self.approvees {
            body["approvees"] = json!(approvees);
        }

        let client = self.client;
        let res: FindTransactionsResponse = match &client.permanode {
            Some(url) => {
                let body_ = body.clone();
                let body2 = body.clone();
                let res: FindTransactionsResponseBuilder = response!(client, body_, url);
                let mut hashes: Vec<Hash> = Vec::new();
                let mut permanode_response: FindTransactionsResponse = res.build().await?;
                loop {
                    hashes.extend(&permanode_response.hashes);
                    match permanode_response.hints {
                        None => break,
                        Some(hints) => {
                            if hints.paging_state.is_none() {
                                break;
                            }
                            let mut body = json!({
                                "command": "findTransactions",
                            });
                            body["hints"] = json!(vec![hints]);
                            let res: FindTransactionsResponseBuilder = response!(client, body, url);
                            permanode_response = res.build().await?;
                        }
                    }
                }
                // fallback to normal node if permanode didn't return anything
                if hashes.is_empty() {
                    let res: FindTransactionsResponseBuilder = response!(client, body2);
                    res.build().await?
                } else {
                    FindTransactionsResponse {
                        hashes,
                        hints: None,
                    }
                }
            }
            None => {
                let res: FindTransactionsResponseBuilder = response!(client, body);
                res.build().await?
            }
        };
        Ok(res)
    }
}