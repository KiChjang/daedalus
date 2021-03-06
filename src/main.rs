use csv::{ReaderBuilder, Writer};
use serde::{Serialize, Serializer};
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
    /// Only display clients with a locked status
    #[structopt(long)]
    locked: bool,
}

fn main() -> csv::Result<()> {
    let opt = CommandOpt::from_args();
    // csv::Reader is buffered by default, so that the entire input file
    // doesn't get loaded in memory all at once.
    let mut rdr = ReaderBuilder::new()
        // Disputes, resolves and chargebacks may omit the amount column,
        // enable flexible here to allow amount omission.
        .flexible(true)
        .from_path(opt.input.as_path())?;

    // Client IDs are only relevant here in this HashMap -- the Client
    // struct itself does not store the ID, thus eliminating redundancy
    // and saving storage space for Clients.
    let mut clients: HashMap<u16, Client> = HashMap::new();
    // Assumption: TxIDs are monotonically increasing, so we can track which
    // transactions happened before the other.
    let mut last_tx_id = 0;

    for res in rdr.deserialize() {
        process_tx(res?, &mut last_tx_id, &mut clients, opt.input.as_path())?;
    }

    write_client_statements(io::stdout(), clients, opt.locked)
}

fn process_tx<A: AsRef<Path>>(
    tx: Transaction,
    last_tx_id: &mut u32,
    clients: &mut HashMap<u16, Client>,
    txs_path: A,
) -> csv::Result<()> {
    let tx_id = tx.id;
    let client = clients.entry(tx.client_id).or_default();

    let disputed_tx = if matches!(tx.ty, TransactionType::Dispute) {
        if tx_id > *last_tx_id {
            eprintln!(
                "Error encountered while disputing TxID {}: transaction has not yet happened",
                tx_id,
            );
            return Ok(());
        }

        locate_tx(txs_path, tx_id)?
    } else {
        None
    };

    if matches!(
        tx.ty,
        TransactionType::Deposit | TransactionType::Withdrawal
    ) {
        *last_tx_id += 1;

        debug_assert_eq!(*last_tx_id, tx_id);
    }

    if let Err(e) = client.process_tx(tx, disputed_tx) {
        eprintln!("Error encountered while processing TxID {}: {}", tx_id, e);
    }

    Ok(())
}

// Assumption: Disputes rarely happen, so we do not store an entire history of
// transactions in the client. Instead, whenever there is a dispute, we reopen
// the list of transactions file and search for the disputed transaction from
// the beginning.
fn locate_tx<A: AsRef<Path>>(path: A, tx_id: u32) -> csv::Result<Option<Transaction>> {
    let mut rdr = ReaderBuilder::new().flexible(true).from_path(path)?;

    for res in rdr.deserialize() {
        let tx: Transaction = res?;

        // Ensure that we don't dispute a dispute, chargeback or a resolved transaction
        if tx.id == tx_id
            && matches!(
                tx.ty,
                TransactionType::Deposit | TransactionType::Withdrawal
            )
        {
            return Ok(Some(tx));
        }
    }

    Ok(None)
}

fn write_client_statements<W: Write>(
    output: W,
    clients: HashMap<u16, Client>,
    only_locked: bool,
) -> csv::Result<()> {
    fn serialize_amount<S>(data: &f32, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        const PRECISION: i32 = 4;
        let trunc = (*data * 10.0f32.powi(PRECISION)).trunc() / 10.0f32.powi(PRECISION);
        serializer.serialize_f32(trunc)
    }

    #[derive(Serialize)]
    struct Row {
        client: u16,
        #[serde(serialize_with = "serialize_amount")]
        available: f32,
        #[serde(serialize_with = "serialize_amount")]
        held: f32,
        #[serde(serialize_with = "serialize_amount")]
        total: f32,
        locked: bool,
    }

    let mut wtr = Writer::from_writer(output);

    for (id, client) in clients {
        if only_locked && !client.locked {
            continue;
        }

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
