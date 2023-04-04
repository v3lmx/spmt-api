use log::{info, warn, error, debug};
use rocket::http::{CookieJar, Cookie, Header, SameSite};
use rocket::response::Redirect;
use rocket::{Request, Response};
use rocket::fairing::{Fairing, Info, Kind};
use rspotify::Token;
use rspotify::{prelude::*, AuthCodeSpotify, Config, OAuth, scopes, Credentials, ClientResult, http::Form};
// use rspotify_macros::scopes;
// use rspotify::clients::OAuthClient;
use uuid::Uuid;
use rocket::serde::{Serialize, json::Json};
use std::collections::HashSet;


#[derive(Serialize, Responder)]
#[serde(crate = "rocket::serde")]
struct BasicResponse {
    msg: String
}

#[derive(Debug, Responder)]
pub enum AppResponse<T> {
    Redirect(Redirect),
    Json(T),
}


#[macro_use] extern crate rocket;

#[async_trait]
trait DatabaseToken {
    /// Obtains a user access token given a code, as part of the OAuth
    /// authentication. The access token will be saved internally.
    async fn request_token_explicit(&mut self, code: &str) -> ClientResult<()>;
}

// #[async_trait]
// impl DatabaseToken for AuthCodeSpotify {
//     /// Obtains a user access token given a code, as part of the OAuth
//     /// authentication. The access token will be saved internally.
//     async fn request_token_explicit(&mut self, code: &str) -> ClientResult<Token> {
//         log::info!("Requesting Auth Code token");

//         let scopes = join_scopes(&self.oauth.scopes);
//         // let scopes = scopes!("playlist-read-private", "playlist-read-collaborative");

//         let mut data = Form::new();
//         data.insert(params::GRANT_TYPE, params::GRANT_TYPE_AUTH_CODE);
//         data.insert(params::REDIRECT_URI, &self.oauth.redirect_uri);
//         data.insert(params::CODE, code);
//         data.insert(params::SCOPE, &scopes);
//         data.insert(params::STATE, &self.oauth.state);

//         let headers = self
//             .creds
//             .auth_headers()
//             .expect("No client secret set in the credentials.");

//         let token = self.fetch_access_token(&data, Some(&headers)).await?;
//         *self.token.lock().await.unwrap() = Some(token);

//         // self.write_token_cache().await
//     }
// }


fn init_spotify(jar: &CookieJar<'_>) -> AuthCodeSpotify {
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


#[get("/")]
fn index(jar: &CookieJar<'_>) -> Json<BasicResponse> {
    jar.get("message").map(|crumb| format!("Message: {}", crumb.value()));
    jar.add(Cookie::build("test", "1")
        .domain("http://localhost")
        .secure(true)
        .same_site(SameSite::None)
        .finish()
    );
    // Some(String::from("heelo!!"))
    Json(BasicResponse { msg: String::from("heelo!!") })
}

#[get("/login")]
fn login(jar: &CookieJar<'_>) -> Json<BasicResponse> {
    // TODO check if cookie exists in db
    let authenticated = jar.get("uuid").is_some();
    if !authenticated {
        info!("Not authenticated, creating cookie");
        jar.add(Cookie::new("uuid", Uuid::new_v4().to_string()));

        let spotify = init_spotify(&jar);
        let auth_url = spotify.get_authorize_url(true).unwrap();
        // TODO add uri! macro
        info!("Auth URL: {}", auth_url);
        // return Some(auth_url)
        return Json(BasicResponse { msg: String::from(auth_url) })
    }
    return Json(BasicResponse { msg: String::from("nok") })
}

#[get("/callback?<code>")]
async fn callback(jar: &CookieJar<'_>, code: String) -> AppResponse<BasicResponse> {
    let mut spotify = init_spotify(jar);

    // let res = spotify.request_token(&code).await;
    let token = spotify.request_token_explicit(&code).await;

    match token {
        Ok(_) => {
            info!("Request user token successful");
            // return AppResponse::Json(BasicResponse { msg: String::from("logged in!") });
            AppResponse::Redirect(Redirect::to("http://localhost:5173/"))
        }
        Err(err) => {
            error!("Failed to get user token {:?}", err);
            // return AppResponse::Json(BasicResponse { msg: String::from("not logged in") });
            // let mut context = HashMap::new();
            // context.insert("err_msg", "Failed to get token!");
            // AppResponse::Template(Template::render("error", context))
            AppResponse::Redirect(Redirect::to("http://localhost:5173/error"))
        }
    }
    // return Json(BasicResponse { msg: String::from("not logged in") });
}

#[get("/me")]
fn me(jar: &CookieJar<'_>) -> Json<BasicResponse> {
    let authenticated = jar.get("uuid").is_some();
    if !authenticated {
        return Json(BasicResponse { msg: String::from("not logged in") });
    }
    return Json(BasicResponse { msg: String::from("not logged in") });
}

pub struct CORS;

#[rocket::async_trait]
impl Fairing for CORS {
    fn info(&self) -> Info {
        Info {
            name: "Add CORS headers to responses",
            kind: Kind::Response
        }
    }

    async fn on_response<'r>(&self, _request: &'r Request<'_>, response: &mut Response<'r>) {
        response.set_header(Header::new("Access-Control-Allow-Origin", "*"));
        response.set_header(Header::new("Access-Control-Allow-Methods", "POST, GET, PATCH, OPTIONS"));
        response.set_header(Header::new("Access-Control-Allow-Headers", "*"));
        response.set_header(Header::new("Access-Control-Allow-Credentials", "true"));
    }
}

#[launch]
fn rocket() -> _ {

    rocket::build()
        .mount("/api", routes![index, login, me, callback])
        .attach(CORS)
}
