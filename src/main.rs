extern crate juniper;
#[macro_use] extern crate juniper_codegen;

use actix_cors::Cors;
use actix_web::{
	http::header,
	middleware,
	web::{ 
		self, 
		Data,
		Payload as ActixPayload
	},
	App,
	Error as ActixError,
	HttpRequest as ActixRequest,
	HttpResponse as ActixResponse,
	HttpServer, 
};

use juniper_actix::{ graphql_handler, playground_handler };

pub mod lib;
use lib::graphql::{ Context, Schema, schema };

async fn playground_route() -> Result<ActixResponse, ActixError>
{
	playground_handler(
		"/graphql", 
		Some("/graphql_subscriptions")
	).await
}

async fn graphql_route(
	req: ActixRequest,
	payload: ActixPayload,
	schema: Data<Schema>,
) -> Result<ActixResponse, ActixError> {
	let context = Context::new().await;
	graphql_handler(&schema, &context, req, payload).await
}

#[actix_web::main]
async fn main() -> std::io::Result<()>
{
	println!("Starting Alchemy on port 8080");

	// Actix server
	HttpServer::new(|| {
		App::new()
			.app_data(Data::new(schema()))
			.wrap(
				Cors::default()
					.allow_any_origin()
					.allowed_methods(vec!["POST", "GET"])
					.allowed_headers(vec![header::AUTHORIZATION, header::ACCEPT])
					.allowed_header(header::CONTENT_TYPE)
					.supports_credentials()
					.max_age(3600)
			)
			.wrap(middleware::Compress::default())
			.wrap(middleware::Logger::default())
			.service(
				web::resource("/graphql")
					.route(web::post().to(graphql_route))
					.route(web::get().to(graphql_route))
			)
			.service(web::resource("/playground").route(web::get().to(playground_route)))
	})
	.bind(("0.0.0.0", 8080))?
	.run()
	.await
}
