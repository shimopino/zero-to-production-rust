-- Add migration script here
BEGIN;
    -- NOT NULLを適用する前に NULL なカラムに対してデータを挿入する
    UPDATE subscriptions
        SET status = 'confirmed'
        WHERE status IS NULL;
    -- データ挿入が成功した後でなければ NOT NULL への変更は失敗する
    ALTER TABLE subscriptions ALTER COLUMN status SET NOT NULL;
    -- 全体が完了して初めてコミットを行う
COMMIT;