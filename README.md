# Tokio-based Web Scraper
This implementation uses the Tokio runtime to make concurrent HTTP requests asynchronously. It is designed to efficiently handle large numbers of concurrent operations using lightweight tasks.

## How It Works
- The run_with_tokio function runs multiple HTTP requests concurrently.
- Results are sent through an mpsc channel.
- The consume_results function receives results from the channel and updates a shared Vec.
