// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use std::{sync::{Arc, Mutex}, thread, time::Duration};
use tauri::{Emitter, State};

#[derive(Default, Clone)]
struct AppState {
    count: Option<u16>, // カウント値を管理（Optionで初期化を判定）
    running: bool,      // 処理が動いているかを管理
    stop_signal: bool, // 停止信号
}

// ready関数: 初期値を設定
#[tauri::command]
fn ready(count: u16, app_handle: tauri::AppHandle, state: State<'_, Arc<Mutex<AppState>>>) {
    let mut app_state = state.lock().unwrap();
    // 動作中は何もしない
    if app_state.running {
        return;
    }
    // 動作中で無い場合は初期化を行う
    app_state.count = Some(count);
    app_state.stop_signal = false;
    // ここは、emitでメッセージを返すほどのことも無いのだが
    // 手抜きでカウントダウンスレッドと同じ方法を採用している。
    // すぐ終わる同期処理なので、本来は戻り値で扱う方が自然だろう
    app_handle.emit("count-update", app_state.count).unwrap(); // フロントエンドに通知
}

// start関数: カウントダウンを開始
#[tauri::command]
fn start(app_handle: tauri::AppHandle, state: State<'_, Arc<Mutex<AppState>>>) {
    let mut app_sate = state.lock().unwrap();
    if app_sate.count.is_none() {
        return; // カウントが初期化されていない場合は何もしない
    }
    if app_sate.running {
        return; // 動作中の場合は何もしない
    }
    // カウントダウンスレッド起動
    app_sate.running = true; // 動作中に設定
    app_sate.stop_signal = false; // 停止信号をリセット
    let state = state.inner().clone();  // カウントダウンスレッドに渡すためにclone
    let app_handle = app_handle.clone(); // カウントダウンスレッドに渡すためにclone
    thread::spawn(move || {
        loop {
            // ブロック内でのみロックするようにしている。
            // このブロックを抜けた時点でstateは解放され、Mutexも解放される。
            {
                // カウントダウン処理時のみmutexをロック
                let mut state = state.lock().unwrap(); // ミュータブルにするためにロック // 1秒待機
                // 停止信号を確認
                if state.stop_signal {
                    state.running = false;
                    state.stop_signal = false;
                    state.count = None;
                    break
                }
                // カウントを取得・更新
                match state.count {
                    // カウントが0になった場合や、何らかの理由でNoneになった場合は終了
                    Some(0) | None => {
                        state.running = false;
                        state.stop_signal = false;
                        state.count = None;
                        break
                    },
                    // それ以外はカウントダウン
                    _ => state.count = Some(state.count.unwrap() - 1),
                }
                app_handle.emit("count-update", state.count).unwrap(); // フロントエンドに通知
            }
            // 1秒待っている間はロックを解放たいので、これはブロック外に出す
            thread::sleep(Duration::from_secs(1));
        }
    });
}

// stop関数: カウントダウンを停止
#[tauri::command]
fn stop(state: State<'_, Arc<Mutex<AppState>>>) {
    let mut appp_state = state.lock().unwrap();
    if appp_state.running {
        appp_state.stop_signal = true; // 停止信号をセット
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(Arc::new(Mutex::new(AppState::default()))) // Stateを管理
        .invoke_handler(tauri::generate_handler![ready, start, stop])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
