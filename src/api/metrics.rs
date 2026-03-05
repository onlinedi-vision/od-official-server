use crate::api::structures;
use actix_web;
use prometheus;
use prometheus::Encoder;

#[actix_web::get("/metrics")]
async fn metrics(data: actix_web::web::Data<structures::AppState>) -> impl actix_web::Responder {
    let metric_families = data.registry.gather();
    let encoder = prometheus::TextEncoder::new();
    
    let mut buffer = Vec::new();
    if let Err(e) = encoder.encode(&metric_families, &mut buffer) {
        eprintln!("Error encoding metrics: {e}");
        return actix_web::HttpResponse::InternalServerError().body("Error encoding metrics");
    }
    
    match String::from_utf8(buffer) {
        Ok(content) => actix_web::HttpResponse::Ok()
            .content_type("text/plain; version=0.0.4")
            .body(content),
        Err(e) => {
            eprintln!("Error converting metrics to string: {e}");
            actix_web::HttpResponse::InternalServerError().body("Error encoding metrics")
        }
    }
}
