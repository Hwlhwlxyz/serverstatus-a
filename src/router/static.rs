use std::error::Error;
use axum::body::Body;
use axum::http::{Request, Uri};
use tower::ServiceExt;
use std::path::PathBuf;
use std::fmt::Display;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use std::path::{Path};
use axum::body::{BoxBody, boxed};

use axum::response::Response;

use tower_http::services::ServeFile;

fn get_possible_path_from_uri_string(mut uri: &str) -> Result<String, Box<dyn Error>> {
    // uri.remove(0); // remove  first '/'
    let uri_without_params = if let Some(query_start_index) = uri.find('?') {
        uri[..query_start_index].to_string()
    }
    else { String::from(uri) };

    let path_list = [
        Path::new(&uri_without_params).to_path_buf(),
        Path::new(&[&uri_without_params,".html"].join("")).to_path_buf(),
    ];
    for possible_path in path_list {
        println!("possible_path:{:?}", possible_path);
        if possible_path.is_file() {
            return match possible_path.into_os_string().into_string() {
                Ok(string) => Ok(string),
                Err(_os_string) => Err(Box::try_from("NOT_FOUND").unwrap())
            }
        }

    }
    Err(Box::try_from("NOT_FOUND").unwrap())
}

async fn get_static_file(uri: Uri) -> Result<Response<BoxBody>, (StatusCode, String)> {
    let uri_string = uri.to_string();
    let req = Request::builder().uri(&uri_string).body(Body::empty()).unwrap();

    // `ServeDir` implements `tower::Service` so we can call it with `tower::ServiceExt::oneshot`
    let mut full_path = PathBuf::from(format!("static{}", uri));


    println!("full_path: {:?} uri_string: {:?}", full_path.to_str(), uri_string);
    let found_file_path = match get_possible_path_from_uri_string(full_path.to_str().unwrap())  {
        Ok(got_path) => got_path,
        Err(err) => return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {}", err.to_string()),
        ))
    };
    println!("found_file_path: {:?}", found_file_path);
    match ServeFile::new(found_file_path).oneshot(req).await {
        Ok(res) => Ok(res.map(boxed)),
        Err(err) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {}", err),
        )),
    }
}

pub async fn html_handler(uri: Uri) -> Response {
    println!("uri: {}", uri);
    match get_static_file(uri.clone()).await {
        Ok(res) => res,
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Server failed with {:?}", err),
        ).into_response()
    }
}