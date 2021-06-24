use super::*;

/// The logical comparison to use when comparing the value of the filter key in
/// the lobby metadata against the value provided to compare it against
///
/// [API docs](https://discord.com/developers/docs/game-sdk/lobbies#data-models-lobbysearchcomparison-enum)
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
#[repr(i8)]
pub enum LobbySearchComparison {
    LessThanOrEqual = -2,
    LessThan = -1,
    Equal = 0,
    GreaterThan = 1,
    GreaterThanOrEqual = 2,
    NotEqual = 3,
}

/// The search distance from the current user's region, the [`LobbySearchDistance::Default`]
/// is to search in the current user's region and adjacent regions.
///
/// [API docs](https://discord.com/developers/docs/game-sdk/lobbies#data-models-lobbysearchdistance-enum)
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum LobbySearchDistance {
    /// Within the same region
    Local = 0,
    /// Within the same and adjacent regions
    Default = 1,
    /// Far distances, like US to EU
    Extended = 2,
    /// All regions
    Global = 3,
}

impl Default for LobbySearchDistance {
    fn default() -> Self {
        Self::Default
    }
}

/// Determines how the search value is cast before comparison
///
/// [API docs](https://discord.com/developers/docs/game-sdk/lobbies#data-models-lobbysearchcast-enum)
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum LobbySearchCast {
    String = 1,
    Number = 2,
}

#[derive(Serialize)]
pub struct SearchFilter {
    key: String,
    comparison: LobbySearchComparison,
    cast: LobbySearchCast,
    value: String,
}

#[derive(Serialize)]
pub struct SearchSort {
    key: String,
    cast: LobbySearchCast,
    near_value: String,
}

pub enum SearchKey<'md> {
    /// The user id of the owner of the lobby
    OwnerId,
    /// The maximum capacity of the lobby
    Capacity,
    /// The number of available slots in the lobby
    Slots,
    /// A metadata key name
    Metadata(&'md str),
}

impl<'md> From<&'md str> for SearchKey<'md> {
    fn from(key: &'md str) -> Self {
        Self::Metadata(key)
    }
}

use std::fmt;

impl<'md> fmt::Display for SearchKey<'md> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OwnerId => f.write_str("owner_id"),
            Self::Capacity => f.write_str("capacity"),
            Self::Slots => f.write_str("slots"),
            Self::Metadata(key) => write!(f, "metadata.{}", key),
        }
    }
}

pub enum SearchValue {
    String(String),
    Number(String),
}

impl SearchValue {
    pub fn string(s: impl Into<String>) -> Self {
        Self::String(s.into())
    }

    pub fn number<N>(n: N) -> Self
    where
        N: num_traits::PrimInt + fmt::Display,
    {
        Self::Number(n.to_string())
    }

    pub fn cast(&self) -> LobbySearchCast {
        match self {
            Self::String(_) => LobbySearchCast::String,
            Self::Number(_) => LobbySearchCast::Number,
        }
    }
}

impl From<SearchValue> for String {
    fn from(sv: SearchValue) -> Self {
        match sv {
            SearchValue::String(s) | SearchValue::Number(s) => s,
        }
    }
}

/// A query used to [search](https://discord.com/developers/docs/game-sdk/lobbies#search)
/// for lobbies that match a set of criteria.
///
/// By default, this will find a maximum of `25` lobbies in the same or adjacent
/// regions as the current user.
#[derive(Serialize)]
pub struct SearchQuery {
    filter: Vec<SearchFilter>,
    sort: Vec<SearchSort>,
    limit: u32,
    distance: LobbySearchDistance,
}

impl SearchQuery {
    /// Adds a filter to the query which compares the value of the specified key
    /// with the specified comparison against the specified value.
    ///
    /// [API docs](https://discord.com/developers/docs/game-sdk/lobbies#lobbysearchqueryfilter)
    pub fn add_filter<'md>(
        mut self,
        key: impl Into<SearchKey<'md>>,
        comparison: LobbySearchComparison,
        value: SearchValue,
    ) -> Self {
        self.filter.push(SearchFilter {
            key: key.into().to_string(),
            comparison,
            cast: value.cast(),
            value: value.into(),
        });
        self
    }

    /// Sorts the filtered lobbies based on "near-ness" of the specified key's
    /// value to the specified sort value.
    ///
    /// [API docs](https://discord.com/developers/docs/game-sdk/lobbies#lobbysearchquerysort)
    pub fn add_sort<'md>(mut self, key: impl Into<SearchKey<'md>>, value: SearchValue) -> Self {
        self.sort.push(SearchSort {
            key: key.into().to_string(),
            cast: value.cast(),
            near_value: value.into(),
        });
        self
    }

    /// Sets the maximum number of lobbies that can be returned by the search.
    ///
    /// [API docs](https://discord.com/developers/docs/game-sdk/lobbies#lobbysearchquerylimit)
    pub fn limit(mut self, max_results: Option<std::num::NonZeroU32>) -> Self {
        if let Some(mr) = max_results {
            self.limit = mr.get();
        }
        self
    }

    /// Filters lobby results to within certain regions relative to the user's location.
    ///
    /// [API docs](https://discord.com/developers/docs/game-sdk/lobbies#lobbysearchquerydistance)
    pub fn distance(mut self, distance: LobbySearchDistance) -> Self {
        self.distance = distance;
        self
    }
}

impl Default for SearchQuery {
    fn default() -> Self {
        Self {
            filter: Vec::new(),
            sort: Vec::new(),
            limit: 25,
            distance: Default::default(),
        }
    }
}

impl crate::Discord {
    /// Searches available lobbies based on the search criteria
    ///
    /// [API docs](https://discord.com/developers/docs/game-sdk/lobbies#search)
    pub async fn search_lobbies(&self, query: SearchQuery) -> Result<Vec<Lobby>, Error> {
        let rx = self.send_rpc(CommandKind::SearchLobbies, query)?;

        handle_response!(rx, Command::SearchLobbies(lobbies) => {
            Ok(lobbies)
        })
    }
}
