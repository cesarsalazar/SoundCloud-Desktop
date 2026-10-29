use std::time::Duration;

use crate::rt::AppHandle;
use tauri::{Manager, State};

use crate::audio::device;
use crate::audio::engine;
use crate::audio::eq_curve;
use crate::audio::eq_preset::{self, EqFormat, EqImport};
use crate::audio::silence;
use crate::audio::state::AudioState;
use crate::audio::timing;
use crate::audio::types::{AudioLoadResult, AudioSink};
use crate::shared::blocking::run_blocking;

#[tauri::command]
pub async fn audio_load_file(
    path: String,
    cache_key: Option<String>,
    start_paused: bool,
    crossfade_ms: Option<u64>,
    app: AppHandle,
    state: State<'_, AudioState>,
) -> Result<AudioLoadResult, String> {
    let normalization_cache_dir = app
        .path()
        .app_cache_dir()
        .ok()
        .map(|dir| dir.join("audio-normalization"));
    let result = engine::load_file(
        path,
        normalization_cache_dir,
        cache_key,
        start_paused,
        crossfade_ms.filter(|&ms| ms > 0).map(Duration::from_millis),
        &app,
        state,
    )
    .await?;
    silence::scan(&app);
    Ok(result)
}

#[tauri::command]
pub fn audio_play(state: State<'_, AudioState>) {
    engine::play(state);
}

#[tauri::command]
pub fn audio_pause(state: State<'_, AudioState>) {
    engine::pause(state);
}

#[tauri::command]
pub fn audio_stop(state: State<'_, AudioState>) {
    engine::stop(state);
}

#[tauri::command]
pub fn audio_seek(position: f64, state: State<'_, AudioState>) -> Result<(), String> {
    engine::seek(position, state)
}

#[tauri::command]
pub fn audio_set_volume(volume: f64, state: State<'_, AudioState>) {
    engine::set_volume(volume, state);
}

#[tauri::command]
pub fn audio_set_fft_enabled(enabled: bool, state: State<'_, AudioState>) {
    state
        .analyser_buffer
        .enabled
        .store(enabled, std::sync::atomic::Ordering::Relaxed);
}

#[tauri::command]
pub fn audio_set_playback_rate(rate: f64, state: State<'_, AudioState>) {
    engine::set_playback_rate(rate, state);
}

#[tauri::command]
pub fn audio_set_pitch_ratio(ratio: f64, state: State<'_, AudioState>) {
    engine::set_pitch_ratio(ratio, state);
}

#[tauri::command]
pub fn audio_set_ab_loop(a: Option<f64>, b: Option<f64>, state: State<'_, AudioState>) {
    engine::set_ab_loop(a, b, state);
}

#[tauri::command]
pub fn audio_set_skip_silence(enabled: bool, app: AppHandle) {
    silence::set_enabled(&app, enabled);
}

#[tauri::command]
pub fn audio_get_position(state: State<'_, AudioState>) -> f64 {
    engine::get_position(state)
}

#[tauri::command]
pub fn audio_set_eq(
    enabled: bool,
    gains: Vec<f64>,
    preamp: Option<f64>,
    state: State<'_, AudioState>,
) {
    engine::set_eq(enabled, &gains, preamp, state);
}

#[tauri::command]
pub fn audio_eq_auto_preamp(gains: Vec<f64>) -> f64 {
    let gains = eq_curve::band_gains(&gains);
    eq_curve::resolve_preamp(&gains, None)
}

#[tauri::command]
pub fn audio_eq_import(text: String) -> Result<EqImport, String> {
    eq_preset::import(&text)
}

const EQ_IMPORT_EXTENSIONS: [&str; 2] = ["txt", "peace"];
const EQ_SAVE_EXTENSIONS: [&str; 1] = ["txt"];

