use actix_session::Session;
use actix_web::{error, web, Responder, Result};
use rspotify::{
    model::TrackId,
    prelude::{OAuthClient, PlayableId},
    scopes, AuthCodeSpotify, Config, Credentials, OAuth, Token,
};
use uuid::Uuid;

use crate::BasicResponse;

pub fn init_spotify() -> AuthCodeSpotify {
    let config = Config {
        token_cached: false,
        ..Default::default()
    };

    let oauth = OAuth {
        scopes: scopes!(
            "user-read-currently-playing",
            "user-read-recently-played",
            "playlist-modify-private",
            "user-library-read"
        ),
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
        Ok(None) | Err(_) => return Err(error::ErrorUnauthorized("No user logged in.")),
    };
    let spotify_token = match session.get::<Token>("spotify_token") {
        Ok(Some(token)) => token,
        Ok(None) | Err(_) => {
            // TODO: request other token???
            return Err(error::ErrorUnauthorized("No token"));
        }
    };

    let spotify = AuthCodeSpotify::from_token(spotify_token);

    let playlists = spotify
        .current_user_playlists_manual(Some(10), None)
        .await
        .unwrap()
        .items;
    let json = serde_json::to_string(&playlists).unwrap();
    Ok(web::Json(BasicResponse { msg: json }))
}

pub async fn make_playlist_wip(session: Session) -> Result<impl Responder> {
    let user_uuid = match session.get::<Uuid>("user_uuid") {
        Ok(Some(uuid)) => uuid,
        Ok(None) | Err(_) => return Err(error::ErrorUnauthorized("No user logged in.")),
    };
    let spotify_token = match session.get::<Token>("spotify_token") {
        Ok(Some(token)) => token,
        Ok(None) | Err(_) => {
            // TODO: request other token???
            return Err(error::ErrorUnauthorized("No token"));
        }
    };

    let spotify = AuthCodeSpotify::from_token(spotify_token);
    let current_user = spotify.current_user().await.unwrap();

    let new_playlist = spotify
        .user_playlist_create(
            current_user.id,
            "SPMT-test",
            Some(false),
            Some(false),
            Some("SPMT test playlist"),
        )
        .await
        .unwrap();

    let tracks: Vec<PlayableId> = spotify
        .current_user_recently_played(Some(10), None)
        .await
        .unwrap()
        .items
        .into_iter()
        .filter(|track| track.track.id.is_some())
        .map(|track| PlayableId::Track(track.track.id.unwrap()))
        .collect();

    spotify
        .playlist_add_items(new_playlist.id, tracks, None)
        .await
        .unwrap();

    Ok(web::Json(BasicResponse {
        msg: "ok".to_string(),
    }))
}
