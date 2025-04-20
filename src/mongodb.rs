use mongodb::{Client, options::ClientOptions};
use std::error::Error;
use crate::logging::LogEntry;

/// MongoDB client wrapper for logging messages
pub struct MongoLogger {
    client: Client,
    database: String,
    collection: String,
}

impl MongoLogger {
    /// Creates a new MongoLogger instance
    /// * uri - MongoDB connection URI
    /// * database - Database name
    /// * collection - Collection name
    pub async fn new(uri: &str, database: &str, collection: &str) -> Result<Self, Box<dyn Error>> {
        let client_options = ClientOptions::parse(uri).await?;
        let client = Client::with_options(client_options)?;
        
        Ok(Self {
            client,
            database: database.to_string(),
            collection: collection.to_string(),
        })
    }

    /// Logs a message to MongoDB
    /// * entry - LogEntry containing all the logging information
    pub async fn log_entry(&self, entry: LogEntry) -> Result<(), Box<dyn Error>> {
        let collection = self.client
            .database(&self.database)
            .collection(&self.collection);

        // Convert LogEntry to BSON document
        let document = mongodb::bson::to_document(&entry)?;
        collection.insert_one(document, None).await?;
        
        Ok(())
    }
} 