mod world;

use crate::types::NnErrno;
use anyhow::{Context, Error};
use env_logger::Env;
use std::path::PathBuf;
use structopt::StructOpt;
use wasmtime::{Linker, Module, Store, TypedFunc};
use wiggle::GuestErrorType;
use world::World;

const DEFAULT_FILTER: &str = "info,regalloc=warn";

fn main() -> Result<(), Error> {
    env_logger::init_from_env(Env::default().default_filter_or(DEFAULT_FILTER));

    let Args { module } = Args::from_args();

    log::info!("Started");

    let wasm = std::fs::read(&module)
        .with_context(|| format!("Unable to read \"{}\"", module.display()))?;

    log::info!("Loaded {} bytes from \"{}\"", wasm.len(), module.display());

    let mut store = Store::default();
    let m = Module::from_binary(store.engine(), &wasm)
        .context("Unable to load the WebAssembly binary")?;

    log::info!("Loaded the WebAssembly module");

    let mut linker: Linker<World> = Linker::new(store.engine());

    wasi_ephemeral_nn::add_to_linker(&mut linker, |world| {
        &mut world.wasi_nn_ctx
    })
    .context("Unable to add")?;

    let instance = linker
        .instantiate(&mut store, &m)
        .context("Unable to instantiate the WebAssembly module")?;

    log::info!("Instantiated the WebAssembly module");

    let start: TypedFunc<(), ()> = instance
        .get_typed_func(&mut store, "start")
        .context("Unable to get the start() function")?;

    log::info!("Calling start()");

    start
        .call(&mut store, ())
        .context("An error occurred while calling start()")?;

    log::info!("Completed successfully!");

    Ok(())
}

wiggle::from_witx!({
    witx: ["$CARGO_MANIFEST_DIR/../vendor/wasi-nn/phases/ephemeral/witx/wasi_ephemeral_nn.witx"]
});

impl GuestErrorType for NnErrno {
    fn success() -> Self { NnErrno::Success }
}

#[derive(Debug, StructOpt)]
struct Args {
    #[structopt(
        help = "The WebAssembly module to execute",
        parse(from_os_str)
    )]
    module: PathBuf,
}
