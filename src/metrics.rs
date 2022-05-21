use std::net::SocketAddr;

use librespot_playback::player::PlayerEvent;
use once_cell::sync::Lazy;
use prometheus::{
    opts, register_int_counter, register_int_gauge, Encoder, IntCounter, IntGauge, TextEncoder,
};
use warp::{http::Response, Filter};

static TRACKS_PLAYED: Lazy<IntCounter> = Lazy::new(|| {
    register_int_counter!(opts!(
        "spotifyd_tracks_total",
        "Number of Spotify tracks played",
    ))
    .unwrap()
});

static PAUSE_COUNT: Lazy<IntCounter> = Lazy::new(|| {
    register_int_counter!(opts!("spotifyd_pause_count", "Number of pause events")).unwrap()
});

static VOLUME_LEVEL: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge!(opts!(
        "spotifyd_playback_volume",
        "Volume level of the Spotify player",
    ))
    .unwrap()
});

static UNAVAILABLE_TRACKS: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge!(opts!(
        "spotifyd_unavailable_tracks",
        "Number of unavailable tracks that attempted to play",
    ))
    .unwrap()
});

static IS_PLAYING: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge!(opts!(
        "spotifyd_is_playing",
        "Whether or not the player is currently playing",
    ))
    .unwrap()
});

pub async fn run_server(addr: SocketAddr) {
    let filter = warp::path("metrics").map(|| {
        let metrics = prometheus::gather();
        let mut buffer = vec![];
        let encoder = TextEncoder::new();
        encoder
            .encode(&metrics, &mut buffer)
            .expect("failed to encode");
        let response = Response::builder()
            .status(200)
            .header("content-type", encoder.format_type())
            .body(buffer)
            .expect("invalid response");
        response
    });
    log::info!("Starting metrics server on {}", addr);
    warp::serve(filter).run(addr).await;
}

pub fn handle_playback_event(event: &PlayerEvent) {
    match event {
        PlayerEvent::Started { .. } => {
            TRACKS_PLAYED.inc();
        }
        PlayerEvent::Changed {
            old_track_id,
            new_track_id,
            ..
        } => {
            if old_track_id != new_track_id {
                TRACKS_PLAYED.inc();
            }
        }
        PlayerEvent::Playing { .. } => {
            IS_PLAYING.set(1);
        }
        PlayerEvent::Paused { .. } => {
            PAUSE_COUNT.inc();
            IS_PLAYING.set(0);
        }
        PlayerEvent::Stopped { .. } => {
            IS_PLAYING.set(0);
        }
        PlayerEvent::Unavailable { .. } => {
            UNAVAILABLE_TRACKS.inc();
        }
        PlayerEvent::VolumeSet { volume, .. } => {
            VOLUME_LEVEL.set(*volume as i64);
        }
        _ => {}
    }
}
