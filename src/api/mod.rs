/* !TODO:
 *  -- check out passing secrets with GET requests (to replace weird POST request implementation)
 * */
#[macro_use]
pub mod prelude;

pub mod channel;
pub mod friends;
pub mod invites;
pub mod message;
pub mod roles;
pub mod server;
pub mod spell_caster;
pub mod statics;
pub mod structures;
pub mod user;
pub mod metrics;

#[actix_web::get("/version")]
/// Returns the current API version string. This endpoint takes no request body.
///
/// ### Request JSON
/// _None. This is a `GET` request with no body._
///
/// ### Example (reqwest)
/// ```rust
/// let client = reqwest::Client::new();
/// let res = client
///     .get("http://localhost:1313/version")
///     .send()
///     .await?;
/// ```
pub async fn get_api_version() -> impl actix_web::Responder {
    actix_web::HttpResponse::Ok().body("v0.0.9".to_string())
}
