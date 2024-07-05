use std::fmt::Write;
use std::path::PathBuf;
use std::sync::Arc;

use base64::prelude::*;
use structopt::StructOpt;
use xxdk::base::*;
use xxdk::service::*;

const SECRET: &str = "Hello";
const REGISTRATION_CODE: &str = "";

#[derive(Debug, structopt::StructOpt)]
pub struct Options {
    /// Path to network definition file
    #[structopt(long)]
    pub ndf: PathBuf,

    /// Path to state directory
    #[structopt(long)]
    pub state_dir: String,
}

pub async fn run() -> Result<(), String> {
    let options = Options::from_args();

    let ndf_contents = std::fs::read_to_string(&options.ndf).map_err(|e| e.to_string())?;

    println!("[Demo] ======== Rust xxdk RPC demo =========");
    println!(
        "[Demo] xxdk-client version: {}\n",
        xxdk::base::get_version()
    );

    if std::fs::read_dir(&options.state_dir).is_err() {
        CMix::create(
            &ndf_contents,
            &options.state_dir,
            SECRET.as_bytes(),
            REGISTRATION_CODE,
        )?;
    }

    let cmix = CMix::load(&options.state_dir, SECRET.as_bytes(), &[])?;
    let reception_id = cmix.reception_id()?;
    println!(
        "[Demo] cMix reception ID: {}",
        BASE64_STANDARD.encode(&reception_id)
    );

    let cmix_config = CMixServerConfig {
        ndf_path: String::from(options.ndf.to_str().unwrap()),
        storage_dir: options.state_dir,
        secret: String::from(SECRET),
        reception_id: BASE64_STANDARD_NO_PAD.encode(&reception_id),
        private_key: String::from(""),
    };

    let xx_router = xxdk::service::Router::new(Arc::new(cmix)).route("demo", xx_rpc_handler);
    CMixServer::serve(xx_router, cmix_config).await
}

pub async fn xx_rpc_handler(_: Arc<CMix>, request: IncomingRequest) -> Result<Vec<u8>, String> {
    let sender: String = request.sender_id.iter().fold(String::new(), |mut s, b| {
        write!(s, "{b:02x}").unwrap();
        s
    });
    tracing::info!(sender, "Received message via cMix",);
    let text = String::from_utf8_lossy(&request.request);

    Ok(format!("Hi from rust rpc example! Echoed message: {text}").into_bytes())
}
