-- migrations/{timestamp}_create_subscriptions_table.sql
-- Create Subscriptions Table
CREATE TABLE subscriptions(
    id uuid NOT NULL,
    PRIMARY KEY (id),
    -- メールアドレスには一意制約を設けている
    -- 型には文字数制限を加えることを考慮しないので、TEXT型を使用している
    email TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    -- 購読を開始したタイムスタンプ
    subscribed_at timestamptz NOT NULL
);