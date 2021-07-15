mod config;
mod database;
mod model;


use std::collections::{BTreeMap, HashMap};
use std::convert::Infallible;
use std::error::Error;
use std::ffi::OsString;
use std::fmt;
use std::fs::File;
use std::io::{self, Read};
use std::net::{AddrParseError, SocketAddr};
use std::path::PathBuf;
use std::result::Result;

use chrono::{Duration, Local, Timelike};
use env_logger;
use form_urlencoded;
use http::request::Parts;
use hyper::{Body, Method, Request, Response, Server};
use hyper::body;
use hyper::service::{make_service_fn, service_fn};
use log::error;
use once_cell::sync::{Lazy, OnceCell};
use regex::Regex;
use tera::{Context, Tera};
use tokio::sync::RwLock;
use toml;
use url::Url;

use crate::config::{CONFIG, CONFIG_PATH, load_config};
use crate::database::{add_measurement, get_recent_measurements};
use crate::model::{DailyMeasurements, Measurement};


static TERA: OnceCell<RwLock<Tera>> = OnceCell::new();
static STATIC_PATH_RE: Lazy<Regex> = Lazy::new(|| Regex::new("^/static/([a-z0-9-._]+)$").unwrap());


#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct MissingValueError(String);
impl fmt::Display for MissingValueError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "missing value for key {}", self.0)
    }
}
impl Error for MissingValueError {
}


#[derive(Debug)]
pub(crate) enum ServerError {
    OpeningConfigFile(std::io::Error),
    ReadingConfigFile(std::io::Error),
    ParsingConfigFile(toml::de::Error),
    ParsingListenAddress(AddrParseError),
    HyperError(hyper::Error),
    TemplatingSetup(tera::Error),
}
impl fmt::Display for ServerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ServerError::OpeningConfigFile(e)
                => write!(f, "error opening config file: {}", e),
            ServerError::ReadingConfigFile(e)
                => write!(f, "error reading config file: {}", e),
            ServerError::ParsingConfigFile(e)
                => write!(f, "error parsing config file: {}", e),
            ServerError::ParsingListenAddress(e)
                => write!(f, "error parsing listen address: {}", e),
            ServerError::HyperError(e)
                => write!(f, "hyper error: {}", e),
            ServerError::TemplatingSetup(e)
                => write!(f, "error setting up templating: {}", e),
        }
    }
}
impl Error for ServerError {
}


#[derive(Debug)]
pub(crate) enum ClientError {
    MissingValue(String),
    FailedToParseIntValue(String, String, std::num::ParseIntError),
    ValueZeroOrLess(String, i32),
    ValueTooHigh(String, i32, i32),
}
impl fmt::Display for ClientError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ClientError::MissingValue(key)
                => write!(f, "missing value for key: {}", key),
            ClientError::FailedToParseIntValue(key, value, err)
                => write!(f, "failed to parse value {:?} for key {:?}: {}", value, key, err),
            ClientError::ValueZeroOrLess(key, value)
                => write!(f, "value {} for key {:?} is zero or less", value, key),
            ClientError::ValueTooHigh(key, value, max)
                => write!(f, "value {} for key {:?} is too high (> {})", value, key, max),
        }
    }
}
impl Error for ClientError {
}


async fn render_template(template_name: &str, context: &Context) -> Result<Body, tera::Error> {
    let template_string = {
        TERA.get()
            .expect("template engine is set")
            .read()
            .await
            .render(template_name, context)?
    };
    let body = Body::from(template_string);
    Ok(body)
}

async fn respond_template(
    template_name: &str,
    context: &Context,
    status: u16,
    headers: &HashMap<String, String>,
) -> Result<Response<Body>, Infallible> {
    let body = match render_template(template_name, context).await {
        Ok(b) => b,
        Err(e) => {
            error!("failed to render template: {:?}", e);
            return respond_500();
        },
    };

    let mut response_builder = Response::builder()
        .status(status)
        .header("Content-Type", "text/html; charset=utf-8");

    for (key, value) in headers {
        response_builder = response_builder.header(key, value);
    }

    let response = match response_builder.body(body) {
        Ok(r) => r,
        Err(e) => {
            error!("failed to create response: {}", e);
            return respond_500();
        }
    };
    Ok(response)
}

