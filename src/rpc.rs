use std::str;
use anyhow::Error;
use reqwest::blocking::Client;
use reqwest::header::CONTENT_TYPE;
use serde::{Serialize, Deserialize, de::DeserializeOwned};
use url::{Url, ParseError};
use serde_json::{self, json};


pub struct ZClientBuilder {
    client: ZClient,
}

impl Default for ZClientBuilder {
    fn default() -> Self {
        ZClientBuilder {
            client: ZClient::default(),
        }
    }
}

impl ZClientBuilder {
    pub fn with_url(mut self, url: String) -> Result<Self, ParseError> {
        let _ = Url::parse(&url)?;
        self.client.url = url;
        Ok(self)
    }

    pub fn with_auth(mut self, user: String, password: Option<String>) -> Self {
        self.client.user = user;
        self.client.password = password;
        self
    }

    pub fn build(self) -> ZClient {
        self.client
    }
}

#[derive(Debug, Deserialize)]
pub struct ZResponse<T> {
    result: T,
    error: Option<String>,
    id: Option<i32>,
}

// FIXME: Manually implement PartialEq for testing.
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Tx {
    pub txid: String,
    pub amount: f32,
    pub memo: String,
    amount_zat: usize,
    outindex: usize,
    confirmations: usize,
    blockheight: usize,
    blockindex: usize,
    blocktime: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    jsindex: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    jsoutindex: Option<usize>,
    pub change: bool,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
/// Detailed information abut in-wallet transaction <txid>.
/// Result from "gettransaction" method.
pub struct WalletTx {
    pub status: String,
    pub txid: String,
    pub amount: f32,
    amount_zat: usize,
    confirmations: usize,
    blockhash: String,
    blockindex: usize,
    blocktime: usize,
    pub time: usize,
    pub timereceived: usize,
    hex: String,
    details: Vec<serde_json::Value>,
    vjoinsplit: Vec<serde_json::Value>,
}

impl WalletTx {
    fn mock() -> Self {
        WalletTx{
            status: "mined".to_owned(),
            txid: "foobarbaz".to_owned(),
            amount: 99.999,
            amount_zat: 99999999,
            confirmations: 9,
            blockhash: "9999999999999999999999".to_owned(),
            blockindex: 99,
            blocktime: 99999999,
            time: 99999999,
            timereceived: 99999999,
            hex: "999999999999999999999999999999999999999999999999999".to_owned(),
            details: vec![
                json!({
                    "account": "mocker",
                    "categoy": "received",
                    "amount": 99.999,
                    "amount_zat": 99999999,
                    "vout": 9
                }),
            ],
            vjoinsplit: vec![
                json!({
                    "anchor": "someref",
                    "nullifiers": serde_json::Value::Array(vec![]),
                    "commitments": serde_json::Value::Array(vec![]),
                    "macs": serde_json::Value::Array(vec![]),
                    "vpub_old": 9.999,
                    "vpub_new": 9.999,
                }),
            ],
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct ZRequest<T> {
    jsonrpc: String,
    method: String,
    params: Vec<T>,
}

pub struct ZRequestBuilder<T> {
    request: ZRequest<T>,
}

impl<T> Default for ZRequestBuilder<T> {
    fn default() -> Self {
        Self {
            request: ZRequest::default(),
        }
    }
}

impl<T> ZRequestBuilder<T> {
    pub fn jsonrpc(mut self, version: String) -> Self {
        self.request.jsonrpc = version;
        self
    }

    pub fn method(mut self, method: String) -> Self {
        self.request.method = method;
        self
    }

    pub fn params(mut self, params: Vec<T>) -> Self {
        self.request.params = params;
        self
    }

    pub fn build(self) -> ZRequest<T> {
        self.request
    }
}

impl<T> Default for ZRequest<T> {
    fn default() -> Self {
        Self {
            jsonrpc: "1.0".to_string(),
            method: "getbalance".to_string(),
            params: Vec::<T>::new(),
        }
    }
}

impl<T> ZRequest<T> {
    pub fn builder() -> ZRequestBuilder<T> {
        ZRequestBuilder::<T>::default()
    }
}

pub struct ZClient {
    c: Client,
    pub url: String,
    pub user: String,
    pub password: Option<String>,
}

impl Default for ZClient {
    fn default() -> Self {
        Self {
            c: Client::default(),
            url: "http://127.0.0.1:9999".to_string(),
            user: String::new(),
            password: None,
        }
    }
}

impl ZClient {
    pub fn builder() -> ZClientBuilder {
        ZClientBuilder::default()
    }

    fn send<S, T>(&self, req: ZRequest<S>) -> Result<ZResponse<T>, Error> 
    where S: Serialize + 'static, T: DeserializeOwned + 'static {
        let res = self.c.post(self.url.clone())
            .basic_auth(self.user.clone(), self.password.clone())
            .header(CONTENT_TYPE, "text/octet-stream")
            .body(json!(req).to_string())
            .send()?
            .json::<ZResponse<T>>()?;
        Ok(res)
    }
    
    /* Zcash RPC API implementation */

    pub fn getbalance(&self) -> Result<f32, Error> {
        let req = ZRequest::<String>::builder()
            .method("getbalance".to_string())
            .build();
            
        let res: ZResponse<f32> = self.send::<String, f32>(req)?;
        Ok(res.result)
    }

    pub fn z_listaddresses(&self) -> Result<Vec<String>, Error> {
        let req = ZRequest::<String>::builder()
            .method("z_listaddresses".to_string())
            .build();
        let res: ZResponse<Vec<String>> = self.send::<String, Vec<String>>(req)?;
        Ok(res.result)
    }

    pub fn z_listreceivedbyaddress(&self, addr: &str) -> Result<Vec<(Tx, String)>, Error> {
        let req = ZRequest::<String>::builder()
            .method("z_listreceivedbyaddress".to_string())
            .params(vec![addr.to_owned()])
            .build();
        let res: ZResponse<Vec<Tx>> = self.send::<String, Vec<Tx>>(req)?;
        let result_pairs = res.result.iter().map(|tx| (tx.clone(), addr.to_string())).collect::<Vec<_>>();
        Ok(result_pairs)
    }

    pub fn gettransaction(&self, txid: &str) -> Result<WalletTx, Error> {
        let req = ZRequest::<String>::builder()
            .method("gettransaction".to_string())
            .params(vec![txid.to_owned()])
            .build();
        let res: ZResponse<WalletTx> = self.send::<String, WalletTx>(req)?;
        Ok(res.result)
    }

    pub fn z_sendmany(
        &self, 
        sender_addr: &str,
        receiver_addr: &str, 
        amount: f32,
        memo: String
    ) -> Result<String, Error> {
        let req = ZRequest::<serde_json::Value>::builder()
            .method("z_sendmany".to_string())
            .params(vec![
                serde_json::Value::String(sender_addr.to_string()),
                serde_json::Value::Array(vec![
                    serde_json::json!({
                        "address": receiver_addr,
                        "amount": amount,
                        "memo": memo
                    }),
                ])
            ])
            .build();
        let res = self.send::<serde_json::Value, String>(req)?;
        Ok(res.result)
    }
}

#[cfg(test)]
mod tests {
    use crate::hex::*;
    use super::*;
    use httpmock::MockServer;
    use httpmock::Method::POST;
    use serde_json::{self, json, Value::Null};
    
    #[test]
    fn test_getbalance() {
        let server = MockServer::start();
        let getbalance_mock = server.mock(|when, then| {
            when.method(POST)
                .path("/getbalance")
                .body(json!({
                    "jsonrpc": "1.0", 
                    "method": "getbalance", 
                    "params": serde_json::Value::Array(vec![])
                }).to_string());
            then.status(200)
                .header("Content-Type", "application/json")
                .body(json!({
                    "result": 2.5,
                    "error": Null,
                    "id": Null,
                }).to_string());
        });

        let balance = ZClient::builder()
            .with_url(server.url("/getbalance"))
            .expect("Failed to parse URL")
            .with_auth("user".to_string(), Some("pass".to_string()))
            .build()
            .getbalance()
            .expect("Failed to build client");

        assert!(balance == 2.5);
    }

    #[test]
    fn test_z_listaddresses() {
        let server = MockServer::start();

        let listaddresses_mock = server.mock(|when, then| {
            when.method(POST)
                .path("/z_listaddresses")
                .body(json!({
                    "jsonrpc": "1.0", 
                    "method": "z_listaddresses", 
                    "params": serde_json::Value::Array(vec![])
                }).to_string());
            then.status(200)
                .header("Content-Type", "application/json")
                .body(json!({
                    "result": vec!["z_addr1", "z_addr2", "z_addr3"],
                    "error": Null,
                    "id": Null,
                }).to_string());
        });

        let addresses = ZClient::builder()
            .with_url(server.url("/z_listaddresses"))
            .expect("Failed to parse URL")
            .with_auth("user".to_string(), Some("pass".to_string()))
            .build()
            .z_listaddresses()
            .expect("Failed to build client");

        assert!(addresses == vec![
            "z_addr1".to_string(), 
            "z_addr2".to_string(), 
            "z_addr3".to_string(),
        ]);
    }

    #[test]
    fn test_z_listreceivedbyaddress() {
        let expected = Tx{
            txid: "90ac85f44c412b43db85d2c52e1ccafeea6385661f4b58cb8dd372cac73d1978".to_owned(),
            amount: 0.01,
            amount_zat: 1000000,
            memo: "68656c6c6f207a63617368000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000".to_owned(),
            outindex: 0,
            confirmations: 7651,
            blockheight: 1400579,
            blockindex: 1,
            blocktime: 1620543097,
            change: false,
            jsindex: None,
            jsoutindex: None,
        };

        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(POST)
                .path("/")
                .body(json!({
                    "jsonrpc": "1.0", 
                    "method": "z_listreceivedbyaddress", 
                    "params": vec!["z_addr"]
                }).to_string());
            then.status(200)
                .header("Content-Type", "application/json")
                .body(json!({
                    "result": vec![
                        json!(expected)
                    ],
                    "error": Null,
                    "id": Null
                }).to_string());
        });

        let txs: Vec<(Tx, String)> = ZClient::builder()
            .with_url(server.url("/"))
            .expect("Failed to parse URL")
            .with_auth("user".to_string(), Some("pass".to_string()))
            .build()
            .z_listreceivedbyaddress("z_addr")
            .expect("Failed to build client");
        
        let (tx, addr) = &txs[0];
        assert!(tx == &expected);
        assert!(addr == "z_addr");
        assert!(hex_to_string(&tx.memo).unwrap().starts_with("hello zcash"));
    }

    #[test]
    fn test_gettransaction() {
        let expected_wtx = WalletTx::mock();
        let txid = expected_wtx.txid.clone();
        let data = json!({
            "jsonrpc": "1.0", 
            "method": "gettransaction", 
            "params": vec![txid.clone()]
        }).to_string();

        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(POST)
                .path("/")
                .body(data);
            then.status(200)
                .header("Content-Type", "application/json")
                .body(json!({
                    "result": expected_wtx,
                    "error": Null,
                    "id": Null,
                }).to_string());
        });

        let wtx = ZClient::builder()
            .with_url(server.url("/"))
            .expect("Failed to parse URL")
            .with_auth("user".to_string(), Some("pass".to_string()))
            .build()
            .gettransaction(&txid)
            .expect("Failed to build client");

        assert!(wtx == expected_wtx);
    }

    #[test]
    fn test_z_sendmany() {
        let expected_opid = "opid-f757ae55-530b-4499-a1e2-12fd32c96a36";
        let memo = "68656c6c6f207a63617368";
        let amount: f32 = 2.99;
        let sender = "sender_addr";
        let recv = "recv_addr";

        let data = json!({
            "jsonrpc": "1.0", 
            "method": "z_sendmany", 
            "params": vec![
                serde_json::Value::String(sender.to_string()),
                serde_json::Value::Array(vec![
                    serde_json::json!({
                        "address": recv,
                        "amount": amount,
                        "memo": memo
                    }),
                ])
            ]
        }).to_string();

        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(POST)
                .path("/")
                .body(data);
            then.status(200)
                .header("Content-Type", "application/json")
                .body(json!({
                    "result": expected_opid,
                    "error": Null,
                    "id": Null,
                }).to_string());
        });

        let opid = ZClient::builder()
            .with_url(server.url("/"))
            .expect("Failed to parse URL")
            .with_auth("user".to_string(), Some("pass".to_string()))
            .build()
            .z_sendmany(sender, recv, amount, memo.to_string())
            .expect("Failed to build client");

        assert!(opid == expected_opid);
    }
}