/// The webview hands us the path, so only preset files are touched: no reading
/// or overwriting arbitrary files (shell profiles, launch agents, keys).
fn preset_path(path: &str, allowed: &[&str]) -> Result<std::path::PathBuf, String> {
    let path = std::path::PathBuf::from(path);
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(str::to_ascii_lowercase)
        .unwrap_or_default();
    if !path.is_absolute() || !allowed.contains(&ext.as_str()) {
        return Err("unsupported preset path".to_string());
    }
    Ok(path)
}

#[tauri::command]
pub async fn audio_eq_import_file(path: String) -> Result<EqImport, String> {
    let path = preset_path(&path, &EQ_IMPORT_EXTENSIONS)?;
    let bytes = tokio::fs::read(&path).await.map_err(|e| e.to_string())?;
    eq_preset::import(&eq_preset::decode(&bytes))
}

#[tauri::command]
pub fn audio_eq_export(gains: Vec<f64>, preamp: Option<f64>, format: EqFormat) -> String {
    let gains = eq_curve::band_gains(&gains);
    let preamp = eq_curve::resolve_preamp(&gains, preamp);
    eq_preset::export(&gains, preamp, format)
}

#[tauri::command]
pub async fn audio_eq_save(path: String, text: String) -> Result<(), String> {
    let path = preset_path(&path, &EQ_SAVE_EXTENSIONS)?;
    tokio::fs::write(&path, text)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn audio_set_normalization(enabled: bool, state: State<'_, AudioState>) {
    engine::set_normalization(enabled, state);
}

#[tauri::command]
pub fn audio_is_playing(state: State<'_, AudioState>) -> bool {
    engine::is_playing(state)
}

#[tauri::command]
pub fn audio_set_metadata(
    title: String,
    artist: String,
    cover_url: Option<String>,
    duration_secs: f64,
    state: State<'_, AudioState>,
) {
    engine::set_metadata(title, artist, cover_url, duration_secs, state);
}

#[tauri::command]
pub fn audio_set_playback_state(playing: bool, state: State<'_, AudioState>) {
    engine::set_playback_state(playing, state);
}

#[tauri::command]
pub fn audio_set_media_position(position: f64, state: State<'_, AudioState>) {
    engine::set_media_position(position, state);
}

#[tauri::command]
pub async fn audio_list_devices(app: AppHandle) -> Result<Vec<AudioSink>, String> {
    run_blocking(move || device::list_devices(&app.state::<AudioState>())).await
}

#[tauri::command]
pub async fn audio_switch_device(
    device_name: Option<String>,
    app: AppHandle,
) -> Result<(), String> {
    run_blocking(move || device::switch_device(app.state(), device_name)).await?
}

#[tauri::command]
pub fn audio_set_follow_default_output(follow: bool, state: State<'_, AudioState>) {
    device::set_follow_default_output(state, follow);
}

#[tauri::command]
pub fn audio_set_lyrics_timeline(
    lines: Vec<crate::audio::types::LyricsTimingLine>,
    state: State<'_, AudioState>,
) {
    timing::audio_set_lyrics_timeline(lines, state);
}

#[tauri::command]
pub fn audio_clear_lyrics_timeline(state: State<'_, AudioState>) {
    timing::audio_clear_lyrics_timeline(state);
}

#[tauri::command]
pub fn audio_set_comments_timeline(
    comments: Vec<crate::audio::types::FloatingCommentEvent>,
    state: State<'_, AudioState>,
) {
    timing::audio_set_comments_timeline(comments, state);
}

#[tauri::command]
pub fn audio_clear_comments_timeline(state: State<'_, AudioState>) {
    timing::audio_clear_comments_timeline(state);
}

#[tauri::command]
pub async fn audio_preview_play(
    path: String,
    volume: f64,
    r#gen: u64,
    state: State<'_, AudioState>,
) -> Result<(), String> {
    engine::preview_play(path, volume, r#gen, state).await
}

#[tauri::command]
pub fn audio_preview_stop(fade_ms: u64, r#gen: u64, state: State<'_, AudioState>) {
    engine::preview_stop(fade_ms, r#gen, state);
}
