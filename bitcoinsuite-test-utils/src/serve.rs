use std::{net::SocketAddr, time::Duration};

use tokio::task::JoinHandle;
use warp::Filter;

use crate::is_free_tcp;

pub async fn serve_and_wait<F>(routes: F, host_addr: impl Into<SocketAddr>) -> JoinHandle<()>
where
    F: Filter + Clone + Send + Sync + 'static,
    F::Extract: warp::Reply,
{
    let host_addr = host_addr.into();
    let handle = tokio::spawn(async move {
        warp::serve(routes).run(host_addr).await;
    });
    for _ in 0..100 {
        if !is_free_tcp(host_addr.port()) {
            break;
        }
        tokio::time::sleep(Duration::from_millis(1)).await;
    }
    handle
}
