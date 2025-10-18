use reqwest::Client;
use serde_json::json;

use crate::errors::{AppError, AppResult};

const USERNAME: &str = "user@gmail.com";
const USERPASS: &str = "qwerty1234!";

pub async fn sync_database(ip: String, db_name: &str) -> AppResult<()> {
    let base_url = format!("http://{}:3000", ip);
    let client = Client::new();
    let login_res: serde_json::Value = client
        .post(format!("{}/api/session", base_url))
        .json(&serde_json::json!({
            "username": USERNAME,
            "password": USERPASS
        }))
        .send()
        .await?
        .json()
        .await?;

    let token = login_res["id"].as_str().unwrap();
    let dbs: serde_json::Value = client
        .get(format!("{}/api/database", base_url))
        .header("X-Metabase-Session", token)
        .send()
        .await?
        .json()
        .await?;

    let db = dbs
        .get("data")
        .ok_or(AppError::MT("Невірний формат"))?
        .as_array()
        .ok_or(AppError::MT("Невірний формат"))?;
    let db = db
        .iter()
        .find(|db| db["name"].as_str().unwrap() == db_name)
        .ok_or(AppError::MT("Невдалося знайти базу даних"))?;
    let db_id = db.get("id").unwrap().as_i64().unwrap();

    let _ = client
        .post(format!("{}/api/database/{}/sync_schema", base_url, db_id))
        .header("X-Metabase-Session", token)
        .send()
        .await?;

    Ok(())
}

pub async fn add_db(ip: &str) -> AppResult<()> {
    let base_url = format!("http://{}:3000", ip);
    let client = Client::new();

    let login_res: serde_json::Value = client
        .post(format!("{}/api/session", base_url))
        .json(&serde_json::json!({
            "username": USERNAME,
            "password": USERPASS
        }))
        .send()
        .await?
        .json()
        .await?;
    let token = login_res["id"]
        .as_str()
        .ok_or(AppError::MT("Невірний формат"))?;
    println!("Got Metabase token: {}", token);

    // Build the request body
    let body = json!({
        "engine": "mysql",
        "name": "mydb",
        "details": {
            "host": ip,
            "port": 3306,
            "dbname": "mydb",
            "user": "user",
            "password": "password",
            "ssl": false
        },
        "is_full_sync": true,
        "is_on_demand": false,
        "schedules": {}
    });

    let res = client
        .post("http://localhost:3000/api/database") // Metabase API URL
        .header("Content-Type", "application/json")
        .header("X-Metabase-Session", token)
        .json(&body)
        .send()
        .await?;

    println!("Status: {}", res.status());
    Ok(())
}

pub async fn create_first_user(ip: &str) -> AppResult<()> {
    let base_url = format!("http://{}:3000", ip);

    let client = Client::new();
    // First get the setup token
    let props: serde_json::Value = client
        .get(format!("{}/api/session/properties", base_url))
        .send()
        .await?
        .json()
        .await?;

    let setup_token = props["setup-token"]
        .as_str()
        .ok_or(AppError::MT("Невірний формат"))?;

    let setup_body = json!({
        "prefs": {
            "site_name": "My Metabase",
            "site_locale": "uk"
        },
        "user": {
            "first_name": "Admin",
            "last_name": "User",
            "email": USERNAME,
            "password": USERPASS,
            "site_name": "My Metabase"
        },
        "database": {

        },
        "token": setup_token
    });

    let resp: serde_json::Value = client
        .post(format!("{}/api/setup", base_url))
        .json(&setup_body)
        .send()
        .await?
        .json()
        .await?;

    println!("Setup response: {:#}", resp);
    Ok(())
}
