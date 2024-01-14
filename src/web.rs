use std::{
    num::NonZeroU32,
    sync::{Arc, Mutex},
};

use anyhow::Result;
use esp_idf_svc::{
    hal::{gpio::Level, task::notification::Notifier},
    http::{
        server::{Configuration, EspHttpServer},
        Method,
    },
    io::Write,
};

use crate::app_state::AppState;

fn content(content: impl AsRef<str>) -> String {
    format!(
        r#"
      <main>
        {}
      </main>
      <button hx-post="/toggle" hx-push-url="false" hx-target="body">
        toggle
      </button>
"#,
        content.as_ref()
    )
}

fn template(c: impl AsRef<str>) -> String {
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
        content(c)
    )
}

fn index_html(level: Level, only_body: bool) -> String {
    let state = match level {
        Level::High => "on",
        Level::Low => "off",
    };
    let c = format!("Led is {}", state);
    if only_body {
        content(c)
    } else {
        template(c)
    }
}

fn register_index(app_state: Arc<Mutex<AppState>>, server: &mut EspHttpServer<'_>) -> Result<()> {
    server.fn_handler("/", Method::Get, move |request| {
        let current_level = { app_state.clone().lock().unwrap().led };
        let html = index_html(current_level, false);
        let mut response = request.into_ok_response()?;
        response.write_all(html.as_bytes())?;
        Ok(())
    })?;

    Ok(())
}

fn register_toggle(
    app_state: Arc<Mutex<AppState>>,
    notifier: Arc<Notifier>,
    server: &mut EspHttpServer<'_>,
) -> Result<()> {
    server.fn_handler("/toggle", Method::Post, move |request| {
        let current_level = { !app_state.clone().lock().unwrap().led };
        unsafe {
            notifier.notify_and_yield(NonZeroU32::new(3).unwrap());
        }
        let html = index_html(current_level, true);
        let mut response = request.into_ok_response()?;
        response.write_all(html.as_bytes())?;
        Ok(())
    })?;

    Ok(())
}

pub fn server<'a>(
    app_state: Arc<Mutex<AppState>>,
    notifier: Arc<Notifier>,
) -> Result<EspHttpServer<'a>> {
    let mut server = EspHttpServer::new(&Configuration::default())?;

    register_index(app_state.clone(), &mut server)?;
    register_toggle(app_state.clone(), notifier.clone(), &mut server)?;

    Ok(server)
}
