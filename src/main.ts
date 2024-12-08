import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

// HTML要素の取得
const countDisplay = document.getElementById("count-display") as HTMLElement;
const readyButton = document.getElementById("ready-button") as HTMLButtonElement;
const startButton = document.getElementById("start-button") as HTMLButtonElement;
const stopButton = document.getElementById("stop-button") as HTMLButtonElement;

// 初期化処理
async function setup() {
  // Tauriからのカウント更新イベントをリッスン
  await listen<number>("count-update", (event) => {
    const count = event.payload;
    countDisplay.textContent = `Count: ${count}`;
  });

  // "Ready"ボタンのクリックイベント
  readyButton.addEventListener("click", async () => {
    const initialCount = 10; // 初期カウント値
    await invoke("ready", { count: initialCount });
  });

  // "Start"ボタンのクリックイベント
  startButton.addEventListener("click", async () => {
    await invoke("start");
  });

  // "Stop"ボタンのクリックイベント
  stopButton.addEventListener("click", async () => {
    await invoke("stop");
  });
}

// アプリをセットアップ
setup();
