use csv::{Reader, Writer};
use serde::Serialize;
use std::{
    collections::HashMap,
    io::{self, Write},
    path::{Path, PathBuf},
};
use structopt::StructOpt;

use crate::client::Client;
use crate::transaction::{Transaction, TransactionType};

pub mod client;
pub mod error;
pub mod transaction;

#[derive(Debug, StructOpt)]
#[structopt(about = "Payments engine")]
struct CommandOpt {
    input: PathBuf,
}

fn main() -> csv::Result<()> {
    let opt = CommandOpt::from_args();
    let mut rdr = Reader::from_path(opt.input.clone())?;

    let mut clients: HashMap<u16, Client> = HashMap::new();

    for res in rdr.deserialize() {
        let tx: Transaction = res?;
        let tx_id = tx.id;
        let client = clients.entry(tx.client_id).or_default();

        let disputed_tx = if matches!(tx.ty, TransactionType::Dispute) {
            if tx.id > client.nonce {
                eprintln!(
                    "Error encountered while processing TxID {}: Disputing a future transaction",
                    tx_id,
                );
                continue;
            }

            locate_tx(opt.input.clone(), tx_id)?
        } else {
            None
        };

        if let Err(e) = client.process_tx(tx, disputed_tx) {
            eprintln!("Error encountered while processing TxID {}: {}", tx_id, e);
        }
    }

    write_client_statements(io::stdout(), clients)?;

    Ok(())
}

fn locate_tx<A: AsRef<Path>>(path: A, tx_id: u32) -> csv::Result<Option<Transaction>> {
    let mut rdr = Reader::from_path(path)?;

    for res in rdr.deserialize() {
        let tx: Transaction = res?;
        if tx.id == tx_id {
            return Ok(Some(tx));
        }
    }

    Ok(None)
}

fn write_client_statements<W: Write>(output: W, clients: HashMap<u16, Client>) -> csv::Result<()> {
    #[derive(Serialize)]
    struct Row {
        client: u16,
        available: f32,
        held: f32,
        total: f32,
        locked: bool,
    }

    let mut wtr = Writer::from_writer(output);

    for (id, client) in clients {
        let held = client.get_held();
        wtr.serialize(Row {
            client: id,
            available: client.total - held,
            held,
            total: client.total,
            locked: client.locked,
        })?;
    }

    Ok(())
}
