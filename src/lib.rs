use serde_json::json;
use worker::*;
use std::collections::HashMap;

mod utils;

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
        ("Mercury", 0.4),
        ("Venus", 0.7),
        ("Earth", 1.0),
        ("Mars", 1.5),
    ]);
    router
        .get("/", |_, _| Response::ok("Aircraft Performance calculators."))
        .post_async("/performance/:id", |mut req, ctx| async move {
            let res = req.bytes().await;
            let data = res.unwrap();
            let h: HashMap<String, i64> = serde_json::from_slice(&data).unwrap();
        
            let hsh = ctx.data;
            console_log!("{:?}", h);
            return Response::from_json(&HashMap::from(hsh));
        })
        .run(req, env)
        .await
}
