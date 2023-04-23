use tracing::{subscriber::set_global_default, Subscriber};
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::LogTracer;
use tracing_subscriber::{prelude::__tracing_subscriber_SubscriberExt, EnvFilter, Registry};

pub fn get_subscriber(name: String, env_filter: String) -> impl Subscriber + Send + Sync {
    // デフォルトでは INFO レベル以上のログを出力する
    // 環境変数から設定できるように EnvFilter を利用する
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(env_filter));

    // フォーマットされた span を標準出力に出す
    let formatting_layer = BunyanFormattingLayer::new(name, std::io::stdout);

    // with メソッドを使用して機能を拡張する
    // Registryのおかげで複数の機能を簡単に組み合わせることが可能
    Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer)
}

pub fn init_subscriber(subscriber: impl Subscriber + Send + Sync) {
    // Logで出力されている内容をSubscriberに転送する
    LogTracer::init().expect("Failed to set logger");
    set_global_default(subscriber).expect("Failed to set subscriber");
}
