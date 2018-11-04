#![feature(await_macro, async_await, futures_api)]

extern crate futures;
extern crate gotham;
extern crate hyper;
extern crate hyper_tls;
extern crate mime;
extern crate pretty_env_logger;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
extern crate tokio_core;
extern crate url;

use gotham::router::builder::{build_simple_router, DefineSingleRoute, DrawRoutes};
use gotham::router::Router;
use hyper::Method;

use std::collections::HashMap;
use std::env;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

mod facebook_app;
mod games;
mod receive;
mod verification;

use crate::facebook_app::{FacebookApp, FacebookPage};

pub fn get_app() -> FacebookApp {
    let app_secret = env::var("APP_SECRET").unwrap_or(String::new());
    let webhook_verify_token = env::var("WEBHOOK_VERIFY_TOKEN").unwrap_or(String::new());

    let mut page_config = HashMap::new();
    page_config.insert(
        env::var("ECHO_PAGE_ID").unwrap_or(String::new()),
        FacebookPage::new(
            env::var("ECHO_ACCESS_TOKEN").unwrap_or(String::new()),
            Some(games::echo::echo_message),
        ),
    );
    page_config.insert(
        env::var("PREFIX_PAGE_ID").unwrap_or(String::new()),
        FacebookPage::new(
            env::var("PREFIX_ACCESS_TOKEN").unwrap_or(String::new()),
            Some(games::echo::echo_message_with_prefix),
        ),
    );
    FacebookApp::new(app_secret, webhook_verify_token, page_config)
}

/// Look up our server port number in PORT, for compatibility with Heroku.
fn get_server_port() -> u16 {
    let port_str = env::var("PORT").unwrap_or(String::new());
    port_str.parse().unwrap_or(8080)
}

fn router() -> Router {
    build_simple_router(|route| {
        let app = get_app();
        route
            .request(vec![Method::GET, Method::POST], "/webhook")
            .to_new_handler(app);
    })
}

fn main() {
    pretty_env_logger::init();

    let addr = SocketAddr::V4(SocketAddrV4::new(
        Ipv4Addr::new(0, 0, 0, 0),
        get_server_port(),
    ));

    gotham::start(addr, router());
}


#[cfg(test)]
mod tests {
    use super::*;
    use gotham::test::TestServer;

    #[test]
    fn get_request() {
        let test_server = TestServer::new(router()).unwrap();
        let response = test_server
            .client()
            .post(
                "http://localhost/webhook",
                r#"{
                    "entry": [{
                        "id": "446574812442849",
                        "messaging": [{
                            "message": {
                                "mid": "J1mDvYbKMXep5gd33zTzsQ-qajMGJdQsNQ8WpXxuvnig0LlcPT9F3VX6HDICD5Cx-93JE8aBFjxWPJ-RjmJoRg",
                                "seq": 20230,
                                "text": "yo"
                            },
                            "recipient": {
                                "id": "446574812442849"
                            },
                            "sender": {
                                "id": "1614085592031831"
                            },
                            "timestamp": 1541364211913
                        }],
                        "time": 1541367254070
                    }],
                    "object": "page"
                }"#,
                mime::TEXT_PLAIN)
            .perform()
            .unwrap();

        assert_eq!(response.status(), hyper::StatusCode::OK);
    }
}
