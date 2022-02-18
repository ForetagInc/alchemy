use actix_web::{
	Error as ActixError,
	HttpRequest as ActixRequest,
	HttpResponse as ActixResponse,
};

pub async fn rest_routes(
	_req: ActixRequest
) -> Result<ActixResponse, ActixError> {
	Ok(ActixResponse::Ok().body("Hello World!"))
}