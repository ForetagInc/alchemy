Data structures:

Alchemy API 					Database				Alchemy API
(to create the models) 		-> 		"services"		-> 		Get all collections

Example:
REST: /create_collection 
ex. services

HTTP Request (Schema)				Schema (JSON)			Create GraphQL AST
													-> Queries (add, get, count)
													-> Mutations (update, delete, deleteAll)
												
												Create a REST endpoints
													-> GET / POST / PATCH / DELETE

												GRPC / OData / ????

------

Querying 

GraphQL / REST / OData ?? -> AQL


get_allServices 
{
	id
	name
	price
	user {
		id
		first_name
	}
}

->

AQL?

------

Optimisations (Data structuries & Querying)

-> Full text Search (ArangoSearch)
-> Graphing


------

Structure / Data type (Object)

User / Service 
	-> Properties