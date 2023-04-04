use actix_cors::Cors;
use actix_session::{storage::CookieSessionStore, Session, SessionMiddleware};
use actix_web::{cookie::Key, http, web, App, HttpResponse, HttpServer, Responder, Result};
use log::{debug, error, info, warn};
use rand_chacha::ChaCha8Rng;
use rand_core::{OsRng, RngCore, SeedableRng};
use rspotify::{
    http::Form, prelude::*, scopes, AuthCodeSpotify, ClientResult, Config, Credentials, OAuth,
    Token,
};
use sea_orm::{DatabaseConnection, Database};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

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

#[derive(Deserialize)]
struct SessionID(pub u128);

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
async fn hello(session: Session) -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

/// Login route
/// This should be the entrypoint to initiate login
async fn login(session: Session) -> Result<impl Responder> {
    let _session_id = session.get::<SessionID>("id")?;

    if _session_id.is_some() {
        // TODO: Check that the token is valid
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

            // TODO: store token in db using session id
            info!("Token: {}", token.access_token);
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

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    pretty_env_logger::init();

    let db: DatabaseConnection = Database::connect("postgres://spmt@localhost/spmt").await.unwrap();

    HttpServer::new(|| {
        // TODOPROD: Permissive for local development
        let cors = Cors::permissive();

        // TODOPROD: Cookies unsecure for local development
        let cookies_key = Key::generate();
        let session = SessionMiddleware::new(CookieSessionStore::default(), cookies_key.clone());

        let random = ChaCha8Rng::seed_from_u64(OsRng.next_u64());
        App::new()
            .wrap(cors)
            .wrap(session)
            .app_data(web::Data::new(RandomGenerator {
                random: Arc::new(Mutex::new(random)),
            }))
            .service(
                web::scope("/api")
                    .route("/", web::get().to(hello))
                    .route("/login", web::get().to(login))
                    .route("/callback", web::get().to(callback)),
            )
    })
    .bind(("0.0.0.0", 8000))?
    .run()
    .await
}
