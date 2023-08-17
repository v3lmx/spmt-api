use actix_cors::Cors;
use actix_session::{storage::CookieSessionStore, Session, SessionMiddleware};
use actix_web::{
    cookie::{Key, SameSite},
    web, App, HttpServer, Responder, Result,
};

use sea_orm::{Database, DatabaseConnection, DbErr};
use serde::{Deserialize, Serialize};

use spotify::{get_playlists, make_playlist_wip};
use uuid::Uuid;

#[derive(Serialize)]
struct BasicResponse {
    msg: String,
}

#[derive(Deserialize)]
pub struct ResponseCode {
    code: String,
}

// const SECS_IN_WEEK: i64 = 60 * 60 * 24 * 7;

#[derive(Deserialize, Debug)]
struct UserID(pub u128);

mod entities;
mod error;
mod session;
mod spotify;
mod sync;
use session::{callback, login};

/// Index route
async fn hello(session: Session) -> Result<impl Responder> {
    // session.insert("test", "test hello".to_string()).unwrap();
    match session.get::<Uuid>("user_uuid") {
        Ok(Some(uuid)) => {
            log::debug!("uuid: {:?}", uuid);
            let entries = session.entries(); //session.get::<String>("test").unwrap();
            return Ok(web::Json(BasicResponse {
                msg: format!("Hello user {:?} ! entries: {:?}", uuid, entries),
            }));
        }
        Ok(None) | Err(_) => Ok(web::Json(BasicResponse {
            msg: "Hello anon !".to_string(),
        })),
    }
}

/// WIP
async fn wip(session: Session, db: web::Data<DatabaseConnection>) -> Result<impl Responder> {
    sync::sync(session, db.get_ref()).await.unwrap();
    Ok("ok")
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

async fn connect_db() -> Result<DatabaseConnection, DbErr> {
    let db: DatabaseConnection =
        Database::connect("postgres://spmt:spmt-database-dev@localhost/spmt").await?;
    log::debug!("Ok connecting to db?");
    Ok(db)
}

#[actix_web::main]
async fn main() -> Result<(), anyhow::Error> {
    pretty_env_logger::init();

    let db = connect_db().await?;

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

        // let random = ChaCha8Rng::seed_from_u64(OsRng.next_u64());

        App::new()
            .wrap(cors)
            .wrap(session)
            // .app_data(web::Data::new(RandomGenerator {
            //     random: Arc::new(Mutex::new(random)),
            // }))
            .app_data(web::Data::new(db.clone()))
            .service(
                web::scope("/api")
                    .route("/", web::get().to(hello))
                    .route("/wip", web::get().to(wip))
                    .route("/test_cookies", web::get().to(test_cookies))
                    .route("/login", web::get().to(login))
                    .route("/playlists", web::get().to(get_playlists))
                    .route("/callback", web::get().to(callback))
                    .route("/make_playlist_wip", web::post().to(make_playlist_wip)),
            )
    })
    .bind(("0.0.0.0", 8000))?
    .run()
    .await?;
    Ok(())
}
