mod channel;
mod router;
mod entity;

use std::{fmt, io};
use axum::{
    routing::{get, post},
    http::StatusCode,
    response::IntoResponse,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::net::{SocketAddr};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc};

use tokio::sync::{Mutex, oneshot};
use crate::channel::broadcast_channel::CHANNEL;
use crate::router::serverstatus;
use bytes::Bytes;
use crate::router::r#static::html_handler;


#[derive(Debug)]
#[derive(Clone)]
pub enum CommandMessage {
    Get {
        key: String,
    },
    Set {
        config_index: i32,
        key: String,
        val: Bytes,
    },
}

impl fmt::Display for CommandMessage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Get => write!(f, "Get {:?}", self),
            Set => write!(f, "Set {:?}", self),
        }
    }
}


async fn root() -> &'static str {
    "Hello, World!"
}

#[tokio::main]
async fn main() {
    println!("Hello, world!");


    let btx2 = Arc::clone(&CHANNEL);
    let rx = btx2.subscribe(); // need to make sure that there exists at least one receiver
    tracing_subscriber::fmt::init();

    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(root))
        .nest("/api",serverstatus::get_status_route())
        .fallback(html_handler);

    let listenerSocket = TcpListener::bind("127.0.0.1:8080").await.unwrap();
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);

    // let t_server = tokio::spawn(async move {
    //
    //
    //     let mut brx2 = btx2.clone().subscribe();
    //     while let message = brx2.recv().await {
    //         println!("GOT = {:?}", message);
    //     }
    // });
    tokio::spawn(async move {
        let server_config_json = entity::server_config::ServerConfigJSON::build_server_config_json();
        loop {
            let server_config_json = server_config_json.clone();
            let mut btx2 = Arc::clone(&CHANNEL);
            // let rx = btx2.subscribe();

            let (mut socket, addr) = listenerSocket.accept().await.unwrap();
            println!("socket accept {}", addr);
            tokio::spawn(async move {
                let mut buf = vec![0; 1024];
                let mut config_index:i32 = -1;
                println!("spawn");
                let mut first_count = 0;
                let mut auth_success = false;
                // In a loop, read data from the socket and write the data back.
                loop {
                    if first_count==0 {
                        socket.write_all("Authentication required".as_ref()).await.expect("failed to write");
                    }
                    buf = vec![0; 1024];
                    let n = socket
                        .read(&mut buf)
                        .await
                        .expect("failed to read data from socket");
                    if first_count<3 {
                        if first_count<=3 && auth_success==false {
                            // if success
                            let len_without_nulls = buf.iter().rposition(|&x| x != 0 && x!=10).map_or(0, |pos| pos + 1); // 10 \n
                            let received_str = String::from_utf8(Vec::from(buf[0..len_without_nulls].to_vec())).unwrap();
                            let name_pass_vec: Vec<&str> = received_str.split(':').collect();
                            println!("username: {} password:{}",name_pass_vec.get(0).unwrap(),name_pass_vec.get(1).unwrap());
                            match &server_config_json.check_name_pass(name_pass_vec.get(0).unwrap(),name_pass_vec.get(1).unwrap()) {
                                Some(found_index) => {
                                    println!("found: {}",found_index);
                                    println!("auth_success:{}",auth_success);
                                    config_index = found_index.clone() as i32;
                                    socket.write_all(format!("Authentication successful. Index is {}", found_index).as_ref()).await.expect("failed to write");
                                    // TODO check ipv4 or ipv6
                                    socket.write_all(format!("You are connecting via IPv4 {}", addr).as_ref()).await.expect("failed to write");
                                    auth_success = true;
                                    continue;
                                }
                                None => {
                                    println!("not found: username or password not correct");
                                    // let shutdown_response = socket.shutdown();
                                    // println!("{:?}", shutdown_response);
                                }
                            }

                        }
                        if first_count>1
                        {
                            socket.write_all(format!("You are connecting via IPv4 {}", addr).as_ref()).await.expect("failed to write")
                        }
                        first_count = first_count+1;
                    }
                    else {
                        // echo messages
                        // socket
                        //     .write_all(&buf[0..n])
                        //     .await
                        //     .expect("failed to write data to socket");
                    }

                    let cmd = CommandMessage::Set {
                        config_index: config_index,
                        key: "received".to_string(),
                        val: Bytes::from(buf.clone())
                    };


                    if auth_success {
                        let cmd_clone = cmd.clone();
                        // println!("{:?}", (Some(cmd_clone)).unwrap());
                        println!("btx2 send");
                        // let message_to_send = (Some(cmd_clone)).unwrap();
                        btx2.send((Some(cmd_clone)).unwrap()).expect("btx2.send error");
                    }
                    println!("buf:{:?}", buf);
                    println!("{}", String::from_utf8(buf.clone()).unwrap());
                    if n == 0 {
                        return;
                    }
                }
            });
        }
    });
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}