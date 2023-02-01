use clap::Parser;
use jsonrpsee::http_server::HttpServerBuilder;
use log::{error, info};
use rocksdb::{prelude::Open, OptimisticTransactionDB};
use std::net::SocketAddr;
use sub_account_store::rpc_server::{RpcServer, RpcServerImpl};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    //listen address
    #[clap(short, long, default_value = "127.0.0.1:10000")]
    listen_addr: String,

    //database path of rocksdb
    #[clap(short, long, default_value = "/tmp/smt-store")]
    db_path: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let args = Args::parse();

    info!("opening database");

    let db = match OptimisticTransactionDB::open_default(args.db_path) {
        Ok(d) => d,
        Err(e) => {
            error!("cannot open database :{}", &e);
            return Ok(());
        }
    };

    //perhaps try tuning the database performance with the following parameters
    // let mut opts = Options::default();
    // opts.create_if_missing(true);
    //opts.set_bytes_per_sync(1048576);
    //opts.set_max_background_jobs(6);
    //opts.set_keep_log_file_num(32);
    //opts.set_level_compaction_dynamic_level_bytes(true);
    // opts.set_write_buffer_size(128 * 1024 * 1024);
    // opts.set_min_write_buffer_number_to_merge(1);
    // opts.set_max_write_buffer_number(2);
    // opts.set_max_write_buffer_size_to_maintain(16);
    // opts.set_max_file_opening_threads(32);

    // let db = match OptimisticTransactionDB::open(&opts, args.db_path) {
    //     Ok(d) => d,
    //     Err(e) => {
    //         error!("cannot open database :{}", &e);
    //         return Ok(());
    //     }
    // };

    info!("opening database success");
    let server = HttpServerBuilder::default()
        .build(args.listen_addr.parse::<SocketAddr>()?)
        .await?;
    let _handle = server.start(RpcServerImpl::new(db).into_rpc())?;
    info!("server started at http://{}", args.listen_addr);
    futures::future::pending().await
}
