use crate::csv::CsvParser;
use crate::prelude::*;
use crate::transaction_engine::TransactionEngine;
use ::csv::Writer;
use anyhow::Context;
use clap::Parser;
use cli::Args;
use flume::{bounded, Sender};
use std::collections::HashMap;
use std::fs::File;
use std::io::{stdout, Write};
use std::sync::Arc;
use tokio::fs::File as AsyncFile;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::task::JoinHandle;
use transaction::Transaction;

mod cli;
mod csv;
mod prelude;
mod setup;
mod transaction;
mod transaction_engine;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let args = Args::parse();

    setup::setup()?;

    run(args).await?;

    Ok(())
}

async fn run(args: Args) -> Result<()> {
    info!("Processing from {:?}", args.input_csv);

    let reader = BufReader::new(AsyncFile::open(args.input_csv).await?);
    let mut reader = reader.lines();

    let header_line = reader.next_line().await?.ok_or_else(|| {
        anyhow!(
        "There must be at least 1 line in the provided CSV, a header line and 0 or more data lines."
    )
    })?;

    let mut csv_parser = CsvParser::new(CsvParser::valid_line(header_line).context("context")?)?;

    let engine = TransactionEngine::default();

    let task_pool = TaskPool::new(engine, args.queue_depth, args.workers);

    while let Some(line) = reader.next_line().await? {
        match CsvParser::valid_line(line) {
            Some(valid_line) => {
                let transaction = csv_parser.line_to_transaction(valid_line)?;
                task_pool.add_transaction(transaction).await?;
            }
            None => {
                warn!("Get an invalid line, skipping");
            }
        }
    }

    let engine = task_pool.wait().await?;

    let user_summaries = engine.current_account_states();

    // Why have this be synchronous when io before has been async? Because we no longer
    // benefit from doing anything concurrently. Honestly,
    // choosing to do this asynchronously has been purely to show an ability to
    // do so, and partly to create an implementation that could
    // conceivably be used in a server. For CLI use,
    // there's no reason this couldn't have been done
    // synchronously.
    let writer: Box<dyn Write> = match &args.output_csv {
        Some(path) => {
            info!("Outputting to {:?}", path);
            let file = File::create(path)?;
            Box::new(file)
        }
        None => {
            info!("No output path specified, outputting to stdout");
            Box::new(stdout().lock())
        }
    };

    let mut csv_writer = Writer::from_writer(writer);

    for summary in user_summaries {
        csv_writer.serialize(summary)?;
    }

    Ok(())
}

pub struct TaskPool {
    parallelism: usize,
    senders: HashMap<usize, Sender<transaction::Transaction>>,
    tasks: Vec<JoinHandle<Result<()>>>,
    engine: Arc<TransactionEngine>,
}

impl TaskPool {
    pub fn new(engine: TransactionEngine, queue_depth: usize, num_workers: isize) -> Self {
        let system_parallelism = std::thread::available_parallelism()
            .expect("Required parallel capable environment")
            .get();

        let parallelism = if num_workers < 0 {
            // subtract from total available
            std::cmp::max(1, (system_parallelism as isize) + num_workers) as usize
        } else if num_workers == 0 {
            // set to available
            system_parallelism
        } else {
            // set to num
            num_workers as usize
        };

        info!("Using {parallelism} engine worker threads");

        let mut senders = HashMap::new();
        let mut tasks = Vec::new();
        let engine = Arc::new(engine);

        for i in 0..(parallelism) {
            let (sender, recv) = bounded(queue_depth);
            tasks.push(tokio::task::spawn_blocking({
                let engine = Arc::clone(&engine);
                move || {
                    for transaction in recv {
                        engine.add_transaction(transaction)?;
                    }
                    Ok(())
                }
            }));
            senders.insert(i, sender);
        }

        Self {
            parallelism,
            senders,
            tasks,
            engine,
        }
    }

    pub async fn add_transaction(&self, transaction: Transaction) -> Result<()> {
        let associated_task = (transaction.client as usize) % self.parallelism;

        let sender = self
            .senders
            .get(&associated_task)
            .expect("Must have created associated task");

        sender.send_async(transaction).await?;

        Ok(())
    }

    pub async fn wait(mut self) -> Result<TransactionEngine> {
        self.senders.clear();

        for task in self.tasks.drain(..) {
            task.await??;
        }

        Arc::try_unwrap(self.engine)
            .map_err(|_| anyhow!("Did not have only reference to the engine!"))
    }
}
