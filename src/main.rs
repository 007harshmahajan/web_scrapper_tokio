use std::sync::{Arc, Mutex};
use std::sync::mpsc::{self, Sender, Receiver};
use reqwest::Client;
use futures::stream::{FuturesUnordered, StreamExt};
use tokio::task;
use std::time::Instant;

#[derive(Clone)]
struct Request {
    url: String,
    number: usize, // Number between 0-1000
}

struct ResultStruct {
    number: usize,
    size: isize, // Size of the image or -1 if an error occurs
}

// Function to make API calls using Tokio and send results to a channel
async fn run_with_tokio(requests: Vec<Request>, sender: Sender<ResultStruct>) {
    let client = Client::new();
    let mut futures = FuturesUnordered::new();

    for request in requests {
        let client_clone = client.clone();
        let sender_clone = sender.clone();
        futures.push(task::spawn(async move {
            let number = request.number;
            match client_clone.get(&request.url).send().await {
                Ok(response) => {
                    let size = match response.bytes().await {
                        Ok(bytes) => bytes.len() as isize,
                        Err(_) => -1,
                    };
                    sender_clone.send(ResultStruct { number, size }).unwrap();
                }
                Err(_) => {
                    sender_clone.send(ResultStruct { number, size: -1 }).unwrap();
                }
            }
        }));
    }

    // Wait for all tasks to complete
    while futures.next().await.is_some() {}
}

// Function to consume result structs from the channel and update the vector
fn consume_results(receiver: &Receiver<ResultStruct>, results: Arc<Mutex<Vec<isize>>>) {
    for result in receiver.iter() {
        let mut results_lock = results.lock().unwrap();
        results_lock[result.number] = result.size;
    }
}

// Main function to execute the Tokio-based method
#[tokio::main]
async fn main() {
    // Generate 1,000 requests
    let requests: Vec<Request> = (0..1000)
        .map(|i| Request {
            url: "https://yral.com/img/android-chrome-384x384.png".to_string(),
            number: i,
        })
        .collect();

    // Initialize a vector with 0s to store image sizes
    let tokio_results = Arc::new(Mutex::new(vec![0; 1000]));

    // Create a channel for communication
    let (tokio_sender, tokio_receiver) = mpsc::channel();

    // Measure time for the Tokio-based method
    let tokio_start = Instant::now();
    println!("Running Tokio-based method:");
    run_with_tokio(requests.clone(), tokio_sender).await;
    let tokio_duration = tokio_start.elapsed();
    println!("Tokio-based method took: {:?}", tokio_duration);

    // Consume the results and update the vector
    consume_results(&tokio_receiver, Arc::clone(&tokio_results));

    // Print results summary (e.g., the first 10 image sizes)
    let tokio_final_results = tokio_results.lock().unwrap();
    println!("First 10 image sizes (Tokio): {:?}", &tokio_final_results[..10]);
}
