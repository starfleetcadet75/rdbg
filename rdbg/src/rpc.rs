use tarpc::sync::{client, server};
use tarpc::sync::client::ClientExt;
use tarpc::util::Message;

use std::borrow::BorrowMut;
use std::error::Error;
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::mpsc;
use std::thread;

use rdbg_core::core::debugger::Debugger;
use rdbg_core::core::profile::Profile;


// Definitions of the rpc calls provided by the service.
service! {
    rpc new_project(profile: Profile) -> String | Message;
    rpc run() -> String | Message;
}


#[derive(Clone)]
struct DebuggerService {
    debugger: Arc<Mutex<Debugger>>,
}

impl DebuggerService {
    fn new() -> DebuggerService {
        DebuggerService { debugger: Arc::new(Mutex::new(Debugger::new())) }
    }
}

/// Implements rpc calls for the debugger service.
impl SyncService for DebuggerService {
    fn new_project(&self, profile: Profile) -> Result<String, Message> {
        let mut debugger = self.debugger.lock().unwrap();
        if let Err(error) = debugger.borrow_mut().new_project(profile) {
            Err(Message(String::from(error.description()))) // TODO: Find a cleaner way to do this
        } else {
            Ok(String::from("Created new project from profile"))
        }
    }

    fn run(&self) -> Result<String, Message> {
        let mut debugger = self.debugger.lock().unwrap();
        if let Err(error) = debugger.borrow_mut().execute() {
            Err(Message(format!("{}", error)))
        } else {
            Ok(String::from("Running program"))
        }
    }
}

/// Starts a local debugger session with the service running in a seperate thread.
pub fn run() -> Option<SyncClient> {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let handle = DebuggerService::new()
            .listen("localhost:0", server::Options::default())
            .unwrap();
        tx.send(handle.addr()).unwrap();
        handle.run();
    });
    connect(rx.recv().unwrap())
}

/// Starts a headless server running at the given address.
pub fn listen(address: SocketAddr) {
    let handle = DebuggerService::new()
        .listen(address, server::Options::default())
        .expect("Failed to start server");
    info!("Listening on: {:?}", handle.addr());
    handle.run();
}

/// Connects to a remote debugger at the given address.
pub fn connect(address: SocketAddr) -> Option<SyncClient> {
    Some(
        SyncClient::connect(address, client::Options::default()).expect("Connection failed"),
    )
}
