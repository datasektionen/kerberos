use std::io::Write;

pub fn send_card_or_onboard(
    client: &reqwest::blocking::Client,
    server_url: &str,
    key: &str,
    uid: String,
    onboard_only: bool,
) -> Result<(), std::io::Error> {
    let req = if onboard_only {
        client
            .post(format!("{}/onboard", server_url))
            .bearer_auth(key)
            .body(uid.clone())
    } else {
        client
            .post(format!("{}/card", server_url))
            .bearer_auth(key)
            .body(uid.clone())
    };
    match req.send() {
        Ok(s) => match s.status() {
            reqwest::StatusCode::OK => {
                log::info!("Successfully sent card event to server");
            }
            reqwest::StatusCode::UNPROCESSABLE_ENTITY => {
                log::error!("Card not found on server, onboarding...");
                onboard_card(client, server_url, key, uid, false)?;
            }
            s => {
                log::error!("Failed to send card event to server: HTTP {}", s);
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
) -> Result<(), std::io::Error> {
    println!("Enter the kthid for the card (e.g. \"turetek\"): ");
    std::io::stdout().flush().expect("Failed to flush stdout");
    let mut buf = String::new();
    std::io::stdin()
        .read_line(&mut buf)
        .expect("Failed to read input from stdin");
    let kthid = buf.trim();

    let url = if onboard_only {
        format!("{}/onboard", server_url)
    } else {
        format!("{}/card?onboard=1", server_url)
    };

    match client
        .post(url)
        .bearer_auth(key)
        .body(format!("{}:{}", kthid, uid))
        .send()
    {
        Ok(s) => match s.status() {
            reqwest::StatusCode::OK => {
                log::info!("Successfully onboarded card");
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
