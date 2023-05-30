use super::*;

/// Application specific
pub struct Database {}

/// To write a new application, just comment this impl out.
type NeoResult<T> = Result<T, neo4rs::Error>;
impl Database {
    /// I do NOT want to write cypher queries in strings.
    /// They're all in .cypher files, with proper formatting and syntax highlighting.
    pub fn read_query(name: &str) -> String {
        std::fs::read_to_string(format!("src/cypher/{name}.cypher")).unwrap()
    }

    pub async fn constraints(db: &Arc<Graph>) {
        db.run(Query::new(Self::read_query("simple_graph_for_subs")))
            .await
            .unwrap();

        db.run(Query::new(Self::read_query("unique_topics")))
            .await
            .unwrap();

        db.run(Query::new(Self::read_query("simple_graph_for_services")))
            .await
            .unwrap();
    }

    /// Creates a new service.
    /// Called only by ServiceProvider. Redundant arg performs dev-time check.
    pub async fn new_service(
        db: &Arc<Graph>,
        // me: Identity,
        typ: models::ServiceType,
        // topic name
        topic: String,
    ) -> NeoResult<self::models::Service> {
        // --
        // if me.auth != Authority::Admin {
        //     return Err(neo4rs::Error::AuthenticationError(
        //         "Remove this".to_string(),
        //     ));
        // }
        // --
        let mut rs = db
            .execute(
                Query::new(Self::read_query("new_service"))
                    .param("sid", uuid::Uuid::new_v4().to_string())
                    .param("tname", topic.clone())
                    .param("typ", typ.to_string()),
            )
            .await?;

        let r = rs.next().await?.unwrap();
        let x = r.get::<Path>("x").unwrap();
        Ok(self::models::Service {
            id: x.nodes()[0].get("id").unwrap(),
            typ: (&*x.rels()[0].get::<String>("type").unwrap()).into(),
            topic: x.nodes()[0].get("topic").unwrap(),
        })
    }

    /// Create a new subscription to the given topic
    pub async fn subscribe_to(
        db: &Arc<Graph>,
        me: String,
        topic: String,
    ) -> NeoResult<models::FollowRequest> {
        let mut rs = db
            .execute(
                Query::new(Self::read_query("subscribe_to"))
                    .param("me", me.clone())
                    .param("tname", topic.clone())
                    .param("id", uuid::Uuid::new_v4().to_string())
                    .param("from", chrono::offset::Utc::now().to_string()),
            )
            .await?;
        let row = rs.next().await?.unwrap();
        let out = row.get::<Path>("out").unwrap();
        Ok(models::FollowRequest {
            id: out.rels()[0].get("sub_id").unwrap(),
            topic: out.rels()[0].get("topic").unwrap(),
            user: me,
        })
    }

    /// Also admin-only.
    pub async fn new_topic(
        db: &Arc<Graph>,
        me: Identity,
        topic: String,
    ) -> NeoResult<self::models::Topic> {
        // --
        if me.auth != Authority::Admin {
            return Err(neo4rs::Error::AuthenticationError(
                "Remove this".to_string(),
            ));
        }
        // --
        let mut rs = db
            .execute(
                Query::new(Self::read_query("new_topic"))
                    .param("id", uuid::Uuid::new_v4().to_string())
                    .param("name", topic),
            )
            .await?;

        let row = rs.next().await?.unwrap();
        let x = row.get::<Path>("t").unwrap();
        Ok(Extractor::topic(&x.nodes()[0]))
    }

    /// Deletes subscription
    pub async fn unsubscribe(db: &Arc<Graph>, me: String, topic: String) -> NeoResult<bool> {
        let mut rs = db.execute(
            Query::new(Self::read_query("unsubscribe"))
                .param("me", me)
                .param("tname", topic),
        )
        .await?;
        Ok(!rs.next().await?.unwrap().get::<i64>("c").unwrap() == 0)
    }

    /// Admin-only, again.
    pub async fn kill_service(db: &Arc<Graph>, id: String) -> NeoResult<()> {
        db.run(Query::new(Self::read_query("kill_service")).param("id", id))
            .await?;
        Ok(())
    }

    /// Admin-only, again.
    pub async fn retire_topic(db: &Arc<Graph>, topic: String) -> NeoResult<()> {
        db.run(Query::new(Self::read_query("retire_topic")).param("id", topic))
            .await?;
        Ok(())
    }

