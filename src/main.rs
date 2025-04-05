use std::fmt::format;

use anyhow::Result;

use iced::{
    Element, Task,
    widget::{column, container, row, text_input},
};
use serde_json::Value;

#[derive(Default)]
struct Information {
    id: u128,
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
    Response,
}

impl App {
    async fn request(&self, url: String) -> Result<()> {
        let client = reqwest::Client::new();

        let response = client
            .get(url.as_str())
            .header("Authorization", "Bot")
            .send()
            .await?;

        let data = response.text().await?;

        let v: Value = serde_json::from_str(data.as_str())?;

        Ok(())
    }

    fn update(&mut self, msg: Message) -> Task<Message> {
        const BASE_URL: &str = "https://discord.com/api/v9/users/";

        match msg {
            Message::Response => Task::none(),
            Message::Get => Task::perform(
                Self::request(App, format!("{BASE_URL}{}", self.info.id)),
                |_| Message::Response,
            ),
        }
    }

    fn view(&self) -> Element<Message> {
        let col = column![text_input("a", self.info.username.as_str())];

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
