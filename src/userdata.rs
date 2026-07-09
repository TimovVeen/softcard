use std::{collections::HashMap, io, time::Duration};

use directories::ProjectDirs;
use iced::Task;
use serde::{Deserialize, Serialize};

use crate::Screen;

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct UserData {
    pub best_times: HashMap<Screen, Duration>,
    pub best_cards: HashMap<Screen, u32>,
}

impl UserData {
    pub async fn try_load() -> Option<Self> {
        let data_file = ProjectDirs::from("com", "ItsAPixel", "Softcard")?
            .data_dir()
            .join("score.ron");
        let data = smol::fs::read_to_string(data_file).await.ok()?;
        ron::from_str(&data).ok()
    }

    fn write_score(&self) -> Task<io::Result<()>> {
        let data_dir = ProjectDirs::from("com", "ItsAPixel", "Softcard")
            .unwrap()
            .data_dir()
            .to_path_buf();
        let data_file = data_dir.join("score.ron");
        let serialized =
            ron::ser::to_string_pretty(self, ron::ser::PrettyConfig::default()).unwrap();
        Task::future(async move {
            if !data_dir.exists() {
                smol::fs::create_dir_all(data_dir).await?;
            }
            smol::fs::write(data_file, serialized).await
        })
    }

    pub fn add_time(&mut self, screen: Screen, time: Duration) -> Task<io::Result<()>> {
        if self
            .best_times
            .get(&screen)
            .copied()
            .unwrap_or(Duration::MAX)
            > time
        {
            self.best_times.insert(screen, time);
            self.write_score()
        } else {
            Task::none()
        }
    }

    pub fn add_cards(&mut self, screen: Screen, cards: u32) -> Task<io::Result<()>> {
        if self.best_cards.get(&screen).copied().unwrap_or(0_u32) < cards {
            self.best_cards.insert(screen, cards);
            self.write_score()
        } else {
            Task::none()
        }
    }
}
