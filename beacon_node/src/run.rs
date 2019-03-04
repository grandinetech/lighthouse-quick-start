use client::client_types::{StandardClientType, TestingClientType};
use client::error;
use client::{notifier, Client, ClientConfig};
use futures::sync::oneshot;
use futures::Future;
use slog::info;
use std::cell::RefCell;
use tokio::runtime::Builder;

pub fn run_beacon_node(config: ClientConfig, log: slog::Logger) -> error::Result<()> {
    let mut runtime = Builder::new()
        .name_prefix("main-")
        .build()
        .map_err(|e| format!("{:?}", e))?;

    // Log configuration
    info!(log, "";
          "data_dir" => &config.data_dir.to_str(),
          "port" => &config.net_conf.listen_port);

    // run service until ctrl-c
    let (ctrlc_send, ctrlc) = oneshot::channel();
    let ctrlc_send_c = RefCell::new(Some(ctrlc_send));
    ctrlc::set_handler(move || {
        if let Some(ctrlc_send) = ctrlc_send_c.try_borrow_mut().unwrap().take() {
            ctrlc_send.send(()).expect("Error sending ctrl-c message");
        }
    });

    let (exit_signal, exit) = exit_future::signal();

    let executor = runtime.executor();

    // currently testing - using TestingNode type
    let client: Client<TestingClientType> = Client::new(config, log.clone(), executor.clone())?;
    notifier::run(&client, executor, exit);

    runtime.block_on(ctrlc);

    // perform global shutdown operations.
    info!(log, "Shutting down..");
    exit_signal.fire();
    drop(client);
    runtime.shutdown_on_idle().wait().unwrap();
    Ok(())
}
