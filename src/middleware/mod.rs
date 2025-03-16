use actix_web::{
    body::BoxBody,
    dev::{ServiceRequest, ServiceResponse},
    http::Method,
    middleware::Next,
};

pub async fn strip_body(
    req: ServiceRequest,
    next: Next<BoxBody>,
) -> Result<ServiceResponse<BoxBody>, actix_web::Error> {
    let method = req.method().clone();
    let res = next.call(req).await;

    match res {
        Err(e) => Err(e),
        Ok(res) if method == Method::HEAD => Ok(res.map_body(|_, _| ()).map_into_boxed_body()),
        Ok(res) => Ok(res.map_into_boxed_body()),
    }
}