fn respond_500() -> Result<Response<Body>, Infallible> {
    let body = Body::from(String::from(
        r#"<!DOCTYPE html>
<html xmlns="http://www.w3.org/1999/xhtml" lang="en" xml:lang="en">
<head>
<meta charset="utf-8" />
<title>Internal Server Error</title>
</head>
<body>
<h1>Internal Server Error</h1>
<p>Something went wrong. It's not your fault. Tell the people responsible to check the logs.</p>
</body>
</html>"#
    ));

    // can't do much except unwrap/expect here, as this *is* the error handler
    let response = Response::builder()
        .header("Content-Type", "text/html; charset=utf-8")
        .body(body)
        .expect("failed to create response");
    Ok(response)
}

async fn respond_400(err: ClientError) -> Result<Response<Body>, Infallible> {
    let mut context = Context::new();
    context.insert("error", &err.to_string());

    respond_template(
        "400.html.tera",
        &context,
        400,
        &HashMap::new(),
    ).await
}

async fn respond_403() -> Result<Response<Body>, Infallible> {
    respond_template(
        "403.html.tera",
        &Context::new(),
        403,
        &HashMap::new(),
    ).await
}

async fn respond_404() -> Result<Response<Body>, Infallible> {
    respond_template(
        "404.html.tera",
        &Context::new(),
        404,
        &HashMap::new(),
    ).await
}

async fn respond_405(allowed_methods: &[Method]) -> Result<Response<Body>, Infallible> {
    let methods: Vec<String> = allowed_methods.iter()
        .map(|m| m.to_string())
        .collect();
    let joined_methods = methods.join(", ");

    let mut context = Context::new();
    context.insert("allowed_methods", &methods);

    let mut headers = HashMap::new();
    headers.insert(String::from("Allow"), joined_methods);

    respond_template(
        "405.html.tera",
        &context,
        405,
        &headers,
    ).await
}

async fn redirect_to_self(parts: Parts) -> Result<Response<Body>, Infallible> {
    let req_uri_string = parts.uri.to_string();
    let req_uri_noslash = req_uri_string.trim_start_matches('/');

    let base_uri: Url = {
        let base_uri_str = &CONFIG
            .get().expect("cannot get config")
            .read().await
            .base_url;
        match base_uri_str.parse() {
            Ok(bus) => bus,
            Err(e) => {
                error!("failed to parse URI {:?}: {}", base_uri_str, e);
                return respond_500();
            },
        }
    };
    let page_uri = match base_uri.join(&req_uri_noslash) {
        Ok(pu) => pu,
        Err(e) => {
            error!("failed to join {} and {}: {}", base_uri, req_uri_noslash, e);
            return respond_500();
        }
    };
    let page_uri_string = page_uri.to_string();

    let mut context = Context::new();
    context.insert("url", &page_uri_string);

    let mut headers = HashMap::new();
    headers.insert(String::from("Location"), page_uri_string);

    respond_template(
        "redirect.html.tera",
        &context,
        302,
        &headers,
    ).await
}

