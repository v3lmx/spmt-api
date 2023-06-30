# spmt api prototype

Manage your playlists using custom rules and tags

⚠️ This repository is meant to be a proof of concept, some stuff might be sub-optimal or outright broken.

## Concept

- Sync your music library into `spmt` (better name ideas welcome)
- Define tags on your songs
- Define playlists based on rules that can make use of every metadata the music platform offers (genre, BPM, liked songs, whether a song is in another playlist, etc.) and your custom tags

Example: You can make a playlist for your birthday party with all the songs that you liked, are in one of your playlists, are faster than 100 BPM, and are of Dance genre

## Contributing

This is the goal, but current development is far from it. I like to make this by myself for now, I like to learn this way. However, if you like the idea, feel free to make your own implementation (and make it free and open source ❤️). If my project gets more mature, I may accept contributions in the future.

## Development 

### Requirements

- a postgres install
- rust (cargo)

The script setup.sql can be used to setup the db with a user and tables

### Running 

```
./startdb.sh
cargo run
```