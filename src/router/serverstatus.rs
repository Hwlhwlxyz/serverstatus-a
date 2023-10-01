use axum::{response::sse::{Event, Sse}, routing::get, Router, headers, TypedHeader, Json};
use futures::stream::{self, Stream};
use std::{convert::Infallible, path::PathBuf, time::Duration};
use std::fmt::Display;
use std::future::Future;
use std::sync::Arc;
use async_stream::__private::AsyncStream;
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};
use axum::response::sse::KeepAlive;
use axum::routing::post;
use futures::TryFutureExt;
use serde::Serialize;
use tokio_stream::StreamExt as _;
use tower_http::{services::ServeDir, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use tokio_stream::{wrappers, StreamExt};
use tokio_stream::wrappers::BroadcastStream;
use crate::channel::broadcast_channel::CHANNEL;
use crate::{CommandMessage, entity};
use tokio::sync::broadcast;
use tokio::sync::broadcast::error::RecvError;
use crate::entity::server_config::{ServerConfigJSON, ServerConfigJSONWithoutPassword, ServerConfigWithoutPassword};

// https://juejin.cn/post/7236591682615525431
async fn status_sse(TypedHeader(user_agent): TypedHeader<headers::UserAgent>) -> Sse<impl futures::stream::Stream<Item=Result<Event, Infallible>>> {
    println!("`{}` connected", user_agent.as_str());
    let btx = Arc::clone(&CHANNEL);
    let mut rx = btx.subscribe();

    let stream = async_stream::stream! {
         loop {
             let signal = rx.recv().await;
             yield signal
             };
         }
        .map(|signal| {
            let event = if let Ok(signal) = signal {
                println!("signal {:?}", signal);
                match signal {
                    CommandMessage::Get {key:keystr} =>{println!("GET key:{}", keystr);
                        Event::default()
                            .data("received data ".to_string() + &keystr)
                    }
                    CommandMessage::Set {config_index:index, key:keystr,  val:valbyte} =>{
                        println!("valbyte {:?}",valbyte);
                        let len_without_nulls = valbyte.iter().rposition(|&x| x != 0).map_or(0, |pos| pos + 1);
                        let valstr = String::from_utf8(Vec::from(valbyte[0..len_without_nulls].to_vec())).unwrap();
                        println!("Set config_index:{} key:{},value:{}",index, keystr,valstr);
                        let mut substring_after_update: Option<&str> = None;
                        // if let Some(start_index) = valstr.find("update") {
                        //     let substring = &valstr[start_index+"update".len()..];
                        // } else {
                        //     None
                        // }
                        if let Some(start_index) = valstr.find("update") {
                            substring_after_update = Some(&valstr[(start_index + "update".len())..]);
                        }

                        let datasubstring = match substring_after_update {
                            Some(datasubstring) => datasubstring,
                            None => &valstr
                        };
                        let data_to_send = format!("received data {{\"index\":{0},\"data\":{1} }}", index,datasubstring);
                        Event::default()
                            .data(data_to_send)
                    }
                }
            } else {
                Event::default().data(format!("None"))
            };
            println!("发送event: {:?}", event);
            event
        })
        .map(Ok)
        .throttle(Duration::from_secs(1));
    Sse::new(stream)
        .keep_alive(axum::response::sse::KeepAlive::new().interval(Duration::from_secs(15)))
}


async fn sse_handler(
    TypedHeader(user_agent): TypedHeader<headers::UserAgent>,
) -> Sse<impl Stream<Item=Result<Event, Infallible>>> {
    println!("`{}` connected", user_agent.as_str());

    // A `Stream` that repeats an event every second
    let stream = stream::repeat_with(|| Event::default().data("hi!"))
        .map(Ok)
        .throttle(Duration::from_secs(1));

    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(1))
            .text("keep-alive-text"),
    )
}

async fn sse_html() -> impl IntoResponse {
    Html(
        r#"
        <!DOCTYPE html>
<html lang="en">
<head>
   <meta charset="UTF-8">
   <title>Server-Sent Events client example with EventSource</title>
</head>
<body>
<script>
   if (window.EventSource == null) {
       alert('The browser does not support Server-Sent Events');
   } else {
       var eventSource = new EventSource('http://127.0.0.1:3000/status/statussse');

       eventSource.onopen = function () {
           console.log('connection is established');
       };

       eventSource.onerror = function (error) {
           console.log('connection state: ' + eventSource.readyState + ', error: ' + event);
       };

       eventSource.onmessage = function (event) {
           console.log('id: ' + event.lastEventId + ', data: ' + event.data);

           if (event.data.endsWith('.')) {
               eventSource.close();
               console.log('connection is closed');
           }
       };
   }
</script>
</body>
</html>

      "#
    )
}

async fn server_config() -> Json<ServerConfigJSONWithoutPassword> {
    let server_config_json = entity::server_config::ServerConfigWithoutPassword::build_server_config_json();
    return Json(server_config_json)
}

pub fn get_status_route() -> Router {
    let server_status_routes = Router::new()
        .route("/", post(|| async {}))
        .route("/page", get(sse_html))
        .route("/sse", get(sse_handler))
        .route("/statussse", get(status_sse))
        .route("/server", get(server_config));
    server_status_routes
}