macro_rules! handle_response {
    ($oneshot:expr, $bind:pat => $arm:block) => {
        match $oneshot.await?? {
            $bind => $arm,
            other => unreachable!("response {:?} should be impossible", other),
        }
    };
}

pub(crate) mod string {
    use std::fmt::Display;
    use std::str::FromStr;

    use serde::{de, Deserialize, Deserializer};

    // pub fn serialize<T, S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
    // where
    //     T: Display,
    //     S: Serializer,
    // {
    //     serializer.collect_str(value)
    // }

    pub fn deserialize<'de, T, D>(deserializer: D) -> Result<T, D::Error>
    where
        T: FromStr,
        T::Err: Display,
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer)?
            .parse()
            .map_err(de::Error::custom)
    }

    // pub fn serialize_opt<T, S>(value: &Option<T>, serializer: S) -> Result<S::Ok, S::Error>
    // where
    //     T: Display,
    //     S: Serializer,
    // {
    //     serializer.collect_str(value.as_ref().unwrap())
    // }

    pub fn deserialize_opt<'de, T, D>(deserializer: D) -> Result<Option<T>, D::Error>
    where
        T: FromStr,
        T::Err: Display,
        D: Deserializer<'de>,
    {
        match Option::<String>::deserialize(deserializer)? {
            Some(s) => Ok(Some(s.parse().map_err(de::Error::custom)?)),
            None => Ok(None),
        }
    }
}

pub(crate) mod timestamp {
    use serde::{de, Deserialize, Deserializer};

    pub fn deserialize_opt<'de, D>(
        deserializer: D,
    ) -> Result<Option<chrono::DateTime<chrono::Utc>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        match Option::<&'de str>::deserialize(deserializer)? {
            Some(s) => {
                use chrono::TimeZone;
                let ts = s.parse().map_err(de::Error::custom)?;
                let dt = chrono::Utc.timestamp(ts, 0);

                Ok(Some(dt))
            }
            None => Ok(None),
        }
    }
}
