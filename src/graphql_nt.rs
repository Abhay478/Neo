use std::pin::Pin;

use async_graphql::{FieldError, FieldResult, Object, Subscription};

// use crate::{neo_nt::models::Message, State};
// impl async_graphql::Context for State {}

pub struct Query;

#[Object]
impl Query {
    async fn test(&self) -> FieldResult<i32> {
        FieldResult::Ok(1)
    }
}
pub struct Mutation;
#[Object]
impl Mutation {
    async fn test(&self) -> FieldResult<i32> {
        FieldResult::Ok(1)
    }
}

// type MessageStream = Pin<Box<dyn futures::Stream<Item = Result<Message, FieldError>> + Send>>;

type IntStream = Pin<Box<dyn futures::Stream<Item = Result<i32, FieldError>> + Send>>;

pub struct Subscription;
#[Subscription]
impl Subscription {
    async fn test(&self) -> IntStream {
        Box::pin(async_stream::stream! {loop {
            yield Ok(1)
        }})
        // todo!()
    }
}
