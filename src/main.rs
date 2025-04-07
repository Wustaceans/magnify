use anyhow::Result;
use iced::{
    Element, Task,
    widget::{column, container, row, text_input},
};
use serde::Deserialize;

#[derive(Debug, Deserialize, Default, Clone)]
struct Information {
    id: u64,
    username: String,
    avatar_url: String,
    date_created: u32,
    has_nitro: bool,
}

#[derive(Default)]
struct App {
    info: Information,
}

#[derive(Debug, Clone)]
enum Message {
    Get,
    Response(Result<Information, String>),
}

impl App {
    async fn request(url: String) -> Result<Information> {
        let client = reqwest::Client::new();
        let response = client
            .get(&url)
            .header("Authorization", "Bot")
            .send()
            .await?;
        let data = response.text().await?;

        let info: Information = serde_json::from_str(&data)?;
        Ok(info)
    }

    fn update(&mut self, msg: Message) -> Task<Message> {
        const BASE_URL: &str = "https://discord.com/api/v9/users";
        match msg {
            Message::Response(result) => {
                if let Ok(info) = result {
                    self.info = info;
                }
                Task::none()
            }
            Message::Get => {
                let url = format!("{}/{}", BASE_URL, self.info.id);

                Task::perform(
                    // So,
                    // - async: create a future (your JS promise)
                    // - move: moves ownership of the data to the block
                    // - map_err: converts the error type to a string, keeping the success value
                    // The reason I'm using async move is because Task::perform requires a future
                    async move { Self::request(url).await.map_err(|e| e.to_string()) },
                    // The return value of request() is sent to the closure of the second argument
                    |result| Message::Response(result),
                )
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let col = column![text_input("Username", &self.info.username)];
        let row = row![col].spacing(10);
        container(row).width(400.0).into()
    }
}

#[tokio::main]
async fn main() -> iced::Result {
    let theme = |_s: &App| iced::Theme::Dark;
    iced::application("Magnify", App::update, App::view)
        .centered()
        .theme(theme)
        .run()
}
