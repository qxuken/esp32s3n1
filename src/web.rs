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

// Max payload length
const MAX_LEN: usize = 128;

fn template(rgb: Rgb, flash: Option<String>) -> String {
    format!(
        r#"
<!DOCTYPE html>
<html>
  <head>
      <meta charset="utf-8">
      <title>esp-rs web server</title>
      <script src="https://unpkg.com/htmx.org@1.9.10"></script>
  </head>
  <body hx-boost="true">
{}
  </body>
</html>
"#,
        content(rgb, flash)
    )
}

fn flash_component(text: &str) -> String {
    format!(
        r#"
        <h5>
        {}
        </h5>
"#,
        text
    )
}

fn content(rgb: Rgb, flash: Option<String>) -> String {
    format!(
        r##"
{}
<form hx-post="/set_rgb" hx-push-url="false" hx-target="body">

    <div>
        <label for="onboard_led">RGB Led color</label>
        <input type="color" id="onboard_led" name="onboard_led" value="{}" />
    </div>

    <input type="submit" value="Update" />
</form>
"##,
        flash.map(|f| flash_component(&f)).unwrap_or_default(),
        rgb.to_css_hex_string()
    )
}

fn index_html(rgb: Rgb, flash: Option<String>, only_body: bool) -> String {
    if only_body {
        content(rgb, flash)
    } else {
        template(rgb, flash)
    }
}

fn register_index<'a>(
    app_state: Arc<Mutex<AppState<'a>>>,
    server: &mut EspHttpServer<'a>,
) -> Result<()> {
    server.fn_handler("/", Method::Get, move |request| {
        let rgb_val = { app_state.clone().lock().unwrap().rgb_val };
        let html = index_html(rgb_val, None, false);
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
                Some("Updated".to_string())
            }
            Err(e) => {
                log::error!("{:?}", e);
                let message = format!("Bad Data: {}", e);
                Some(message)
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
