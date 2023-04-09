use actix_cors::Cors;
use actix_session::{
    config::PersistentSession, storage::CookieSessionStore, Session, SessionMiddleware,
};
use actix_web::{
    cookie::{time::Duration, Key, SameSite},
    http, web, App, HttpResponse, HttpServer, Responder, Result,
};
use log::{debug, error, info, warn};
use rand_chacha::ChaCha8Rng;
use rand_core::{OsRng, RngCore, SeedableRng};
use rspotify::{
    http::Form, prelude::*, scopes, AuthCodeSpotify, ClientResult, Config, Credentials, OAuth,
    Token,
};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, ConnectionTrait, Database, DatabaseConnection,
    DbErr, EntityTrait, QueryFilter, Set, Statement,
};
use serde::{Deserialize, Serialize};
use std::{
    fmt::Display,
    sync::{Arc, Mutex},
    thread::current,
};
use uuid::Uuid;

mod entities;
use entities::{prelude::*, *};

#[derive(Serialize)]
struct BasicResponse {
    msg: String,
}

#[derive(Deserialize)]
struct ResponseCode {
    code: String,
}

type Random = Arc<Mutex<ChaCha8Rng>>;

struct RandomGenerator {
    random: Random,
}

const SECS_IN_WEEK: i64 = 60 * 60 * 24 * 7;

#[derive(Deserialize, Debug)]
struct UserID(pub u128);

fn init_spotify() -> AuthCodeSpotify {
    let config = Config {
        token_cached: false,
        ..Default::default()
    };

    let oauth = OAuth {
        scopes: scopes!("user-read-currently-playing"),
        redirect_uri: "http://localhost:8000/api/callback".to_owned(),
        ..Default::default()
    };

    let creds = Credentials::new(
        "3b1de2acf7034579ae78b6e7ec675d49",
        "5362151430ee43a5b8273eaa88b381ca",
    );

    AuthCodeSpotify::with_config(creds, oauth, config)
}

/// Index route
async fn hello(session: Session) -> Result<impl Responder> {
    // session.insert("test", "test hello".to_string()).unwrap();
    match session.get::<Uuid>("user_uuid") {
        Ok(uuid) => {
            log::debug!("uuid: {:?}", uuid);
            let entries = session.entries(); //session.get::<String>("test").unwrap();
            return Ok(web::Json(BasicResponse {
                msg: format!("Hello user {:?} ! entries: {:?}", uuid, entries),
            }));
        }
        Err(_) => Ok(web::Json(BasicResponse {
            msg: "Hello anon !".to_string(),
        })),
    }
}

/// Index route
async fn test_cookies(session: Session) -> Result<impl Responder> {
    session
        .insert("test_cookie", "test cookie".to_string())
        .unwrap();
    Ok(web::Json(BasicResponse {
        msg: "Cookie set".to_string(),
    }))
}

/// Login route
/// This should be the entrypoint to initiate login
async fn login(session: Session) -> Result<impl Responder> {
    if let Some(user_id) = session.get::<UserID>("user_id")? {
        // TODO: Check that the token is valid
        log::debug!("User with id `{:?}` already logged in", user_id);

        return Ok(web::Json(BasicResponse {
            msg: String::from("User already logged in"),
        }));
    }

    let spotify = init_spotify();
    let auth_url = spotify.get_authorize_url(true).unwrap();
    info!("Auth URL: {}", auth_url);

    Ok(web::Json(BasicResponse { msg: auth_url }))
}

