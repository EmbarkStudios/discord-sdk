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

pub(crate) mod datetime_opt {
    use serde::{de, Deserialize, Deserializer, Serializer};

    pub fn deserialize<'de, D>(
        deserializer: D,
    ) -> Result<Option<chrono::DateTime<chrono::Utc>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        match Option::<&'de str>::deserialize(deserializer)? {
            Some(s) => {
                use chrono::TimeZone;
                let ts: i64 = s.parse().map_err(de::Error::custom)?;
                let dt = chrono::Utc.timestamp_millis(ts);

                Ok(Some(dt))
            }
            None => Ok(None),
        }
    }

    #[allow(dead_code)]
    pub fn serialize<S>(
        value: &Option<chrono::DateTime<chrono::Utc>>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match value {
            Some(dt) => serializer.collect_str(&(dt.timestamp_millis())),
            None => serializer.serialize_none(),
        }
    }
}

#[cfg(test)]
#[inline]
pub(crate) fn timestamp(ts: i64) -> chrono::DateTime<chrono::Utc> {
    use chrono::TimeZone;
    chrono::Utc.timestamp_millis(ts)
}
