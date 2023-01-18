use std::collections::HashMap;
use std::convert::Infallible;
use std::env;
use std::ptr::write;
use std::sync::Arc;
use futures::StreamExt;
use log::info;
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};
use tokio::sync::RwLock;
use tokio_stream::wrappers::UnboundedReceiverStream;
use warp::Filter;
use warp::ws::{Message, WebSocket};

// 全局唯一id
static NEXT_USERID: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(1);

// 支持多线程的读写锁, RwLock由tokio提供
// k: id, v: Message sender
type Users = Arc<RwLock<HashMap<usize, UnboundedSender<Result<Message, warp::Error>>>>>;

#[tokio::main]
async fn main() {
    env::set_var("RUST_APP_LOG", "debug");
    pretty_env_logger::init_custom_env("RUST_APP_LOG");

    let users = Users::default();
    // warp::ws标注为websocket请求，而不是http
    let chat = warp::path("ws")
        .and(warp::ws())
        .and(with_users(users))
        // on_upgrade会获得一个socket
        .map(|ws: warp::ws::Ws, users: Users| ws.on_upgrade(move |socket| connect(socket, users)));

    let files = warp::fs::dir("static");

    let routes = chat.or(files);
    warp::serve(routes).run(([127,0,0,1], 7070)).await;

}

fn with_users(users: Users) -> impl Filter<Extract = (Users, ), Error = Infallible> + Clone {
    warp::any().map(move || users.clone())
}

async fn connect(ws: WebSocket, users: Users) {
    let my_id = NEXT_USERID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    info!("Connect UserId {}", my_id);
    let (user_tx, mut user_rx) = ws.split();
    let (tx, rx) = unbounded_channel();
    let rx = UnboundedReceiverStream::new(rx);
    tokio::spawn(rx.forward(user_tx));
    users.write().await.insert(my_id, tx);
    while let Some(res) = user_rx.next().await {
        broadcast(res.unwrap(), &users).await;
    }

    disconnect(my_id, &users).await;
}

async fn broadcast(msg: Message, users: &Users) {
    if let Ok(_) = msg.to_str() {
        for (&uid, tx) in users.read().await.iter() {
            info!("uid: {} send msg: {:?}", uid, msg.clone());
            tx.send(Ok(msg.clone())).expect("Failed to send message.");
        }
    }
}

async fn disconnect(my_id: usize, users: &Users) {
    info!("GOODBYE {}", my_id);
    users.write().await.remove(&my_id);
}