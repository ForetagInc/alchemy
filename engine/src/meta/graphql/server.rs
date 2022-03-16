use super::{Context, Schema};

use actix_web::{
	web::{Data, Payload as ActixPayload},
	Error as ActixError, HttpRequest as ActixRequest, HttpResponse as ActixResponse,
};

use juniper_actix::{graphql_handler, playground_handler};

pub async fn graphql_meta_route(
	req: ActixRequest,
	payload: ActixPayload,
	schema: Data<Schema>,
) -> Result<ActixResponse, ActixError> {
	let context = Context::new().await;
	graphql_handler(&schema, &context, req, payload).await
}

pub async fn playground_meta_route() -> Result<ActixResponse, ActixError> {
	playground_handler("/meta/graphql", Some("/meta/graphql_subscriptions")).await
}
