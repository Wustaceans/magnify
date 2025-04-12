use anyhow::Result;
use bytes::Bytes;
use iced::{
    Element, Length, Task,
    widget::{button, column, container, image, image::Handle, row, text, text_input},
};
use rfd::FileDialog;
use serde::{Deserialize, Serialize};
use std::{env, fmt::format, path::Path, path::PathBuf};
use tokio::{
    fs::{self, File},
    io::AsyncWriteExt,
};

#[derive(Debug, Deserialize, Default, Clone)]
struct Information {
    id: u64,
    username: String,
    avatar_url: String,
    banner_url: String,
    global_name: String,
    date_created: u32,
    has_nitro: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub username: String,
    pub discriminator: String,
    pub global_name: Option<String>,
    pub avatar: Option<String>,
    #[serde(default)]
    pub bot: Option<bool>,
    #[serde(default)]
    pub system: Option<bool>,
    #[serde(default)]
    pub mfa_enabled: Option<bool>,
    pub banner: Option<String>,
    #[serde(default)]
    pub accent_color: Option<i32>,
    #[serde(default)]
    pub locale: Option<String>,
    #[serde(default)]
    pub verified: Option<bool>,
    pub email: Option<String>,
    #[serde(default)]
    pub flags: Option<i32>,
    #[serde(default)]
    pub premium_type: Option<i32>,
    #[serde(default)]
    pub public_flags: Option<i32>,
    #[serde(default)]
    pub avatar_decoration_data: serde_json::Value,
}

#[derive(Default)]
struct App {
    info: Information,
    user_id: String,
    request_made: bool,
}
#[derive(Debug, Clone)]
enum Message {
    Get,
    IdChanged(String),
    Response(Result<Information, String>),
    DownloadPfp,
}

impl App {
    async fn download_avatar(id: String, avatar_url: String) -> Result<Bytes> {
        let extension = if avatar_url.contains("a_") {
            "gif"
        } else {
            "png"
        };

        let url = format!("https://cdn.discordapp.com/avatars/{id}/{avatar_url}.{extension}");
        let request = reqwest::get(&url).await?;

        Ok(request.bytes().await?)
    }

    async fn download_banner(id: String, banner_url: String) -> Result<Bytes> {
        let extension = if banner_url.contains("a_") {
            "gif"
        } else {
            "png"
        };

        let url = format!("https://cdn.discordapp.com/banners/{id}/{banner_url}.{extension}");
        let request = reqwest::get(&url).await?;

        Ok(request.bytes().await?)
    }

    async fn save_avatar_to(dest: PathBuf, contents: Bytes) -> Result<()> {
        let mut file = File::create(&dest).await?;

        file.write_all(&contents).await?;
        Ok(())
    }

    async fn check_if_file_exists(file_path: String) -> Option<String> {
        let extensions = vec![".png", ".gif"];

        for ext in extensions {
            let full_path = format!("{}{}", file_path, ext);
            if Path::new(&full_path).exists() {
                return Some(ext.to_string());
            }
        }
        None
    }

    async fn request(url: String) -> Result<Information> {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();

        runtime.block_on(async move {
            let bot_token = env::var("DISCORD_BOT_TOKEN").unwrap();
            let client = reqwest::Client::new();
            let token = format!("Bot {}", bot_token);

            let response = client
                .get(&url)
                .header("Authorization", &token)
                .send()
                .await?;

            let data = response.text().await?;

            println!("deserializing data");
            let discord_info: User = serde_json::from_str(&data).unwrap();

            let info = Information {
                id: discord_info.id.parse().unwrap_or_default(),
                username: discord_info.username,
                global_name: discord_info.global_name.unwrap_or_default(),
                avatar_url: discord_info.avatar.unwrap_or_default(),
                banner_url: discord_info.banner.unwrap_or_default(),
                date_created: 0,
                has_nitro: discord_info.premium_type.unwrap_or_default() == 1,
            };

            println!("information: {:#?}", info);

            let avatar =
                Self::download_avatar(info.id.to_string(), info.avatar_url.clone()).await?;
            let banner =
                Self::download_banner(info.id.to_string(), info.banner_url.clone()).await?;

            Self::save_avatar_to(PathBuf::from(format!("/tmp/{}", info.id)), avatar).await?;
            Self::save_avatar_to(PathBuf::from(format!("/tmp/{}", info.banner_url)), banner)
                .await?;

            return Ok(info);
        })
    }

    fn update(&mut self, msg: Message) -> Task<Message> {
        const BASE_URL: &str = "https://discord.com/api/v10/users";
        match msg {
            Message::Response(result) => {
                if let Ok(info) = result {
                    self.info = info;
                }
                Task::none()
            }
            Message::Get => {
                self.info.id = self.user_id.parse().unwrap();
                let url = format!("{}/{}", BASE_URL, self.info.id);

                self.request_made = true;

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
            Message::IdChanged(new_id) => {
                self.user_id = new_id;
                Task::none()
            }
            Message::DownloadPfp => {
                FileDialog::new()
                    .add_filter("image", &["jpg", "png"])
                    .save_file();
                Task::none()
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let pfp_check = Self::check_if_file_exists(format!("/tmp/{}", info.id)).await;
        let banner_check = Self::check_if_file_exists(format!("/tmp/{}", info.banner_url)).await;

        let pfp_path: String = format!("/tmp/{}.{}", self.info.id, pfp_check);

        let banner_path: String = format!("/tmp/{}.{}", self.info.banner_url, banner_check);

        let mut col = column![
            text_input("user id", &self.user_id).on_input(Message::IdChanged),
            button(text("Get")).on_press(Message::Get),
        ];

        if self.request_made {
            col = col.push(image(Handle::from_path(banner_path)));
            col = col.push(image(Handle::from_path(pfp_path)));
            col = col.push(text(format!("Username: {}", &self.info.username)));
            col = col.push(text(format!("Global Name: {}", &self.info.global_name)));
        }

        let row = row![col].spacing(10);
        container(row)
            .width(Length::Fill)
            .height(Length::Fill)
            .center(Length::Fill)
            .into()
    }
}

#[tokio::main]
async fn main() -> iced::Result {
    dotenv::dotenv().ok();

    let theme = |_s: &App| iced::Theme::Dark;
    iced::application("Magnify", App::update, App::view)
        .centered()
        .theme(theme)
        .run()
}
