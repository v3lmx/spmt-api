use actix_session::Session;
use actix_web::{web, Responder, Result};
use rspotify::prelude::{BaseClient, OAuthClient};
use rspotify::{AuthCodeSpotify, Token};
use sea_orm::{ActiveValue, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use uuid::Uuid;

use crate::entities::{prelude::*, user};
use crate::spotify::init_spotify;
use crate::{BasicResponse, ResponseCode};

/// Login route
/// This should be the entrypoint to initiate login
pub async fn login(session: Session) -> Result<impl Responder> {
    if let Some(user_uuid) = session.get::<Uuid>("user_uuid")? {
        // TODO: Check that the token is valid
        log::debug!("User with id `{:?}` already logged in", user_uuid);

        return Ok(web::Json(BasicResponse {
            msg: "http://localhost:5173/".to_string(),
        }));
        // return Ok(web::Redirect::to("http://localhost:5173/"));
    }

    let spotify = init_spotify();
    let auth_url = spotify.get_authorize_url(true).unwrap();
    log::info!("Auth URL: {}", auth_url);

    // Ok(web::Redirect::to(auth_url))
    Ok(web::Json(BasicResponse { msg: auth_url }))
}

/// OAuth2 callback route
/// This route is called by spotify to complete the OAuth2 authorization process
pub async fn callback(
    response_code: web::Query<ResponseCode>,
    session: Session,
    db: web::Data<DatabaseConnection>,
) -> Result<impl Responder> {
    let spotify = init_spotify();

    // let res = spotify.request_token(&code).await;
    let token = spotify.request_token(&response_code.code).await;

    match token {
        Ok(_) => {
            log::info!("Request user token successful");
            let token_mutex = spotify.get_token();
            let mut token = token_mutex.lock().await.unwrap();
            let token: &Token = token.as_mut().expect("Token can't be empty as this point");

            let db = db.get_ref();

            // TODO: store token in db using session id
            log::info!("Token: {}", token.access_token);

            // TODO: maybe get user before taing token out of spotify we could avoid 2 new spotify objects
            let spotify = AuthCodeSpotify::from_token(token.clone());

            let current_user = spotify.me().await.unwrap();
            let user_spotify_id = current_user.id.to_string();
            log::info!("Current user has spotify id : {}", &user_spotify_id);
            //current_user.id
            let user: Option<user::Model> = User::find()
                .filter(user::Column::SpotifyId.eq(&user_spotify_id))
                .one(db)
                .await
                .unwrap();
            let current_user_uuid = match user {
                Some(user) => {
                    log::debug!("Retreived User uuid : {}", user.id);
                    user.id
                    // let mut user: user::ActiveModel = user.into();
                    // user.token = Set(token.access_token.clone());
                    // let user: user::Model = user.update(db).await.unwrap();
                    // log::debug!("Token updated for user with id: `{}`", user.id);
                }
                None => {
                    let new_user_uuid = Uuid::new_v4();

                    let new_user = user::ActiveModel {
                        id: ActiveValue::Set(new_user_uuid),
                        name: ActiveValue::Set(
                            current_user
                                .display_name
                                .unwrap_or_else(|| String::from("No name")),
                        ),
                        spotify_id: ActiveValue::Set(Some(user_spotify_id)),
                        email: ActiveValue::Set(
                            current_user
                                .email
                                .unwrap_or_else(|| String::from("No name")),
                        ),
                    };
                    let res = User::insert(new_user).exec(db).await.unwrap();
                    log::debug!("Inserted user with id: `{}`", res.last_insert_id);

                    new_user_uuid
                }
            };
            session.renew();
            session.insert("user_uuid", current_user_uuid).unwrap();
            session.insert("spotify_token", token).unwrap();
            session.insert("test", "test".to_string()).unwrap();
            log::debug!("session entries : {:?}", session.entries());

            // return AppResponse::Json(BasicResponse { msg: String::from("logged in!") });
            //AppResponse::Redirect(Redirect::to("http://localhost:5173/"))
            Ok(web::Redirect::to("http://localhost:5173/"))
        }
        Err(err) => {
            log::error!("Failed to get user token {:?}", err);
            // return AppResponse::Json(BasicResponse { msg: String::from("not logged in") });
            // let mut context = HashMap::new();
            // context.insert("err_msg", "Failed to get token!");
            // AppResponse::Template(Template::render("error", context))
            Ok(web::Redirect::to("http://localhost:5173/error"))
        }
    }
    // return Json(BasicResponse { msg: String::from("not logged in") });
}
