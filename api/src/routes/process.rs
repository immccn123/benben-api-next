use dxr::{TryFromValue, TryToValue};
use dxr_client::Call;
use lazy_static::lazy_static;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::err::AppError;

lazy_static! {
    static ref SUPERVISIOR: dxr_client::Client = {
        dxr_client::ClientBuilder::new(
            Url::parse(
                std::env::var("PROC_XMLRPC_URL")
                    .expect("PROC_XMLRPC_URL is not set")
                    .as_str(),
            )
            .expect("PROC_XMLRPC_URL is not a vaild url"),
        )
        .user_agent("benben-api")
        .build()
    };
}

#[derive(Debug, Clone, Deserialize, Serialize, TryToValue, TryFromValue)]
struct ProcessInfo {
    name: String,
    group: String,
    description: String,
    start: i32,
    stop: i32,
    now: i32,
    state: i32,
    statename: String,
    spawnerr: String,
    exitstatus: i32,
    stdout_logfile: String,
    stderr_logfile: String,
    pid: i32,
}

#[get("/proc/status")]
async fn get_proc_status() -> Result<serde_json::Value, AppError> {
    let ProcessInfo {
        statename: fetcher_status,
        ..
    } = SUPERVISIOR
        .call(Call::new("supervisor.getProcessInfo", "lgfeed-fetch"))
        .await?;

    let ProcessInfo {
        statename: loop_status,
        ..
    } = SUPERVISIOR
        .call(Call::new("supervisor.getProcessInfo", "lgfeed-loop"))
        .await?;

    Ok(json!({
        "fetcher_status": fetcher_status,
        "loop_status": loop_status,
    }))
}

#[get("/proc/start?<proc>")]
async fn restart_proc(proc: &str) -> Result<serde_json::Value, AppError> {
    let _: bool = match proc {
        "fetcher" => {
            SUPERVISIOR
                .call(Call::new("supervisor.startProcess", "lgfeed-fetch"))
                .await?
        }

        "loop" => {
            SUPERVISIOR
                .call(Call::new("supervisor.startProcess", "lgfeed-loop"))
                .await?
        }

        _ => return Err(AppError::ParamValidateFailed("ZmxhZ3tCVjFHSjQxMXg3aDd9")),
    };

    Ok(json!({ "result": "ok" }))
}

export_routes![get_proc_status, restart_proc];
