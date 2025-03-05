mod structures;

#[actix_web::get("/api/test")]
pub async fn get_test(
    req: actix_web::web::Query<structures::TestParamsStruct>
) -> impl actix_web::Responder {  
    actix_web::HttpResponse::Ok().body(
        format!("{:?}{:?}", req.param1, req.param2)
    )

}
