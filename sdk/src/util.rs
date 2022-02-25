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

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<time::OffsetDateTime>, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(match Option::<&'de str>::deserialize(deserializer)? {
            Some(s) => {
                let ts: i64 = s.parse().map_err(de::Error::custom)?;
                Some(
                    time::OffsetDateTime::from_unix_timestamp_nanos(ts as i128 * 1000000)
                        .map_err(de::Error::custom)?,
                )
            }
            None => None,
        })
    }

    #[allow(dead_code)]
    pub fn serialize<S>(
        value: &Option<time::OffsetDateTime>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match value {
            Some(dt) => serializer.collect_str(&(dt.unix_timestamp_nanos() / 1000000)),
            None => serializer.serialize_none(),
        }
    }
}

#[cfg(test)]
#[inline]
pub(crate) fn timestamp(ts: i64) -> time::OffsetDateTime {
    time::OffsetDateTime::from_unix_timestamp_nanos(ts as i128 * 1000000).unwrap()
}
