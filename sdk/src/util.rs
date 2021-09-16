macro_rules! handle_response {
    ($oneshot:expr, $bind:pat => $arm:block) => {
        match $oneshot.await?? {
            $bind => $arm,
            other => unreachable!("response {:?} should be impossible", other),
        }
    };
}

pub(crate) mod string {
    use serde::{de, Deserialize, Deserializer};
    use std::{fmt::Display, str::FromStr};

    pub fn deserialize<'de, T, D>(deserializer: D) -> Result<T, D::Error>
    where
        T: FromStr,
        T::Err: Display,
        D: Deserializer<'de>,
    {
        let s = <&'de str>::deserialize(deserializer)?;
        s.parse().map_err(de::Error::custom)
    }

    pub fn deserialize_opt<'de, T, D>(deserializer: D) -> Result<Option<T>, D::Error>
    where
        T: FromStr,
        T::Err: Display,
        D: Deserializer<'de>,
    {
        Ok(match Option::<&'de str>::deserialize(deserializer)? {
            Some(s) => Some(s.parse().map_err(de::Error::custom)?),
            None => None,
        })
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
        Ok(match Option::<&'de str>::deserialize(deserializer)? {
            Some(s) => {
                use chrono::TimeZone;
                let ts: i64 = s.parse().map_err(de::Error::custom)?;
                let dt = chrono::Utc.timestamp_millis(ts);

                Some(dt)
            }
            None => None,
        })
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