async fn get_index() -> Result<Response<Body>, Infallible> {
    let mut recent_measurements = match get_recent_measurements(Duration::days(3*31)).await {
        Ok(rm) => rm,
        Err(e) => {
            error!("error obtaining recent measurements: {}", e);
            return respond_500();
        },
    };
    recent_measurements.sort_by_key(|m| m.timestamp);

    // group measurements by day
    let hours = {
        let config_guard = CONFIG
            .get().unwrap()
            .read().await;
        config_guard.hours
    };
    let mut day_to_measurements: BTreeMap<String, DailyMeasurements> = BTreeMap::new();
    for measurement in recent_measurements.drain(..) {
        let mut day = measurement.timestamp.date().naive_local();
        if measurement.timestamp.hour() < hours.morning_start {
            // count this as (the evening of) the previous day
            day = day.pred();
        }

        let date_string = day.format("%Y-%m-%d").to_string();

        let entry = day_to_measurements.entry(date_string.clone())
            .or_insert_with(|| DailyMeasurements::new_empty(date_string));

        let this_hour = measurement.timestamp.hour();

        if this_hour < hours.morning_start && entry.evening.is_none() {
            // night (previous day)
            entry.evening = Some(measurement);
        } else if this_hour >= hours.morning_start && this_hour < hours.morning_end && entry.morning.is_none() {
            // morning
            entry.morning = Some(measurement);
        } else if this_hour >= hours.midday_start && this_hour < hours.midday_end && entry.midday.is_none() {
            // midday
            entry.midday = Some(measurement);
        } else if this_hour >= hours.evening_start && entry.evening.is_none() {
            // night
            entry.evening = Some(measurement);
        } else {
            entry.other.push(measurement);
        }
    }

    let days_and_measurements: Vec<DailyMeasurements> = day_to_measurements
        .values()
        .rev()
        .map(|v| v.clone())
        .collect();

    let mut context = Context::new();
    context.insert("days_and_measurements", &days_and_measurements);

    respond_template(
        "list.html.tera",
        &context,
        200,
        &HashMap::new(),
    ).await
}

fn get_form_i32_gt0(req_kv: &HashMap<String, String>, key: &str) -> Result<Option<i32>, ClientError> {
    let string_value = match req_kv.get(key) {
        Some(sv) => sv,
        None => return Ok(None),
    };
    let i32_value: i32 = string_value.parse()
        .map_err(|e| ClientError::FailedToParseIntValue(String::from(key), string_value.clone(), e))?;
    if i32_value < 0 {
        Err(ClientError::ValueZeroOrLess(String::from(key), i32_value))
    } else {
        Ok(Some(i32_value))
    }
}

fn get_req_form_i32_gt0(req_kv: &HashMap<String, String>, key: &str) -> Result<i32, ClientError> {
    match get_form_i32_gt0(req_kv, key) {
        Ok(Some(i)) => Ok(i),
        Ok(None) => Err(ClientError::MissingValue(String::from(key))),
        Err(e) => Err(e),
    }
}

fn get_measurement_from_form(req_kv: &HashMap<String, String>) -> Result<Measurement, ClientError> {
    let systolic: i32 = get_req_form_i32_gt0(&req_kv, "systolic")?;
    let diastolic: i32 = get_req_form_i32_gt0(&req_kv, "diastolic")?;
    let pulse: i32 = get_req_form_i32_gt0(&req_kv, "pulse")?;
    let spo2: Option<i32> = get_form_i32_gt0(&req_kv, "spo2")?;

    if let Some(sat) = spo2 {
        if sat > 100 {
            return Err(ClientError::ValueTooHigh("spo2".into(), sat, 100));
        }
    }

    let local_now = Local::now();
    let measurement = Measurement::new(
        -1,
        local_now,
        systolic,
        diastolic,
        pulse,
        spo2,
    );
    Ok(measurement)
}

async fn post_index(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let (req_parts, req_body) = req.into_parts();
    let req_body_bytes = match body::to_bytes(req_body).await {
        Ok(rbb) => rbb,
        Err(e) => {
            error!("error reading request bytes: {}", e);
            return respond_500();
        },
    }.to_vec();
    let req_kv: HashMap<String, String> = form_urlencoded::parse(&req_body_bytes)
        .map(|(a, b)| (a.to_string(), b.to_string()))
        .collect();

    let mut new_measurement = match get_measurement_from_form(&req_kv) {
        Ok(nm) => nm,
        Err(e) => {
            return respond_400(e).await;
        },
    };

    match add_measurement(&mut new_measurement).await {
        Ok(rm) => rm,
        Err(e) => {
            error!("error adding measurement: {}", e);
            return respond_500();
        },
    };

    redirect_to_self(req_parts).await
}

