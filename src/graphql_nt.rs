use std::pin::Pin;

use async_graphql::{Context, FieldError, FieldResult, Object, Subscription};

use crate::{
    auth_nt::{Authority, Identity},
    neo_nt::Database,
    State,
};

// Big three

///Query root
pub struct Query;
/// Mutation root
pub struct Mutation;
/// Subscription Root
pub struct Subscription;

mod models {
    use async_graphql::SimpleObject;

    use crate::neo_nt::models::Topic;

    #[derive(SimpleObject, Clone)]
    pub struct SubscriptionList {
        pub fd: Vec<Issue>,
    }

    #[derive(SimpleObject, Clone)]
    pub struct Issue {
        pub id: String,
        pub name: String,
    }

    impl From<&Topic> for Issue {
        fn from(value: &Topic) -> Self {
            Self {
                id: value.id.clone(),
                name: value.name.clone(),
            }
        }
    }
}

#[Object]
impl Query {
    async fn test(&self) -> FieldResult<i32> {
        FieldResult::Ok(1)
    }

    async fn get_subscriptions(&self, ctx: &Context<'_>) -> FieldResult<models::SubscriptionList> {
        if let Some(me) = ctx.data_opt::<Identity>() {
            if me.auth != Authority::Subscriber {
                return Err(FieldError::new("Unauthorized"));
            }
        } else {
            panic!("Identity not in Context.");
        }

        let me = &ctx.data_unchecked::<Identity>().user_id;

        if let Some(state) = ctx.data_opt::<State>() {
            let r = Database::get_topics(&state.graph, me.clone())
                .await
                .unwrap();
            return Ok(models::SubscriptionList {
                fd: r.iter().map(|u| Into::<models::Issue>::into(u)).collect(),
            });
        } else {
            panic!("Database not in context.");
        }
    }
}

#[Object]
impl Mutation {
    async fn test(&self) -> FieldResult<i32> {
        FieldResult::Ok(1)
    }
}

/// This is a test type.
type IntStream = Pin<Box<dyn futures::Stream<Item = Result<i32, FieldError>> + Send>>;

#[Subscription]
impl Subscription {
    async fn test(&self) -> IntStream {
        Box::pin(async_stream::stream! {loop {
            yield Ok(1)
        }})
        // todo!()
    }
}
