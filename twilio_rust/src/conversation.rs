use std::{collections::HashMap, fmt};

use reqwest::Method;
use serde::{Deserialize, Serialize};
use strum_macros::{AsRefStr, Display, EnumIter, EnumString};

use crate::{Client, TwilioError};

pub struct Conversations<'a> {
    pub client: &'a Client,
}

#[allow(dead_code)]
#[derive(Deserialize)]
pub struct ConversationPage {
    conversations: Vec<Conversation>,
    meta: ConversationPageMeta,
}

#[allow(dead_code)]
#[derive(Deserialize)]
pub struct ConversationPageMeta {
    page: u16,
    page_size: u16,
    first_page_url: String,
    previous_page_url: Option<String>,
    next_page_url: Option<String>,
    key: String,
}

/// Details related to a specific account.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Conversation {
    pub sid: String,
    pub account_sid: String,
    pub chat_service_sid: String,
    pub messaging_service_sid: String,
    pub unique_name: Option<String>,
    pub friendly_name: Option<String>,
    pub date_created: String,
    pub date_updated: String,
    pub state: State,
    pub url: String,
    pub attributes: String,
    pub timers: Timers,
    pub links: Links,
}

impl fmt::Display for Conversation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} - {}", self.sid, self.state)
    }
}

#[derive(
    AsRefStr, Clone, Display, Debug, EnumIter, EnumString, Serialize, Deserialize, PartialEq,
)]
pub enum State {
    #[strum(serialize = "active")]
    #[serde(rename = "active")]
    Active,
    #[strum(serialize = "inactive")]
    #[serde(rename = "inactive")]
    Inactive,
    #[strum(serialize = "closed")]
    #[serde(rename = "closed")]
    Closed,
}

impl Default for State {
    fn default() -> Self {
        State::Active
    }
}

impl State {
    pub fn as_str(&self) -> &'static str {
        match self {
            &State::Active => "active",
            &State::Inactive => "inactive",
            &State::Closed => "closed",
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Timers {
    pub date_inactive: Option<String>,
    pub date_closed: Option<String>,
}

impl Default for Timers {
    fn default() -> Self {
        Timers {
            date_inactive: None,
            date_closed: None,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Links {
    pub participants: String,
    pub messages: String,
    pub webhooks: String,
}

impl Default for Links {
    fn default() -> Self {
        Links {
            participants: String::from(""),
            messages: String::from(""),
            webhooks: String::from(""),
        }
    }
}

impl<'a> Conversations<'a> {
    /// [Gets a Conversation](https://www.twilio.com/docs/conversations/api/conversation-resource#fetch-a-conversation-resource)
    ///
    /// Takes in a `sid` argument which can also be the conversations `uniqueName`.
    pub fn get(&self, sid: &str) -> Result<Conversation, TwilioError> {
        let conversation = self.client.send_request::<Conversation>(
            Method::GET,
            &format!("https://conversations.twilio.com/v1/Conversations/{}", sid),
            None,
        );

        conversation
    }

    /// [Lists Conversations](https://www.twilio.com/docs/conversations/api/conversation-resource#read-multiple-conversation-resources)
    ///
    /// This will eagerly fetch *all* conversations on the Twilio account and sort by recent message activity.
    /// Takes in `start_date` and `end_date` options to filter results. This should be ISO8601 format e.g. `YYYY-MM-DDT00:00:00Z`.
    ///
    /// Also accepts a `state` option to filter by Conversation state such as closed Conversations.
    pub fn list(
        &self,
        start_date: Option<&str>,
        end_date: Option<&str>,
        state: Option<State>,
    ) -> Result<Vec<Conversation>, TwilioError> {
        let mut params: HashMap<String, &str> = HashMap::new();
        if let Some(start_date) = start_date {
            params.insert(String::from("StartDate"), start_date);
        }
        if let Some(end_date) = end_date {
            params.insert(String::from("EndDate"), end_date);
        }
        if let Some(state) = state {
            params.insert(String::from("State"), &state.to_string());
        }

        let mut conversations_page = self.client.send_request::<ConversationPage>(
            Method::GET,
            "https://conversations.twilio.com/v1/Conversations",
            None,
        )?;

        let mut results: Vec<Conversation> = conversations_page.conversations;

        while (conversations_page.meta.next_page_url).is_some() {
            conversations_page = self.client.send_request::<ConversationPage>(
                Method::GET,
                &conversations_page.meta.next_page_url.unwrap(),
                None,
            )?;

            results.append(&mut conversations_page.conversations);
        }

        Ok(results)
    }
}
