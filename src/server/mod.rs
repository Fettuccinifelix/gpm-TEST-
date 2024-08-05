use std::sync::Arc;

use anyhow::Error;
use anyhow::Result;
use log::error;
use log::info;
use log::warn;
use paste::paste;
use request::TaskData::*;
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio::sync::oneshot;
use tokio::sync::Semaphore;

use crate::config::GPM_TCP_ADDR;
use crate::config::MAX_CONCURRENT_CONNECTIONS;
use crate::retry;
use crate::verify_task_data;
use crate::ManagerChannelMap;
use crate::_dispatch_task as dispatch_task;
use crate::import_sgcp;
use crate::managers::ManagerChannelData;
use crate::sgcp::*;
use crate::streaming::Connection;

pub async fn init_gpm_listener(manager_channel_map: ManagerChannelMap) {
    let listener = TcpListener::bind(GPM_TCP_ADDR).await.unwrap();
    let sem = Arc::new(Semaphore::new(MAX_CONCURRENT_CONNECTIONS));
    info!("GPM listening on {:?}", GPM_TCP_ADDR);
    loop {
        let sem_clone = Arc::clone(&sem);
        let (stream, client_addr) = match listener.accept().await {
            Ok(conn) => conn,
            Err(err) => {
                error!(
                    "Encountered an error when accepting new connection; error={:?}",
                    err
                );
                continue;
            },
        };
        let send_channel_map = manager_channel_map.clone();
        tokio::spawn(async move {
            // Bounds number of concurrent connections
            if let Ok(_) = retry!(sem_clone.try_acquire()) {
                info!("Accpeted new remote connection from host={:?}", client_addr);
                handle_connection(stream, &send_channel_map).await.unwrap();
            } else {
                error!("Rejected new remote connection from host={:?}, currently serving maximum_clients={:?}", client_addr, MAX_CONCURRENT_CONNECTIONS);
            }
        });
    }
}

/// Reads protobufs from the underlying stream and dispatches tasks to the appropriate
/// task manager.
async fn handle_connection(mut stream: TcpStream, map: &ManagerChannelMap) -> Result<()> {
    let mut conn = Connection::new(stream);
    loop {
        match conn.read_frame().await {
            Ok(val) => match val {
                Some(req) => {
                    info!("Recieved request: {:?}", req);
                    let res = match dispatch_task(req, &map).await {
                        Ok(res) => res,
                        Err(err) => {
                            error!("An error occurred when dispatching task; error={err}");
                            conn.write("An error occurred; Error={err}".as_bytes());
                            continue;
                        },
                    };
                    match retry!(conn.write(res.as_bytes()).await) {
                        Ok(_) => (),
                        Err(err) => error!(
                            "An error occurred when writing response to peer; error={:?}",
                            err
                        ),
                    };
                },
                None => {
                    info!("Connection closed with peer");
                    break;
                },
            },
            Err(err) => {
                error!("Reading frame from stream failed with error={:?}", err);
                break;
            },
        }
    }
    Ok(())
}

/// Dispatches a sgcp::Request to the appropiate task manager
pub async fn dispatch_task(request: Request, map: &ManagerChannelMap) -> Result<String> {
    dispatch_task!(
        request,
        Resource::Bms => map.get(request.resource().as_str_name()),
        Resource::Emg => map.get(request.resource().as_str_name()),
        Resource::Maestro => map.get(request.resource().as_str_name())
    )
}
