use std::pin::Pin;

use async_graphql::{Context, FieldError, FieldResult, Object, Subscription};

use crate::{auth_nt::models::*, neo_nt::handlers::Database, State};

// Big three

///Query root
pub struct Query;
/// Mutation root
pub struct Mutation;
/// Subscription Root
pub struct Subscription;

use crate::neo_nt::models::*;
// mod models {
//     
// }

#[Object]
impl Query {
    async fn test(&self) -> FieldResult<i32> {
        FieldResult::Ok(1)
    }

    /// Returns your topics
    pub async fn topics(&self, ctx: &Context<'_>) -> FieldResult<TopicList> {
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
            return Ok(TopicList { fd: r.clone() });
        } else {
            panic!("Database not in context.");
        }
    }

    /// Returns the topic's contents.
    pub async fn book(&self, ctx: &Context<'_>, topic: String) -> FieldResult<Book> {
        if let Some(me) = ctx.data_opt::<Identity>() {
            if me.auth != Authority::Subscriber {
                return Err(FieldError::new("Unauthorized"));
            }
        } else {
            panic!("Identity not in Context.");
        }

        let me = &ctx.data_unchecked::<Identity>().user_id;
        if let Some(state) = ctx.data_opt::<State>() {
            let r = Database::get_book(&state.graph, me.clone(), topic.clone())
                .await
                .unwrap();
            return Ok(Book { fd: r.clone() });
        } else {
            panic!("Database not in context.");
        }

        // todo!()
    }

    /// Prefix-search for topics.
    pub async fn topics_like(&self, ctx: &Context<'_>, prefix: String) -> FieldResult<TopicList> {
        if let Some(me) = ctx.data_opt::<Identity>() {
            if me.auth != Authority::Subscriber {
                return Err(FieldError::new("Unauthorized"));
            }
        } else {
            panic!("Identity not in Context.");
        }

        // let me = &ctx.data_unchecked::<Identity>().user_id;
        if let Some(state) = ctx.data_opt::<State>() {
            let r = Database::search_topic_by_name(&state.graph, prefix.clone())
                .await
                .unwrap();
            return Ok(TopicList { fd: r.clone() });
        } else {
            panic!("Database not in context.");
        }
        // todo!()
    }

    // pub async fn services(&self, ctx)
}

#[Object]
impl Mutation {
    async fn test(&self) -> FieldResult<i32> {
        FieldResult::Ok(1)
    }

    async fn subscribe(&self, ctx: &Context<'_>, topic: String) -> FieldResult<FollowRequest> {
        if let Some(me) = ctx.data_opt::<Identity>() {
            if me.auth != Authority::Subscriber {
                return Err(FieldError::new("Unauthorized"));
            }
        } else {
            panic!("Identity not in Context.");
        }

        let me = &ctx.data_unchecked::<Identity>().user_id;
        if let Some(state) = ctx.data_opt::<State>() {
            let r = Database::subscribe_to(&state.graph, me.clone(), topic)
                .await?;
            return Ok(r);
        } else {
            panic!("Database not in context.");
        }

        // todo!()
    }

    async fn unsubscribe(&self, ctx: &Context<'_>, topic: String) -> FieldResult<bool> {
        if let Some(me) = ctx.data_opt::<Identity>() {
            if me.auth != Authority::Subscriber {
                return Err(FieldError::new("Unauthorized"));
            }
        } else {
            panic!("Identity not in Context.");
        }

        let me = &ctx.data_unchecked::<Identity>().user_id;
        if let Some(state) = ctx.data_opt::<State>() {
            let r = Database::unsubscribe(&state.graph, me.clone(), topic).await?;
            return Ok(r);
        } else {
            panic!("Database not in context.");
        }

        // todo!()
    }

    async fn start_service(
        &self,
        ctx: &Context<'_>,
        topic: String,
        typ: String,
    ) -> FieldResult<Service> {
        if let Some(me) = ctx.data_opt::<Identity>() {
            if me.auth != Authority::ServiceProvider {
                return Err(FieldError::new("Unauthorized"));
            }
        } else {
            panic!("Identity not in Context.");
        }

        let me = &ctx.data_unchecked::<Identity>().user_id;
        if let Some(state) = ctx.data_opt::<State>() {
            let r =
                Database::new_service(&state.graph, me.clone(), typ.as_str().into(), topic).await?;
            return Ok(r);
        } else {
            panic!("Database not in context.");
        }
        // todo!()
    }

    /// `Service` struct must be stored by the ServiceProvider. `Page` is the actual data.
    async fn publish(&self, ctx: &Context<'_>, serv: Service, fr: Page) -> FieldResult<Frame> {
        if let Some(me) = ctx.data_opt::<Identity>() {
            if me.auth != Authority::ServiceProvider && me.auth != Authority::Admin {
                return Err(FieldError::new("Unauthorized"));
            }
        } else {
            panic!("Identity not in Context.");
        }

        // let me = &ctx.data_unchecked::<Identity>().user_id;
        if let Some(state) = ctx.data_opt::<State>() {
            let r = Database::publish(&state.graph, serv, fr).await?;
            return Ok(r);
        } else {
            panic!("Database not in context.");
        }
    }

    pub async fn create_topic(&self, ctx: &Context<'_>, name: String, des: String) -> FieldResult<Topic> {
        if let Some(me) = ctx.data_opt::<Identity>() {
            if me.auth != Authority::ServiceProvider && me.auth != Authority::Admin {
                return Err(FieldError::new("Unauthorized"));
            }
        } else {
            panic!("Identity not in Context.");
        }

        let me = &ctx.data_unchecked::<Identity>().user_id;
        if let Some(state) = ctx.data_opt::<State>() {
            let r = Database::new_topic(&state.graph, me.clone(), name, des).await?;
            return Ok(r);
        } else {
            panic!("Database not in context.");
        }
    }

    pub async fn terminate_service(&self, ctx: &Context<'_>, sid: String) -> FieldResult<bool> {
        if let Some(me) = ctx.data_opt::<Identity>() {
            if me.auth != Authority::ServiceProvider && me.auth != Authority::Admin {
                return Err(FieldError::new("Unauthorized"));
            }
        } else {
            panic!("Identity not in Context.");
        }

        let me = &ctx.data_unchecked::<Identity>().user_id;
        if let Some(state) = ctx.data_opt::<State>() {
            Database::kill_service(&state.graph, sid, me.clone()).await?;
            return Ok(true);
        } else {
            panic!("Database not in context.");
        }
        // todo!()
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