    /// Returns all the topics you're subscribed to. Subscription types be?
    pub async fn get_topics(db: &Arc<Graph>, me: String) -> NeoResult<Vec<models::Topic>> {
        let mut rs = db
            .execute(Query::new(Self::read_query("get_topics")).param("me", me))
            .await?;

        // Wanted to do this part with maps and iterators, but apparrntly RowStream does not implement Iterator or IntoIterator.
        let mut out = vec![];
        while let Ok(Some(row)) = rs.next().await {
            let node = row.get::<Node>("t").unwrap();
            out.push(Extractor::topic(&node));
        }

        Ok(out)
    }

    /// Admin
    pub async fn get_subscribers(db: &Arc<Graph>, topic: String) -> NeoResult<Vec<User>> {
        let mut rs = db
            .execute(Query::new(Self::read_query("get_subscribers")).param("id", topic))
            .await?;
        let mut out = vec![];
        while let Ok(Some(row)) = rs.next().await {
            let node = row.get::<Node>("t").unwrap();
            out.push(User {
                username: node.get("username").unwrap(),
                disp_name: node.get("disp_name").unwrap(),
            });
        }

        Ok(out)

        // todo!()
    }

    /// New Authority called ServiceProvider, let those login too, and call this function to post to a topic.
    pub async fn publish(
        db: &Arc<Graph>,
        serv: models::Service,
        page: models::Frame,
    ) -> NeoResult<models::Page> {
        let mut rs = db
            .execute(
                Query::new(Self::read_query("publish"))
                    .param("sid", serv.id)
                    .param("title", page.title)
                    .param("pid", uuid::Uuid::new_v4().to_string())
                    .param("body", page.body)
                    .param("time", chrono::offset::Utc::now().to_string())
                    .param("tname", serv.topic.clone()),
            )
            .await?;

        let row = rs.next().await?.unwrap();
        let x = row.get::<Path>("x").unwrap();
        let entry = &x.nodes()[0];
        Ok(Extractor::page(entry))
        // todo!()
    }

    /// Might set a limit on this, either hard or timestamp-based - going to be in the cypher, so rust won't change.
    /// Returns everything ever published to a topic (after checking if you've subscribed).
    pub async fn get_book(
        db: &Arc<Graph>,
        me: String,
        topic: String,
    ) -> NeoResult<Vec<models::Page>> {
        let mut rs = db
            .execute(
                Query::new(Self::read_query("is_subbed"))
                    .param("tname", topic.clone())
                    .param("me", me.clone()),
            )
            .await?;
        let row = rs.next().await?.unwrap().get::<i64>("c").unwrap();
        if row == 0 {
            return Err(neo4rs::Error::AuthenticationError(
                "Not subscribed to topic.".to_string(),
            ));
        }
        let mut rs = db
            .execute(Query::new(Self::read_query("get_book")).param("tname", topic))
            .await?;
        let mut out = vec![];
        while let Ok(Some(entry)) = rs.next().await {
            out.push(Extractor::page(&entry.get::<Node>("out").unwrap()))
        }
        Ok(out)
        // todo!()
    }

    /// Returns a vec of topics whose names start with `prefix`.
    pub async fn search_topic_by_name(
        db: &Arc<Graph>,
        prefix: String,
    ) -> NeoResult<Vec<models::Topic>> {
        let mut rs = db
            .execute(Query::new(Self::read_query("search_topic_by_name")).param("prefix", prefix))
            .await?;
        let mut out = vec![];
        while let Ok(Some(x)) = rs.next().await {
            out.push(Extractor::topic(&x.get("t").unwrap()))
        }
        todo!()
    }
}

struct Extractor {}

impl Extractor {
    fn topic(x: &Node) -> models::Topic {
        models::Topic {
            id: x.get("id").unwrap(),
            info: Self::info(x),
            name: x.get("name").unwrap(),
        }
    }

    /// .
    fn info(x: &Node) -> models::TopicInfo {
        models::TopicInfo {
            pages: x.get("pages").unwrap(),
            subs: x.get("subs").unwrap(),
            time: x.get("time").unwrap(),
            desc: x.get("desc").unwrap(),
        }
    }

    fn page(x: &Node) -> models::Page {
        models::Page {
            id: x.get("id").unwrap(),
            body: Self::frame(x),
            time: x.get("time").unwrap(),
            by: x.get("by").unwrap()
        }
    }

    fn frame(x: &Node) -> models::Frame {
        models::Frame {
            title: x.get("title").unwrap(),
            body: x.get("body").unwrap(),
        }
    }
}
