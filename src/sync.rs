use actix_session::Session;
use futures::stream::TryStreamExt;
use rspotify::model::FullTrack;
use rspotify::{model::SavedTrack, prelude::OAuthClient, AuthCodeSpotify, Token};
use sea_orm::{ActiveValue, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use uuid::Uuid;

use crate::entities::artist;
use crate::entities::prelude::{Artist, Track};
use crate::{entities::track, error::ServerError};

pub async fn sync(session: Session, db: &DatabaseConnection) -> Result<(), ServerError> {
    let spotify_token = match session.get::<Token>("spotify_token") {
        Ok(Some(token)) => token,
        Ok(None) | Err(_) => {
            // TODO: request other token???
            return Err(ServerError::Error);
        }
    };

    let spotify = AuthCodeSpotify::from_token(spotify_token);
    // let current_user = spotify.current_user().await.unwrap();

    let tracks = spotify.current_user_saved_tracks(None);

    let start_time = std::time::Instant::now();

    // let mut new_tracks = vec![];

    let tracks = tracks
        .try_for_each_concurrent(0, |track| async move {
            // let res = save_track(track.track, db).await;
            // match res {
            //     Ok(track) => log::debug!("track added: {}", track),
            //     Err(err) => log::error!("Error with track: {}", err),
            // };
            // &new_tracks.push(1.clone());
            Ok(())
        })
        .await
        .unwrap();

    let total_time =
        start_time.elapsed().as_secs() as f64 + start_time.elapsed().subsec_nanos() as f64 / 1e9;

    log::debug!("All tracks saved in {total_time}");
    Ok(())
}

// TODO this is horrible
async fn save_track(track: FullTrack, db: &DatabaseConnection) -> Result<String, ServerError> {
    let spotify_track_id = track.id.unwrap();
    let existing_track: Option<track::Model> = Track::find()
        .filter(track::Column::SpotifyId.eq(spotify_track_id.to_string()))
        .one(db)
        .await
        .unwrap();

    if existing_track.is_some() {
        return Ok(track.name.clone());
    }

    let spotify_artist = track.artists[0].clone();
    let spotify_artist_id = spotify_artist.id.unwrap().to_string();
    let existing_artist: Option<artist::Model> = Artist::find()
        .filter(artist::Column::SpotifyId.eq(spotify_artist_id.clone()))
        .one(db)
        .await
        .unwrap();

    let artist_id = match existing_artist {
        Some(artist) => artist.id,
        None => {
            let new_artist_uuid = Uuid::new_v4();
            let new_artist = artist::ActiveModel {
                id: ActiveValue::Set(new_artist_uuid),
                spotify_id: ActiveValue::Set(spotify_artist_id),
                name: ActiveValue::Set(spotify_artist.name),
            };
            let res = Artist::insert(new_artist).exec(db).await.unwrap();
            new_artist_uuid
        }
    };

    let new_track_uuid = Uuid::new_v4();
    let new_track = track::ActiveModel {
        id: ActiveValue::Set(new_track_uuid),
        spotify_id: ActiveValue::Set(spotify_track_id.to_string()),
        name: ActiveValue::Set(track.name.clone()),
        artist_id: ActiveValue::Set(artist_id),
    };
    let res = Track::insert(new_track).exec(db).await.unwrap();
    Ok(track.name.clone())
}
