# Tauriでのバックエンド⇒フロントエンドの非同期通信のサンプル

## 1. 概要

リアルタイムに動作するアプリを作るうえでは、当然バックエンドから非同期にデータを受信したい、という要求が出てくる。  
本サンプルでは、`emit`を使いグローバルメッセージをバックエンドからフロントエンドに送信する方法を説明する。  

## 2. アプリの説明

非同期ということで、特にひねりは無く、カウントダウンタイマーを実装した。  

- Readyでカウンターを設定(本サンプルでは10秒固定)
- Startでカウントダウン開始
- Stopでカウントダウン停止

一応、カウントダウン中のReadyとStartを無効化する処理が入っている。  
これにより、カウントダウンスレッド動作中に、Stateにフロントエンド側からアクセスることで、安全にStateをスレッド間で共有できることを確認している。  

## 3. 実装の解説

今回の例ではglobalメッセージを使用している。

### 3.1. フロントエンド側

グローバルの`listen`を使い、グローバルメッセージ受信時の処理を登録している。  
今回のメッセージ名は`count-update`である。  
受信時に、カウント表示を更新している。  

それ以外はボタンを配置して、Tarui側で定義したコマンドを呼び出しているだけなので、特に説明なし。  

### 3.2. バックエンド側  

[参考にした公式のdoc](https://v2.tauri.app/develop/state-management/#mutability)。

タイマー自体のStateを表す構造体は、`AppState`として定義している。  
タイマーは当然更新が必要なので、MutableなState管理の説明を参照している。  
加えて、複数のスレッドから同時アクセスされる可能性があるため、さらにArcでラップしている。  
よって、最終的に`namage`に渡しているのは`Arc<Mutex<AppState>>`である。

上記ドキュメントと異なる点は、

```rust
let data = app.state::<AppState>();
```

として、Stateを取得しているのでは無く、メソッドの引数として取得している。  
(tauri::commandのテンプレートの機能)  

そのため、`start`メソッドの定義は以下のようになっている。

```rust
fn start(app_handle: tauri::AppHandle, state: State<'_, Arc<Mutex<AppState>>>)
```

カウントダウンスレッドには、`Mutex<AppState>`に対する`Arc`のクローンを渡して、Stateを共有している。  
ここで、クローンを作成する際の記述は以下の様になっている。  

```rust
let state = state.inner().clone();  // カウントダウンスレッドに渡すためにclone
```

ここで、`innter()`とは?という話になる。  
右辺の`state`は`start()`の引数であるので、そのスコープは`start()`の中である。  
しかし、作成したカウントダウンスレッドは、`start()`より生存時間が長い可能性があるため(というか、このケースでは実際に長い)、`state.clone()` するとコンパイラに怒られる。  

TauriのStateの機能である`inner()`を使うことで、実際で内部で管理しているState(= 引数のstateの元)にアクセスすることができる。  
元のStateはTauriのアプリ中生存しているため、`inner()`でアクセスすることで、上記の問題を解決する。  