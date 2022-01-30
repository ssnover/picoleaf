use serde_json::json;
use std::path::PathBuf;
use casita::leap::{self, CommuniqueType};

mod config;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config_path = std::env::args().nth(1).unwrap_or_else(|| {eprintln!("Forgot your config file"); std::process::exit(1); });
    let config = config::load_config_from_path(&config_path)?;

    let certs = casita::Certs::new(
        PathBuf::from(config.caseta.ca_cert_path),
        PathBuf::from(config.caseta.cert_path),
        PathBuf::from(config.caseta.key_path),
    )?;
    let mut client = casita::Client::new(certs, format!("{}:8081", config.caseta.address)).await;
    let aurora = borealis::Aurora::new(format!("{}:16021", config.aurora.address), &config.aurora.token)?;

    loop {
        loop {
            if let Err(err) = client.connect().await {
                eprintln!("Unable to connect to Caseta: {}", err);
                tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
            } else {
                println!("Connected to Caseta!");
                break;
            }
        }
        if let Err(err) = subscribe_to_button_events(&mut client).await {
            eprintln!("Unable to subscribe to button events: {}", err);
            continue;
        } else {
            println!("Ready to process button events!");
        }

        if let Err(err) = handle_button_events(&mut client, &aurora).await {
            eprintln!("Error handling buttons: {}", err);
            println!("Let's try reconnecting");
            tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
        }
    }
}

async fn read_not_ping(
    client: &mut casita::Client,
) -> Result<leap::Message, Box<dyn std::error::Error>> {
    loop {
        let msg = client.read_message().await?;
        if let Ok(msg) = serde_json::from_value::<leap::Message>(msg) {
            if msg.header.url == "/device/status/deviceheard" {
                continue;
            } else {
                return Ok(msg);
            }
        }
    }
}

async fn subscribe_to_button_events(
    client: &mut casita::Client,
) -> Result<(), Box<dyn std::error::Error>> {
    let request_devices = json!({
        "CommuniqueType": "ReadRequest",
        "Header": {
            "ClientTag": "picoleaf",
            "Url": "/device"
        },
    });
    client.send_raw(request_devices).await?;

    let devices = loop {
        let response = read_not_ping(client).await?;
        if response.communique_type == CommuniqueType::ReadResponse && response.header.url == "/device" {
            if let Some(body) = response.body {
                let devices = body["Devices"].as_array().unwrap().clone();
                let device_hrefs = devices
                    .iter()
                    .filter(|dev| dev["DeviceType"] == "Pico3ButtonRaiseLower")
                    .map(|dev| dev["href"].as_str().unwrap().to_owned())
                    .collect::<Vec<String>>();
                break device_hrefs;
            }
        }
    };

    let mut all_button_hrefs = vec![];
    for device in devices {
        let url = format!("{}/buttongroup", device);
        let request = json!({
            "CommuniqueType": "ReadRequest",
            "Header": {
                "ClientTag": "picoleaf",
                "Url": url,
            },
        });
        client.send_raw(request).await?;
        let button_hrefs: Vec<String> = loop {
            let response = read_not_ping(client).await?;
            if response.communique_type == CommuniqueType::ReadResponse && response.header.url == url {
                if let Some(body) = response.body {
                    break body["ButtonGroups"][0]["Buttons"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|val| val["href"].as_str().unwrap().to_owned())
                        .collect();
                }
            }
        };
        all_button_hrefs.extend_from_slice(&button_hrefs);
    }

    for href in all_button_hrefs {
        let href = format!("{}/status/event", href);
        let request = leap::Message::new(CommuniqueType::SubscribeRequest, href);
        client.send(request).await?;
    }

    Ok(())
}

async fn handle_button_events(
    caseta: &mut casita::Client,
    aurora: &borealis::Aurora<'_>,
) -> Result<(), Box<dyn std::error::Error>> {
    let effects = aurora.get_effects().await?;
    let mut current_effect_idx = 0;
    loop {
        let msg = read_not_ping(caseta).await?;
        if msg.communique_type == CommuniqueType::UpdateResponse && msg.header.status_code.unwrap() == "200 OK" {
            let body = msg.body.unwrap();
            let href = body["ButtonStatus"]["Button"]["href"]
                .as_str()
                .unwrap()
                .to_owned();
            let id = href.split("/").last().unwrap().parse::<u32>().unwrap();
            if body["ButtonStatus"]["ButtonEvent"]["EventType"]
                .as_str()
                .unwrap()
                == "Release"
            {
                if id == 111 {
                    aurora.turn_on().await?;
                    aurora.set_effect("Working").await?;
                } else if id == 112 {
                    aurora.turn_on().await?;
                    aurora.set_effect("Hot Romance").await?;
                } else if id == 113 {
                    aurora.turn_off().await?;
                } else if id == 114 {
                    if current_effect_idx == effects.len() - 1 {
                        current_effect_idx = 0;
                    } else {
                        current_effect_idx += 1;
                    }
                    aurora.set_effect(&effects[current_effect_idx]).await?;
                } else if id == 115 {
                    if current_effect_idx == 0 {
                        current_effect_idx = effects.len() - 1;
                    } else {
                        current_effect_idx -= 1;
                    }
                    aurora.set_effect(&effects[current_effect_idx]).await?;
                }
            }
        }
    }
}