async fn respond_static_file(file_name: &str) -> Result<Response<Body>, Infallible> {
    let mime_type = if file_name.ends_with(".css") {
        "text/css"
    } else if file_name.ends_with(".js") {
        "text/javascript"
    } else if file_name.ends_with(".jpg") || file_name.ends_with(".jpeg") {
        "image/jpeg"
    } else if file_name.ends_with(".png") {
        "image/png"
    } else if file_name.ends_with(".txt") {
        "text/plain; charset=utf-8"
    } else {
        "application/octet-stream"
    };

    let file_path: PathBuf = ["static", file_name].iter().collect();
    let mut file = match File::open(&file_path) {
        Ok(f) => f,
        Err(e) => {
            if e.kind() == io::ErrorKind::NotFound {
                return respond_404().await;
            } else {
                error!("error opening file {:?}: {}", file_path, e);
                return respond_500();
            }
        },
    };
    let mut buf = Vec::new();
    if let Err(e) = file.read_to_end(&mut buf) {
        error!("error reading file {:?}: {}", file_path, e);
        return respond_500();
    }

    let response_res = Response::builder()
        .header("Content-Length", format!("{}", buf.len()))
        .header("Content-Type", mime_type)
        .body(Body::from(buf));
    match response_res {
        Ok(r) => Ok(r),
        Err(e) => {
            error!("failed to create response: {}", e);
            return respond_500();
        }
    }
}

async fn handle_request(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    if let Some(cap) = STATIC_PATH_RE.captures(req.uri().path()) {
        let static_file_name = cap.get(1).expect("filename captured");
        return respond_static_file(static_file_name.as_str()).await;
    }

    // endpoints that do not require authentication before this line

    // check for token
    let query_str = match req.uri().query() {
        None => return respond_403().await,
        Some(q) => q,
    };
    let query_kv: HashMap<String, String> = form_urlencoded::parse(query_str.as_bytes())
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();
    let token_value = match query_kv.get("token") {
        None => return respond_403().await,
        Some(tv) => tv,
    };

    let token_matches = {
        CONFIG
            .get().expect("config is set")
            .read().await
            .auth_tokens
            .iter()
            .any(|t| t == token_value)
    };
    if !token_matches {
        return respond_403().await;
    }

    // authenticated-only endpoints beyond this line

    if req.uri().path() == "/" {
        if req.method() == Method::GET {
            get_index().await
        } else if req.method() == Method::POST {
            post_index(req).await
        } else {
            respond_405(&[Method::GET, Method::POST]).await
        }
    } else {
        respond_404().await
    }
}

async fn run() -> Result<(), ServerError> {
    env_logger::init();

    let args: Vec<OsString> = std::env::args_os().collect();
    let config_path = match args.get(1) {
        Some(cp) => PathBuf::from(cp),
        None => PathBuf::from("config.toml"),
    };
    CONFIG_PATH
        .set(config_path).expect("failed to set config path");

    load_config().await?;

    let tera = Tera::new("templates/**/*")
        .map_err(|e| ServerError::TemplatingSetup(e))?;
    TERA
        .set(RwLock::new(tera)).expect("failed to set templating engine");

    let addr: SocketAddr = {
        CONFIG
            .get().expect("no config lock")
            .read().await
            .http_listen
            .parse()
            .map_err(|e| ServerError::ParsingListenAddress(e))?
    };

    let make_service = make_service_fn(|_conn| async {
        Ok::<_, Infallible>(service_fn(handle_request))
    });

    let server = Server::bind(&addr).serve(make_service);
    server.await
        .map_err(|e| ServerError::HyperError(e))
}

fn main() {
    let result = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async move {
            run().await
        });

    std::process::exit(match result {
        Ok(()) => 0,
        Err(e) => {
            eprintln!("{}", e);
            1
        },
    });
}
