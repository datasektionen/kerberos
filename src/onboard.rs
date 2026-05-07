use std::io::Write;

use crate::cli;

pub fn send_card_or_onboard(
    client: &reqwest::blocking::Client,
    server_url: &str,
    key: &str,
    uid: String,
    onboard_only: bool,
    mut file: Option<&mut std::fs::File>,
) -> Result<(), std::io::Error> {
    if let Some(file) = file.as_mut() {
        writeln!(file, "{}", uid)?;
        log::info!("Wrote card UID to file: {}", uid);
    }

    if onboard_only {
        let addr = format!("{}/onboard", server_url);
        log::info!("Onboard only mode enabled, sending card to {}", addr);
        onboard_card(client, &addr, key, uid.clone(), true, file)?;
        return Ok(());
    }

    match client
        .post(format!("{}/card", server_url))
        .body(uid.clone())
        .send()
    {
        Ok(s) => match s.status() {
            reqwest::StatusCode::OK => {
                log::info!("Successfully sent card event to server");
            }
            reqwest::StatusCode::UNPROCESSABLE_ENTITY => {
                log::error!("Card not found on server, onboarding...");
                onboard_card(client, server_url, key, uid, false, file)?;
            }
            s => {
                log::error!("Status code: Failed to send card event to server: {}", s);
            }
        },
        Err(e) => {
            log::error!("Failed to send card event to server: {}", e);
        }
    }
    Ok(())
}

pub fn onboard_card(
    client: &reqwest::blocking::Client,
    server_url: &str,
    key: &str,
    uid: String,
    onboard_only: bool,
    file: Option<&mut std::fs::File>,
) -> Result<(), std::io::Error> {
    println!("Enter the kthid for the card (e.g. \"turetek\"): ");
    std::io::stdout().flush().expect("Failed to flush stdout");
    let mut buf = String::new();
    std::io::stdin()
        .read_line(&mut buf)
        .expect("Failed to read input from stdin");
    let kthid = buf.trim();

    let url = if onboard_only {
        format!("{}", server_url)
    } else {
        format!("{}/card?onboard=1", server_url)
    };

    log::info!("onboard card url: {}", url);

    if let Some(file) = file {
        writeln!(file, "{}#{}", kthid, uid)?;
        log::info!("Wrote card info to file: {}#{}", kthid, uid);
    }

    match client
        .post(url)
        .bearer_auth(key)
        .body(format!("{}#{}", kthid, uid))
        .send()
    {
        Ok(s) => match s.status() {
            reqwest::StatusCode::OK => {
                log::info!("Successfully onboarded card");
            }
            reqwest::StatusCode::CONFLICT => {
                log::error!("Card with same UID already exists on server");
                let body = s
                    .text()
                    .unwrap_or_else(|_| "Failed to read response body".to_string());
                log::error!("Card conflict server response: {}", body);
                println!("Card with same UID already exists: {}", body);
            }
            s => {
                log::error!("Failed to onboard card: HTTP {}", s);
            }
        },
        Err(e) => {
            log::error!("Failed to onboard card: {}", e);
        }
    }
    Ok(())
}
