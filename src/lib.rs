use serde_json;
use worker::*;
use crate::performance::{performance, Criteria};
use std::collections::HashMap;

mod utils;
mod performance;

fn log_request(req: &Request) {
    console_log!(
        "{} - [{}], located at: {:?}, within: {}",
        Date::now().to_string(),
        req.path(),
        req.cf().coordinates().unwrap_or_default(),
        req.cf().region().unwrap_or("unknown region".into())
    );
}

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: worker::Context) -> Result<Response> {
    log_request(&req);

    // Optionally, get more helpful error messages written to the console in the case of a panic.
    utils::set_panic_hook();

    // Optionally, use the Router to handle matching endpoints, use ":name" placeholders, or "*name"
    // catch-alls to match on specific patterns. Alternatively, use `Router::with_data(D)` to
    // provide arbitrary data that will be accessible in each route via the `ctx.data()` method.
    let router = Router::with_data([
        ("arrow-iii", true)
    ]);
    router
        .get("/", |_, _| Response::ok("Aircraft Performance calculators."))
        .post_async("/performance/:id", |mut req, ctx| async move {
            let profiles = HashMap::from(ctx.data);
            if let Some(id) = ctx.param("id") {
                console_log!("{}, {:?}", id, profiles);
                let res = req.bytes().await;
                let data = res.unwrap();
                let h: Criteria = serde_json::from_slice(&data).unwrap();
            
                let p = performance(h.temp_c, h.pressure_alt, h.take_off_weight, h.headwind);
                return Response::from_json(&p);
            
            }
            Response::error("Not found", 404)
        })
        .run(req, env)
        .await
}