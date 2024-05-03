use chrono::{TimeZone, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Benben {
    pub id: i32,
    pub user_id: i32,
    pub user_color: String,
    pub username: String,
    pub content: String,
    pub hash: String,

    pub time: i64,
    pub grab_time: i64,
}

impl TryFrom<Benben> for crate::feed::Model {
    type Error = ();

    fn try_from(b: Benben) -> Result<Self, Self::Error> {
        let time = match Utc.timestamp_opt(b.time, 0).latest() {
            Some(x) => x.naive_local(),
            None => return Err(()),
        };

        let grab_time = match Utc.timestamp_opt(b.grab_time, 0).latest() {
            Some(x) => x.naive_local(),
            None => return Err(()),
        };

        Ok(Self {
            id: b.id,
            user_id: b.user_id,
            user_color: b.user_color,
            username: b.username,
            content: b.content,
            hash: b.hash,
            time,
            grab_time,
        })
    }
}

impl Benben {
    pub fn to_model(self) -> Result<Self, <Benben as TryFrom<Benben>>::Error> {
        self.try_into()
    }
}
