use reqwest::Client;

use crate::domain::SubscriberEmail;

#[derive(Clone)]
pub struct EmailClient {
    http_client: Client,
    base_url: String,
    sender: SubscriberEmail,
}

impl EmailClient {
    pub fn new(base_url: String, sender: SubscriberEmail) -> Self {
        Self {
            http_client: Client::new(),
            base_url,
            sender,
        }
    }

    pub async fn send_email(
        &self,
        recipient: SubscriberEmail,
        subject: &str,
        html_content: &str,
        text_content: &str,
    ) -> Result<(), String> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use crate::email_client::EmailClient;
    use fake::faker::lorem::en::Paragraph;
    use fake::faker::{internet::en::SafeEmail, lorem::en::Sentence};
    use fake::Fake;
    use wiremock::{matchers::any, Mock, MockServer, ResponseTemplate};

    use crate::domain::SubscriberEmail;

    #[tokio::test]
    async fn send_email_fires_a_request_to_base_url() {
        // Arrange
        // ランダムなポートを使用してバックグラウンドでサーバーを起動する
        // uriメソッドでURLを取得可能
        let mock_server = MockServer::start().await;
        let fake_email = SafeEmail().fake();
        let sender = SubscriberEmail::parse(fake_email).unwrap();
        let email_client = EmailClient::new(mock_server.uri(), sender);

        // Mockサーバーからのレスポンスのモックを設定する
        Mock::given(any())
            .respond_with(ResponseTemplate::new(200))
            // 条件と一致するリクエストを1つだけ受け取ることができる
            .expect(1)
            .mount(&mock_server)
            .await;

        let subscriber_email = SubscriberEmail::parse(SafeEmail().fake()).unwrap();
        let subject = Sentence(1..2).fake::<String>();
        let content = Paragraph(1..10).fake::<String>();

        // Act
        let _ = email_client
            .send_email(subscriber_email, &subject, &content, &content)
            .await;

        // Assert
    }
}
