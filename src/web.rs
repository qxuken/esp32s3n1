use std::sync::{Arc, Mutex};

use anyhow::Result;
use colors_transform::Rgb;
use esp_idf_svc::http::server::{Configuration, EspHttpServer};

use embedded_svc::{
    http::{Headers, Method},
    io::{Read, Write},
};
use serde::Deserialize;

use crate::app_state::AppState;
use maud::{html, Markup, DOCTYPE};

// Max payload length
const MAX_LEN: usize = 128;

fn template(rgb: Rgb, flash: Markup) -> Markup {
    html! {
        (DOCTYPE)
        html {
            head {
                meta charset="utf-8";
                title { "esp32s3" }
                script src="https://unpkg.com/htmx.org@1.9.10" {}
            }
            body hx-boost="true" {
                (content(rgb, flash))
            }
        }
    }
}

fn flash(text: &str) -> Markup {
    if text.is_empty() {
        return html! {};
    }
    html! {
        h5 { (text) }
    }
}

fn content(rgb: Rgb, flash: Markup) -> Markup {
    html! {
        (flash)
        form hx-post="/set_rgb" hx-target="body" hx-trigger="change delay:500ms" {
            div {
                label for="onboard_led" {
                    "RGB Lead color"
                }
                input type="color" id="onboard_led" name="onboard_led" value=(rgb.to_css_hex_string()) ;
            }
        }
    }
}

fn index_html(rgb: Rgb, flash: Markup, only_body: bool) -> String {
    if only_body {
        content(rgb, flash)
    } else {
        template(rgb, flash)
    }
    .into_string()
}

fn register_index<'a>(
    app_state: Arc<Mutex<AppState<'a>>>,
    server: &mut EspHttpServer<'a>,
) -> Result<()> {
    server.fn_handler("/", Method::Get, move |request| {
        let rgb_val = { app_state.clone().lock().unwrap().rgb_val };
        let html = index_html(rgb_val, flash(""), false);
        let mut response = request.into_ok_response()?;
        response.write_all(html.as_bytes())?;
        Ok(())
    })?;

    Ok(())
}

#[derive(Deserialize)]
struct FormData {
    onboard_led: String,
}

fn register_set<'a>(
    app_state: Arc<Mutex<AppState<'a>>>,
    server: &mut EspHttpServer<'a>,
) -> Result<()> {
    server.fn_handler("/set_rgb", Method::Post, move |mut request| {
        let len = request.content_len().unwrap_or(0) as usize;

        if len > MAX_LEN {
            request
                .into_status_response(413)?
                .write_all("Request too big".as_bytes())?;
            return Ok(());
        }

        let mut buf = vec![0; len];
        request.read_exact(&mut buf)?;

        let flash = match serde_urlencoded::from_bytes::<FormData>(&buf)
            .map_err(|e| anyhow::anyhow!(e))
            .and_then(|data| {
                colors_transform::Rgb::from_hex_str(&data.onboard_led)
                    .map_err(|_e| anyhow::anyhow!("color parse error"))
            }) {
            Ok(rgb) => {
                app_state.clone().lock().unwrap().set_rgb(rgb)?;
                flash(&format!("New color: {}", rgb.to_css_hex_string()))
            }
            Err(e) => {
                log::error!("{:?}", e);
                flash(&format!("Bad Data: {}", e))
            }
        };

        let rgb_val = { app_state.clone().lock().unwrap().rgb_val };

        let html = index_html(rgb_val, flash, false);
        let mut response = request.into_ok_response()?;
        response.write_all(html.as_bytes())?;

        Ok(())
    })?;

    Ok(())
}

pub fn server(app_state: Arc<Mutex<AppState>>) -> Result<EspHttpServer> {
    let mut server = EspHttpServer::new(&Configuration::default())?;

    register_index(app_state.clone(), &mut server)?;
    register_set(app_state.clone(), &mut server)?;

    Ok(server)
}
