use tokio::sync::{broadcast, mpsc};
use tokio::sync::broadcast::{Receiver, Sender};
use crate::CommandMessage;
use std::sync::Arc;

use once_cell::sync::Lazy;

// lazy_static! {
//      let (tx, mut rx) = broadcast::channel(32);
//     pub static mut BROADCAST_TX: Option<Sender<Command>> = Option::from(tx.clone());
//     pub static mut BROADCAST_RX: Option<Receiver<Command>> = Option::from(rx);
// }

// static mut BROADCAST_TX: Option<Sender<Command>> = None;
// static mut BROADCAST_RX: Option<Receiver<Command>> = None;
// pub unsafe fn get_broadcast_txrx<T>() -> (Sender<Command>, Receiver<Command>) {
//
//     match BROADCAST_TX {
//         None => {
//             println!("first get_broadcast_txrx");
//             let (tx, mut rx) = broadcast::channel(32);
//             BROADCAST_TX = Option::from(tx.clone());
//             BROADCAST_RX = Option::from(rx);
//             (tx.clone(), tx.subscribe())
//         }
//         Some(_) => {
//             println!("not the first get_broadcast_txrx");
//
//             (BROADCAST_TX.unwrap().clone(),BROADCAST_TX.unwrap().clone().subscribe())
//         }
//     }
//
// }


pub static CHANNEL: Lazy<Arc<broadcast::Sender<CommandMessage>>> = Lazy::new(|| {
    let (tx, _) = broadcast::channel(100);
    Arc::new(tx)
});