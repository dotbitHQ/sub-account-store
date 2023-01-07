use clap::Parser;
use jsonrpsee::http_server::HttpServerBuilder;
use rocksdb::{prelude::Open, OptimisticTransactionDB};
use std::net::SocketAddr;
use sub_account_store::rpc_server::{RpcServer, RpcServerImpl};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    //listen address
    #[arg(short, long, default_value = "127.0.0.1:10000")]
    listen_addr: String,

    //database path of rocksdb
    #[arg(short, long, default_value = "/tmp/smt-store")]
    db_path: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let db = OptimisticTransactionDB::open_default(args.db_path).expect("cannot open database");
    let server = HttpServerBuilder::default()
        .build(args.listen_addr.parse::<SocketAddr>()?)
        .await?;
    let _handle = server.start(RpcServerImpl::new(db).into_rpc())?;
    println!("Server started at http://{}", args.listen_addr);
    futures::future::pending().await
}
