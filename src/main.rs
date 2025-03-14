use dotenv::dotenv;
use std::env;
use std::path::Path;
use chrono::prelude::*;
use chrono::Local;
use ssh2::Session;
use std::net::TcpStream;
use std::io::prelude::*;
use std::fs::File;
use std::io::Read;
use glob::glob;
use tokio::time::{sleep, Duration};
use tokio::signal;
use reqwest::Client;

#[tokio::main]
async fn main() {
    dotenv().ok();

    let sftp_host = env::var("SFTP_HOST").expect("SFTP_HOST not set");
    let sftp_user = env::var("SFTP_USER").expect("SFTP_USER not set");
    let sftp_password = env::var("SFTP_PASSWORD").expect("SFTP_PASSWORD not set");
    let sftp_remote_path = env::var("SFTP_REMOTE_PATH").expect("SFTP_REMOTE_PATH not set");
    let local_folder = env::var("LOCAL_FOLDER").unwrap_or_else(|_| "./layers".to_string());
    let run_hour: u32 = env::var("RUN_HOUR").unwrap_or_else(|_| "99".to_string()).parse().expect("RUN_HOUR must be a number");
    let webhook_url = env::var("DISCORD_WEBHOOK_URL").unwrap_or_default();


    // Handle docker signals correctly
    tokio::spawn(async move {
        signal::ctrl_c().await.unwrap();
        println!("Received Ctrl-C, exiting");
        std::process::exit(0);
    });
    
    let tcp = TcpStream::connect(sftp_host).unwrap();
    let mut sess = Session::new().unwrap();
    sess.set_tcp_stream(tcp);
    sess.handshake().unwrap();
    sess.userauth_password(&sftp_user, &sftp_password).unwrap();

    let sftp = sess.sftp().unwrap();

    loop {
        let now = Local::now();
        if now.hour() == run_hour || run_hour == 99 {
            if let Some(file_path) = get_next_file(&local_folder) {
                upload_file(&sftp, &file_path, &sftp_remote_path, &webhook_url).await;
            }
            
            println!("Sleeping for 24 hours");
            sleep(Duration::from_secs(86400)).await;
        } else {
            println!("Current hour: {} Target run hour {}. Sleeping for 30 min", now.hour(), run_hour);
            sleep(Duration::from_secs(1800)).await;
        }
    }
}

fn get_next_file(folder: &str) -> Option<String> {
    let pattern = format!("{}/*", folder);
    let files: Vec<String> = glob(&pattern).unwrap().filter_map(Result::ok).map(|path| path.to_string_lossy().into_owned()).collect();
    if files.is_empty() {
        None
    } else {
        let day_of_year = Local::now().ordinal() as usize;
        let file_index = day_of_year % files.len();

        println!("Files to choose from {} chosen index {}. Day {}", files.len(), file_index, day_of_year);
        Some(files[file_index].clone())
    }
}

async fn upload_file(sftp: &ssh2::Sftp, local_path: &str, remote_path: &str, webhook_url: &str) {
    let mut file = File::open(local_path).unwrap();
    let mut contents = Vec::new();
    file.read_to_end(&mut contents).unwrap();


    let mut remote_file = sftp.create(Path::new(remote_path)).unwrap();
    remote_file.write_all(&contents).unwrap();
    println!("Uploaded {} to {}", local_path, remote_path);

    if webhook_url.is_empty() {
        println!("No webhook URL set, skipping discord message");
        return;
    }

    if let Err(err) = send_discord_message(&webhook_url, local_path, &contents).await {
        println!("Failed to send discord message: {}", err);
    }
}

async fn send_discord_message(webhook_url: &str, file_name: &str, content: &[u8]) -> Result<(), reqwest::Error> {
    let client = Client::new();

    let content_str = String::from_utf8_lossy(content);
    let lines: Vec<&str> = content_str.lines().filter(|line| !line.starts_with("//")).collect();

    let mut fields = vec![serde_json::json!({
        "name": "Mapas",
        "value": "",
        "inline": false
    })];

    fields.extend(lines.iter().map(|line| {
        serde_json::json!({
            "name": "",
            "value": line,
            "inline": false
        })
    }));

    let embed = serde_json::json!({
        "title": format!("Rotación: {}", file_name),
        "color": 5986018,
        "fields": fields
    });

    let payload = serde_json::json!({ "embeds": [embed] });

    client.post(webhook_url)
        .json(&payload)
        .send()
        .await?
        .error_for_status()?;

    Ok(())
}