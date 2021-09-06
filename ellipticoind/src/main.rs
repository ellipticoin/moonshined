use ellipticoind::{
    config::{SubCommand, OPTS},
    constants::TRANSACTIONS_FILE,
    db,
    sub_commands::{self, generate_keypair},
};
use std::{io::Write, process};

#[async_std::main]
async fn main() {
    ctrlc::set_handler(move || {
        async_std::task::block_on(async {
            let mut transacations_file = TRANSACTIONS_FILE.write().await;
            transacations_file.flush().unwrap();
            db::flush().await;
            db::dump().await;
            process::exit(0)
        })
    })
    .expect("Error setting Ctrl-C handler");
    match &OPTS.subcmd {
        Some(SubCommand::GenerateKeypair) => generate_keypair(),
        None => sub_commands::main().await,
    }
}
