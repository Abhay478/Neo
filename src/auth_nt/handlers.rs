use super::*;

pub async fn dupe_acc(db: &Arc<Graph>, uu: &str) -> bool {
    let c = db
        .execute(Query::new(Database::read_query("dupe_acc")).param("unm", uu))
        .await;
    match c {
        Ok(mut rs) => {
            let row = rs.next().await.unwrap();
            dbg!(&row);
            row.unwrap().get::<i64>("count").unwrap() != 0
        }
        Err(e) => {
            println!("{}", e.to_string());
            panic!("")
        }
    }
}

pub fn hash(s: &str) -> String {
    use argon2::password_hash::{rand_core::OsRng, PasswordHasher, SaltString};
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(s.as_bytes(), &salt)
        .expect("Error while hashing password")
        .to_string()
}

pub async fn makeme(db: &Arc<Graph>, new: models::Creds) -> Result<models::Account, neo4rs::Error> {
    let mut c = db
        .execute(
            Query::new(Database::read_query("makeme"))
                .param("obj", uuid::Uuid::new_v4().to_string())
                .param("unm", new.username)
                .param("pswd", hash(&*new.password))
                .param("dnm", new.disp_name)
                .param("auth", new.auth.to_string()),
        )
        .await?;

    let rs = c.next().await?;
    dbg!(&rs);
    match rs {
        Some(cr) => {
            let x = &cr.get::<Path>("x").unwrap().nodes()[0];

            Ok(models::Account {
                obj: x.get("id").unwrap(),
                creds: models::Creds {
                    username: x.get("username").unwrap(),
                    password: x.get("password").unwrap(),
                    disp_name: x.get("disp_name").unwrap(),
                    auth: (&*x.get::<String>("auth").unwrap()).into(),
                },
            })
        }
        None => Err(neo4rs::Error::UnexpectedMessage("Ayo wut.".to_string())),
    }
}

pub async fn get_account(
    db: &Arc<Graph>,
    username: &str,
) -> Result<models::Account, neo4rs::Error> {
    let mut c = db
        .execute(Query::new(Database::read_query("get_account")).param("unm", username))
        .await?;

    let rs = c.next().await?;
    match rs {
        Some(row) => {
            let x = row.get::<Node>("a").unwrap();
            Ok(models::Account {
                obj: x.get("id").unwrap(),
                creds: models::Creds {
                    username: x.get("username").unwrap(),
                    password: x.get("password").unwrap(),
                    disp_name: x.get("disp_name").unwrap(),
                    auth: (&*x.get::<String>("auth").unwrap()).into(),
                },
            })
        }
        None => Err(neo4rs::Error::AuthenticationError("..".to_string())),
    }
    // todo!()
}

#[post("/auth/register")]
pub async fn register(
    mut body: web::Json<models::Creds>,
    data: web::Data<State>,
) -> impl Responder {
    let db = &data.graph;
    let exists = dupe_acc(db, &body.username).await;
    if exists {
        return HttpResponse::Conflict()
            .json(serde_json::json!({"status": "fail","message": "Doppleganger alert."}));
    }

    // Empty password means set to username.
    if body.password == "" {
        body.password = body.username.clone();
    }

    let res = makeme(db, body.0).await;

    match res {
        Ok(user) => {
            let token = get_token(&user.obj.to_string(), data, user.creds.auth);

            let cookie = Cookie::build("token", token.to_owned())
                .path("/")
                .max_age(AWD::new(60 * 60, 0))
                .http_only(true)
                .finish();
            HttpResponse::Ok().cookie(cookie).json(user)
        }
        Err(e) => HttpResponse::InternalServerError()
            .json(serde_json::json!({"status": "error","message": format!("{:?}", e)})),
    }
}

fn get_token(id: &str, data: web::Data<State>, auth: models::Authority) -> String {
    let now = Utc::now();
    let iat = now.timestamp() as usize;
    let exp = (now + Duration::minutes(60)).timestamp() as usize;
    let claims = models::TokenClaims {
        sub: id.to_string(),
        auth,
        exp,
        iat,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(data.env.jwt_secret.as_ref()),
    )
    .unwrap()
}

#[post("/auth/login")]
pub async fn login(body: web::Json<models::Creds>, data: web::Data<State>) -> impl Responder {
    let db = &data.graph;

    let query_result = get_account(db, &*body.username).await;

    match &query_result {
        Ok(user) => {
            let othertemp = body.clone().password;
            let temp = &user.clone().creds.password;
            let hash = PasswordHash::new(&temp).unwrap();
            let is_valid = Argon2::default()
                .verify_password(othertemp.as_bytes(), &hash)
                .is_ok();

            if !is_valid {
                // wrong password
                return HttpResponse::BadRequest()
                    .json(json!({"status": "fail", "message": "These are not the droids we are looking for."}));
            }
        }
        Err(_e) => {
            // user not found
            return HttpResponse::NotFound()
                .json(json!({"status": "fail", "message": "No record."}));
        }
    }

    let user = query_result.unwrap();

    let token = get_token(&user.obj, data, user.creds.auth);

    let cookie = Cookie::build("token", token.to_owned())
        .path("/")
        .max_age(AWD::new(60 * 60, 0))
        .http_only(true)
        .finish();

    HttpResponse::Ok()
        .cookie(cookie)
        .json(json!({"status": "success", "token": token}))
}

#[get("/auth/logout")]
pub async fn logout(_: models::Identity) -> impl Responder {
    let cookie = Cookie::build("token", "")
        .path("/")
        .max_age(AWD::new(-1, 0))
        .http_only(true)
        .finish();

    HttpResponse::Ok()
        .cookie(cookie)
        .json(json!({"status": "success"}))
}
