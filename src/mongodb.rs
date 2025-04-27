use crate::logger::LogEntry;
use mongodb::{
    Client,
    bson::doc,
    options::{ClientOptions, ServerApi, ServerApiVersion},
};
use std::error::Error;

pub struct MongoLogger {
    client: Client,
    database: String,
    collection: String,
}

impl MongoLogger {
    /// Creates a new MongoLogger instance
    /// Note: probably has to be changed cuz of atlas lol
    pub async fn new(uri: &str, database: &str, collection: &str) -> Result<Self, Box<dyn Error>> {
        // Parse connection string
        let mut client_options = ClientOptions::parse(uri).await?;

        // Set Server API version
        let server_api = ServerApi::builder().version(ServerApiVersion::V1).build();
        client_options.server_api = Some(server_api);

        // Create the client
        let client = Client::with_options(client_options)?;

        // Ping the database
        client
            .database("admin")
            .run_command(doc! { "ping": 1 }, None)
            .await?;

        // Print after ping
        println!("mongodb.rs - Success connecting to mongodb");

        Ok(Self {
            client,
            database: database.to_string(),
            collection: collection.to_string(),
        })
    }

    /// Logs a message to MongoDB
    pub async fn log_entry(&self, entry: LogEntry) -> Result<(), Box<dyn Error>> {
        let collection = self
            .client
            .database(&self.database)
            .collection(&self.collection);

        let document = mongodb::bson::to_document(&entry)?;
        collection.insert_one(document, None).await?;

        Ok(())
    }
}
