use diesel::backend::Backend;
use diesel::deserialize::{self, FromSql};
use diesel::serialize::{self, Output, ToSql};
use diesel::sql_types::Text;
use diesel::sqlite::Sqlite;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Macro to implement Display, FromSql, ToSql for a string-backed enum.
/// Each variant maps to a snake_case string.
macro_rules! text_enum {
    (
        $(#[$meta:meta])*
        pub enum $name:ident {
            $( $(#[$vmeta:meta])* $variant:ident => $str:literal ),+ $(,)?
        }
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, diesel::AsExpression, diesel::FromSqlRow)]
        #[diesel(sql_type = Text)]
        #[serde(rename_all = "snake_case")]
        pub enum $name {
            $( $(#[$vmeta])* $variant ),+
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                match self {
                    $( Self::$variant => write!(f, $str) ),+
                }
            }
        }

        impl ToSql<Text, Sqlite> for $name {
            fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Sqlite>) -> serialize::Result {
                let s = match self {
                    $( Self::$variant => $str ),+
                };
                <str as ToSql<Text, Sqlite>>::to_sql(s, out)
            }
        }

        impl FromSql<Text, Sqlite> for $name {
            fn from_sql(bytes: <Sqlite as Backend>::RawValue<'_>) -> deserialize::Result<Self> {
                let s = <String as FromSql<Text, Sqlite>>::from_sql(bytes)?;
                match s.as_str() {
                    $( $str => Ok(Self::$variant), )+
                    _ => Err(format!("Unknown {} value: {}", stringify!($name), s).into()),
                }
            }
        }
    };
}

text_enum! {
    pub enum Role {
        // Good
        LoyalServant => "loyal_servant",
        Merlin => "merlin",
        Percival => "percival",
        Cleric => "cleric",
        Troublemaker => "troublemaker",
        UntrustworthyServant => "untrustworthy_servant",
        SeniorMessenger => "senior_messenger",
        JuniorMessenger => "junior_messenger",
        GoodSorcerer => "good_sorcerer",
        GoodLancelot => "good_lancelot",
        // Evil
        MinionOfMordred => "minion_of_mordred",
        Assassin => "assassin",
        Morgana => "morgana",
        Mordred => "mordred",
        Oberon => "oberon",
        Trickster => "trickster",
        Brute => "brute",
        Lunatic => "lunatic",
        Revealer => "revealer",
        EvilMessenger => "evil_messenger",
        EvilSorcerer => "evil_sorcerer",
        EvilLancelot => "evil_lancelot",
    }
}

text_enum! {
    pub enum QuestResult {
        Success => "success",
        Fail => "fail",
    }
}

text_enum! {
    pub enum RoundStatus {
        Proposed => "proposed",
        Approved => "approved",
        Rejected => "rejected",
    }
}

text_enum! {
    pub enum Vote {
        Approve => "approve",
        Reject => "reject",
    }
}

text_enum! {
    pub enum CardType {
        Success => "success",
        Fail => "fail",
        Magic => "magic",
        GoodMessage => "good_message",
        EvilMessage => "evil_message",
    }
}

text_enum! {
    pub enum Module {
        LadyOfTheLake => "lady_of_the_lake",
        LancelotSwitching => "lancelot_switching",
        PlotCards => "plot_cards",
    }
}

text_enum! {
    pub enum LancelotSwitchResult {
        Switch => "switch",
        NoSwitch => "no_switch",
    }
}

text_enum! {
    pub enum PlotCardStatus {
        Dealt => "dealt",
        Used => "used",
    }
}

text_enum! {
    pub enum SnipeType {
        Merlin => "merlin",
        Messengers => "messengers",
        UntrustworthyServant => "untrustworthy_servant",
    }
}

text_enum! {
    pub enum ClaimedAffiliation {
        Good => "good",
        Evil => "evil",
    }
}
