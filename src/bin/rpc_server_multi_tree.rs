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
    let server = HttpServerBuilder::default()
        .build(args.listen_addr.parse::<SocketAddr>()?)
        .await?;
    let _handle = server.start(RpcServerImpl::new(db).into_rpc())?;
    info!("server started at http://{}", args.listen_addr);
    futures::future::pending().await
}
