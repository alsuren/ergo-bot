use futures::future;
use gotham::handler::HandlerFuture;
use gotham::http::response::create_response;
use gotham::state::{FromState, State};
use hyper::{StatusCode, Uri};
use mime::TEXT_PLAIN;

use crate::facebook_app;

pub fn handle_verification(state: State, app: facebook_app::FacebookApp) -> Box<HandlerFuture> {
    let uri = Uri::borrow_from(&state).clone();

    let query: &str = uri.query().unwrap_or(&"");
    let hub_challenge = app.verify_webhook_query(&query);

    match hub_challenge {
        Some(challenge) => {
            println!("returning success");
            let res = create_response(
                &state,
                StatusCode::OK,
                Some((challenge.as_bytes().to_vec(), TEXT_PLAIN)),
            );
            Box::new(future::ok((state, res)))
        }
        None => {
            let msg = format!(
                "Incorrect webhook_verify_token or No hub.challenge in {}",
                query
            );
            let res = create_response(
                &state,
                StatusCode::BAD_REQUEST,
                Some((msg.as_bytes().to_vec(), TEXT_PLAIN)),
            );
            Box::new(future::ok((state, res)))
        }
    }
}
