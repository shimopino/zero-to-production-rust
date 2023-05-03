// main.rsを配置して単一バイナリとしてテストを実行する
// これでファイルを分割しても、そえぞれのテストをコンパイルするのではなく
// テスト全体を1つのファイルとして実行することが可能となる
mod health_check;
mod helpers;
mod newsletter;
mod subscription;
mod subscription_confirm;
