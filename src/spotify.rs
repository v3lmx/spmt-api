use actix_session::Session;
use actix_web::{Result, Responder, error, web};
use rspotify::{AuthCodeSpotify, OAuth, Config, scopes, Credentials, Token, prelude::OAuthClient};
use uuid::Uuid;

use crate::BasicResponse;

pub fn init_spotify() -> AuthCodeSpotify {
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

pub async fn get_playlists(session: Session) -> Result<impl Responder> {
    let user_uuid = match session.get::<Uuid>("user_uuid") {
        Ok(Some(uuid)) => uuid,
        Ok(None) | Err(_) => {
            return Err(error::ErrorUnauthorized("No user logged in."))
        },
    };
    let spotify_token = match session.get::<Token>("spotify_token") {
        Ok(Some(token)) => token,
        Ok(None) | Err(_) => {
            // TODO: request other token???
            return Err(error::ErrorUnauthorized("No token"))
        },
    };

    let spotify = AuthCodeSpotify::from_token(spotify_token);

    let playlists = spotify.current_user_playlists_manual(Some(10), None).await.unwrap().items;
    let json = serde_json::to_string(&playlists).unwrap();
    Ok(web::Json(BasicResponse { msg: json }))
}