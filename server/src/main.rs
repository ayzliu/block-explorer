use serde::Deserialize;
use tokio_postgres::{NoTls, Error};
use tokio::signal;
use chrono::{TimeZone, Utc};



#[derive(Deserialize, Debug)]
struct ApiResponse {
    height: i32,
    time: i64,  // UNIX timestamp
}

async fn fetch_block_height() -> Result<ApiResponse, reqwest::Error> {
    let url = "https://blockchain.info/latestblock";
    let response = reqwest::get(url).await?.json::<ApiResponse>().await?;
    Ok(response)
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
    let timestamp_str = timestamp_tz.to_rfc3339(); // Convert to string for PostgreSQL

    let rows = client.query(
        "SELECT 1 FROM blocks WHERE height = $1",
        &[&height]
    ).await?;

    if rows.is_empty() {
        client.execute(
            "INSERT INTO blocks (height, timestamp) VALUES ($1, $2)",
            &[&height, &timestamp_str]
        ).await?;
        println!("Inserted block height: {}, timestamp: {}", height, timestamp_tz);
    } else {
        println!("Block height {} already exists", height);
    }

    Ok(())
}


async fn run_ingestion_loop(client: tokio_postgres::Client) {
    loop {
        match fetch_block_height().await {
            Ok(api_response) => {
                if let Err(e) = insert_block_height(&client, api_response.height, api_response.time).await {
                    eprintln!("Failed to insert block height and timestamp: {}", e);
                }
            },
            Err(e) => eprintln!("Failed to fetch block height: {}", e),
        }
        tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
    }
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    let client = establish_connection().await.expect("Failed to connect to database");
    tokio::select! {
        _ = run_ingestion_loop(client) => {},
        _ = signal::ctrl_c() => {
            println!("Shutdown signal received, stopping the ingestion process.");
        },
    }
}
