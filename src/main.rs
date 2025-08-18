mod client;
mod convert;
mod handlers;
mod processed_transaction;
mod raw_transaction;

use client::Client;
use convert::convert_number_to_fractional;
use handlers::handle_transaction;
use processed_transaction::ProcessedTransaction;
use raw_transaction::RawTransaction;
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let input_file = if args.len() > 1 { &args[1] } else { "unknown" };
    eprintln!("input = {}", input_file);

    if !Path::new(input_file).exists() {
        eprintln!("Error: File '{}' not found", input_file);
        return Ok(());
    }

    let mut transactions: HashMap<u32, ProcessedTransaction> = HashMap::new();
    let mut clients: HashMap<u16, Client> = HashMap::new();

    let file = File::open(input_file)?;
    let mut csv_reader = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .from_reader(file);

    let mut row = 0;
    for result in csv_reader.deserialize() {
        row += 1;
        let raw_tx: RawTransaction = match result {
            Ok(tx) => tx,
            Err(e) => {
                eprintln!("Error parsing row: {}", e);
                continue;
            }
        };

        eprintln!("CSV Row {}, {:?}", row, raw_tx);

        handle_transaction(&raw_tx, &mut transactions, &mut clients);
    }

    println!("client,available,held,total,locked");
    for (client_id, client) in clients.iter() {
        let available = convert_number_to_fractional(client.available);
        let held = convert_number_to_fractional(client.held);
        let total = convert_number_to_fractional(client.total);

        println!(
            "{},{:.4},{:.4},{:.4},{}",
            client_id, available, held, total, client.locked
        );
    }

    Ok(())
}
