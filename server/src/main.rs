use serde::Deserialize;
use tokio_postgres::{NoTls, Error};
use tokio::signal;
use chrono::{TimeZone, Utc};
use reqwest;
use tokio::sync::broadcast;
use tokio::sync::Mutex;
use std::sync::Arc;
use futures_util::SinkExt;
use futures_util::StreamExt;
use tokio_tungstenite::accept_async;
use tokio::net::TcpListener;

#[derive(Deserialize, Debug)]
struct ApiResponse {
    height: i32,
    time: Option<i64>,  // UNIX timestamp
}

#[derive(Deserialize, Debug)]
struct CoinGeckoResponse {
    bitcoin: BitcoinPrice,
}

#[derive(Deserialize, Debug)]
struct BitcoinPrice {
    usd: f64,
}

use serde::{Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct DataPayload {
    height: i32,
    price: f64,
    timestamp: i64,
}

async fn fetch_block_height() -> Result<ApiResponse, reqwest::Error> {
    let url = "https://blockchain.info/latestblock";
    let response = reqwest::get(url).await?.json::<ApiResponse>().await?;
    Ok(response)
}

async fn fetch_bitcoin_price() -> Result<f64, reqwest::Error> {
    let url = "https://api.coingecko.com/api/v3/simple/price?ids=bitcoin&vs_currencies=usd";
    let response = reqwest::get(url).await?.json::<CoinGeckoResponse>().await?;
    Ok(response.bitcoin.usd)
}

async fn establish_connection() -> Result<tokio_postgres::Client, Error> {
    dotenv::dotenv().ok(); // Load environment variables
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let (client, connection) = tokio_postgres::connect(&database_url, NoTls).await?;
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Connection error: {}", e);
        }
    });
    Ok(client)
}

async fn insert_block_height(client: &tokio_postgres::Client, height: i32, timestamp: i64) -> Result<(), Error> {
    let timestamp_tz = Utc.timestamp_opt(timestamp, 0).single().expect("Invalid timestamp");

    let rows = client.query(
        "SELECT 1 FROM blocks WHERE height = $1",
        &[&height]
    ).await?;

    if rows.is_empty() {
        client.execute(
            "INSERT INTO blocks (height, timestamp) VALUES ($1, $2)",
            &[&height, &timestamp_tz]
        ).await?;
        println!("Inserted block height: {}, timestamp: {}", height, timestamp_tz);
    } else {
        println!("Block height {} already exists", height);
    }

    Ok(())
}

async fn insert_bitcoin_price(client: &tokio_postgres::Client, price: f64, timestamp: i64) -> Result<(), Error> {
    let timestamp_tz = Utc.timestamp_opt(timestamp, 0).single().expect("Invalid timestamp");

    client.execute(
        "INSERT INTO bitcoin_prices (price, timestamp) VALUES ($1, $2)",
        &[&price, &timestamp_tz]
    ).await?;
    println!("Inserted Bitcoin price: ${}, timestamp: {}", price, timestamp_tz);

    Ok(())
}

async fn run_ingestion_loop(client: tokio_postgres::Client, tx: Arc<Mutex<broadcast::Sender<String>>>) {
    loop {
        // Fetch and insert block height
        match fetch_block_height().await {
            Ok(api_response) => {
                if let Some(time) = api_response.time {
                    let payload = DataPayload {
                        height: api_response.height,
                        price: 0.0, // Placeholder for Bitcoin price
                        timestamp: time,
                    };
                    let data = serde_json::to_string(&payload).unwrap();
                    tx.lock().await.send(data).unwrap();

                    // Insert block height into database
                    if let Err(e) = insert_block_height(&client, api_response.height, time).await {
                        eprintln!("Failed to insert block height: {}", e);
                    }
                }
            },
            Err(e) => eprintln!("Failed to fetch block height: {}", e),
        }

        // Fetch and insert Bitcoin price
        match fetch_bitcoin_price().await {
            Ok(price) => {
                let timestamp = Utc::now().timestamp();
                let payload = DataPayload {
                    height: 0, // Placeholder for block height
                    price,
                    timestamp,
                };
                let data = serde_json::to_string(&payload).unwrap();
                tx.lock().await.send(data).unwrap();

                // Insert Bitcoin price into database
                if let Err(e) = insert_bitcoin_price(&client, price, timestamp).await {
                    eprintln!("Failed to insert Bitcoin price: {}", e);
                }
            },
            Err(e) => eprintln!("Failed to fetch Bitcoin price: {}", e),
        }

        tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
    }
}

async fn start_websocket_server(tx: Arc<Mutex<broadcast::Sender<String>>>) {
    let addr = "127.0.0.1:8080";
    let listener = TcpListener::bind(&addr).await.expect("Failed to bind");

    while let Ok((stream, _)) = listener.accept().await {
        let tx = Arc::clone(&tx);
        tokio::spawn(async move {
            let ws_stream = accept_async(stream).await.expect("Failed to accept");
            let (write, mut read) = ws_stream.split();
            let write = Arc::new(Mutex::new(write));

            while let Some(Ok(msg)) = read.next().await {
                if let Ok(msg) = msg.to_text() {
                    if msg == "subscribe" {
                        let tx = Arc::clone(&tx);
                        let write = Arc::clone(&write);
                        tokio::spawn(async move {
                            let mut rx = tx.lock().await.subscribe();
                            while let Ok(data) = rx.recv().await {
                                if write.lock().await.send(data.into()).await.is_err() {
                                    break;
                                }
                            }
                        });
                    }
                }
            }
        });
    }
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    let (tx, _rx) = broadcast::channel::<String>(100);
    let tx = Arc::new(Mutex::new(tx));

    tokio::spawn(start_websocket_server(Arc::clone(&tx)));

    let client = establish_connection().await.expect("Failed to connect to database");

    tokio::select! {
        _ = run_ingestion_loop(client, Arc::clone(&tx)) => {},
        _ = signal::ctrl_c() => {
            println!("Shutdown signal received, stopping the ingestion process.");
        },
    }
}
