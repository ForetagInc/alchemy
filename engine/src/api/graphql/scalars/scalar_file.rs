use juniper::{ Value, ParserScalarResult, ParserScalarValue };

#[juniper::graphql_scalar(description = "File")]
impl <S> GraphQLScalar for File
where
	S: ScalarValue
{
		
}