/// OAuth2 callback route
/// This route is called by spotify to complete the OAuth2 authorization process
async fn callback(
    response_code: web::Query<ResponseCode>,
    session: Session,
    db: web::Data<DatabaseConnection>,
) -> Result<impl Responder> {
    let mut spotify = init_spotify();

    // let res = spotify.request_token(&code).await;
    let token = spotify.request_token(&response_code.code).await;

    match token {
        Ok(_) => {
            info!("Request user token successful");
            let token_mutex = spotify.get_token();
            let mut token = token_mutex.lock().await.unwrap();
            let token: &Token = token.as_mut().expect("Token can't be empty as this point");

            let db = db.get_ref();

            // TODO: store token in db using session id
            info!("Token: {}", token.access_token);

            // TODO: maybe get user before taing token out of spotify we could avoid 2 new spotify objects
            let spotify = AuthCodeSpotify::from_token(token.clone());

            let current_user = spotify.me().await.unwrap();
            let user_spotify_id = current_user.id.to_string();
            log::info!("Current user : {}", &user_spotify_id);
            //current_user.id
            let user: Option<user::Model> = User::find()
                .filter(user::Column::SpotifyId.eq(&user_spotify_id))
                .one(db)
                .await
                .unwrap();
            let current_user_uuid = match user {
                Some(user) => {
                    log::debug!("Retreived User uuid : {}", user.uuid);
                    user.uuid
                    // let mut user: user::ActiveModel = user.into();
                    // user.token = Set(token.access_token.clone());
                    // let user: user::Model = user.update(db).await.unwrap();
                    // log::debug!("Token updated for user with id: `{}`", user.id);
                }
                None => {
                    let new_user_uuid = Uuid::new_v4();

                    let new_user = user::ActiveModel {
                        uuid: ActiveValue::Set(new_user_uuid),
                        name: ActiveValue::Set(
                            current_user
                                .display_name
                                .unwrap_or_else(|| String::from("No name")),
                        ),
                        spotify_id: ActiveValue::Set(Some(user_spotify_id)),
                    };
                    let res = User::insert(new_user).exec(db).await.unwrap();
                    log::debug!("Inserted user with id: `{}`", res.last_insert_id);

                    new_user_uuid
                }
            };
            session.renew();
            session.insert("user_uuid", current_user_uuid).unwrap();
            session.insert("test", "test".to_string()).unwrap();
            log::debug!("session entries : {:?}", session.entries());

            // return AppResponse::Json(BasicResponse { msg: String::from("logged in!") });
            //AppResponse::Redirect(Redirect::to("http://localhost:5173/"))
            Ok(web::Redirect::to("http://localhost:5173/"))
        }
        Err(err) => {
            error!("Failed to get user token {:?}", err);
            // return AppResponse::Json(BasicResponse { msg: String::from("not logged in") });
            // let mut context = HashMap::new();
            // context.insert("err_msg", "Failed to get token!");
            // AppResponse::Template(Template::render("error", context))
            Ok(web::Redirect::to("http://localhost:5173/error"))
        }
    }
    // return Json(BasicResponse { msg: String::from("not logged in") });
}

async fn connect_db() -> Result<DatabaseConnection, DbErr> {
    // TODO: connect to "spmt" instead of "public"
    let db: DatabaseConnection =
        Database::connect("postgres://spmt:spmt-database-dev@localhost/spmt").await?;
    log::debug!("Ok connecting to db?");
    Ok(db)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    pretty_env_logger::init();

    let db = connect_db().await.expect("Error creating databse");

    // Cookies key needs to be outside of app context
    // or it keeps regenerating making it impossible to
    // berify content, and session works for a few seconds only
    let cookies_key = Key::generate();

    HttpServer::new(move || {
        // TODOPROD: Permissive for local development
        let cors = Cors::permissive();

        // TODOPROD: Cookies unsecure for local development
        // let cookies_key = Key::generate();
        let session =
            SessionMiddleware::builder(CookieSessionStore::default(), cookies_key.clone())
                .cookie_name("spmtb_session_id".to_string())
                .cookie_same_site(SameSite::Lax)
                .cookie_secure(false)
                .cookie_http_only(false)
                // Not needed probably
                // .session_lifecycle(
                //     PersistentSession::default().session_ttl(Duration::seconds(SECS_IN_WEEK)),
                // )
                .build();
        // let session = SessionMiddleware::new(CookieSessionStore::default(), cookies_key.clone());

        let random = ChaCha8Rng::seed_from_u64(OsRng.next_u64());

        App::new()
            .wrap(cors)
            .wrap(session)
            .app_data(web::Data::new(RandomGenerator {
                random: Arc::new(Mutex::new(random)),
            }))
            .app_data(web::Data::new(db.clone()))
            .service(
                web::scope("/api")
                    .route("/", web::get().to(hello))
                    .route("/test_cookies", web::get().to(test_cookies))
                    .route("/login", web::get().to(login))
                    .route("/callback", web::get().to(callback)),
            )
    })
    .bind(("0.0.0.0", 8000))?
    .run()
    .await
}
