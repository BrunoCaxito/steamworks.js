use napi_derive::napi;

#[napi]
pub mod matchmaking {
    use crate::api::localplayer::PlayerSteamId;
    use napi::bindgen_prelude::{BigInt, Buffer, Error};
    use std::collections::HashMap;
    use std::ffi::CString;
    use steamworks::{
        ComparisonFilter, DistanceFilter, LobbyId, LobbyKey, NearFilter, NumberFilter, StringFilter,
        StringFilterKind,
    };
    use tokio::sync::oneshot;

    /// The largest lobby chat message Steam will hand back in one entry.
    const MAX_CHAT_ENTRY_SIZE: usize = 4096;

    #[napi]
    pub enum LobbyType {
        Private,
        FriendsOnly,
        Public,
        Invisible,
    }

    /// How a numeric lobby filter compares a lobby's value against the requested one.
    #[napi]
    pub enum LobbyComparison {
        Equal,
        NotEqual,
        GreaterThan,
        GreaterThanOrEqual,
        LessThan,
        LessThanOrEqual,
    }

    /// How far afield the lobby search reaches. Anything wider than `Default` trades
    /// latency for population.
    #[napi]
    pub enum LobbyDistance {
        Close,
        Default,
        Far,
        Worldwide,
    }

    /// Matches lobbies whose data at `key` equals — or, with `exclude`, differs from —
    /// `value`. The lobby has to have published that key with `setData`.
    #[napi(object)]
    pub struct LobbyStringFilter {
        pub key: String,
        pub value: String,
        /// Defaults to false, meaning "must equal".
        pub exclude: Option<bool>,
    }

    /// Matches lobbies whose data at `key` compares against `value` as requested.
    #[napi(object)]
    pub struct LobbyNumberFilter {
        pub key: String,
        pub value: i32,
        pub comparison: LobbyComparison,
    }

    /// Does not filter anything out — sorts the results by how close their value at
    /// `key` is to `value`. Use it for skill-based ordering.
    #[napi(object)]
    pub struct LobbyNearFilter {
        pub key: String,
        pub value: i32,
    }

    /// Narrows a lobby search. Without one, Steam returns whatever it likes from the
    /// whole world, which is rarely what a lobby browser wants.
    ///
    /// Every filter applies to the *next* search only — they are consumed by the call.
    #[napi(object)]
    pub struct LobbyFilter {
        pub string: Option<Vec<LobbyStringFilter>>,
        pub number: Option<Vec<LobbyNumberFilter>>,
        pub near_value: Option<Vec<LobbyNearFilter>>,
        /// Only lobbies with at least this many free slots.
        pub open_slots: Option<u32>,
        pub distance: Option<LobbyDistance>,
        /// Caps how many lobbies come back.
        pub count: Option<u32>,
    }

    #[napi]
    pub struct Lobby {
        pub id: BigInt,
        lobby_id: LobbyId,
    }

    #[napi]
    impl Lobby {
        #[napi]
        pub async fn join(&self) -> Result<Lobby, Error> {
            join_lobby(self.id.clone()).await
        }

        #[napi]
        pub fn leave(&self) {
            let client = crate::client::get_client();
            client.matchmaking().leave_lobby(self.lobby_id);
        }

        #[napi]
        pub fn open_invite_dialog(&self) {
            let client = crate::client::get_client();
            client.friends().activate_invite_dialog(self.lobby_id);
        }

        #[napi]
        pub fn get_member_count(&self) -> usize {
            let client = crate::client::get_client();
            client.matchmaking().lobby_member_count(self.lobby_id)
        }

        #[napi]
        pub fn get_member_limit(&self) -> Option<usize> {
            let client = crate::client::get_client();
            client.matchmaking().lobby_member_limit(self.lobby_id)
        }

        #[napi]
        pub fn get_members(&self) -> Vec<PlayerSteamId> {
            let client = crate::client::get_client();
            client
                .matchmaking()
                .lobby_members(self.lobby_id)
                .into_iter()
                .map(PlayerSteamId::from_steamid)
                .collect()
        }

        #[napi]
        pub fn get_owner(&self) -> PlayerSteamId {
            let client = crate::client::get_client();
            PlayerSteamId::from_steamid(client.matchmaking().lobby_owner(self.lobby_id))
        }

        #[napi]
        pub fn set_joinable(&self, joinable: bool) -> bool {
            let client = crate::client::get_client();
            client
                .matchmaking()
                .set_lobby_joinable(self.lobby_id, joinable)
        }

        /// Changes who can find and join the lobby after it was created. Owner only.
        ///
        /// {@link https://partner.steamgames.com/doc/api/ISteamMatchmaking#SetLobbyType}
        #[napi]
        pub fn set_type(&self, lobby_type: LobbyType) -> bool {
            unsafe {
                steamworks::sys::SteamAPI_ISteamMatchmaking_SetLobbyType(
                    steam_matchmaking(),
                    self.lobby_id.raw(),
                    match lobby_type {
                        LobbyType::Private => steamworks::sys::ELobbyType::k_ELobbyTypePrivate,
                        LobbyType::FriendsOnly => {
                            steamworks::sys::ELobbyType::k_ELobbyTypeFriendsOnly
                        }
                        LobbyType::Public => steamworks::sys::ELobbyType::k_ELobbyTypePublic,
                        LobbyType::Invisible => steamworks::sys::ELobbyType::k_ELobbyTypeInvisible,
                    },
                )
            }
        }

        /// Hands ownership to another member — the host migration path when the owner
        /// leaves on purpose. Owner only, and the new owner must already be in the
        /// lobby. When an owner drops without calling this, Steam picks a successor
        /// itself and everyone gets a `LobbyDataUpdate`.
        #[napi]
        pub fn set_owner(&self, steam_id64: BigInt) -> bool {
            unsafe {
                steamworks::sys::SteamAPI_ISteamMatchmaking_SetLobbyOwner(
                    steam_matchmaking(),
                    self.lobby_id.raw(),
                    steam_id64.get_u64().1,
                )
            }
        }

        /// Invites a user straight to this lobby, no overlay involved. They get a Steam
        /// notification and, on accepting, a `GameLobbyJoinRequested` callback fires in
        /// their client. Use `openInviteDialog` instead when the player should pick the
        /// invitee themselves.
        #[napi]
        pub fn invite_user(&self, steam_id64: BigInt) -> bool {
            unsafe {
                steamworks::sys::SteamAPI_ISteamMatchmaking_InviteUserToLobby(
                    steam_matchmaking(),
                    self.lobby_id.raw(),
                    steam_id64.get_u64().1,
                )
            }
        }

        #[napi]
        pub fn get_data(&self, key: String) -> Option<String> {
            let client = crate::client::get_client();
            client
                .matchmaking()
                .lobby_data(self.lobby_id, &key)
                .map(|s| s.to_string())
        }

        #[napi]
        pub fn set_data(&self, key: String, value: String) -> bool {
            let client = crate::client::get_client();
            client
                .matchmaking()
                .set_lobby_data(self.lobby_id, &key, &value)
        }

        #[napi]
        pub fn delete_data(&self, key: String) -> bool {
            let client = crate::client::get_client();
            client.matchmaking().delete_lobby_data(self.lobby_id, &key)
        }

        /// Get an object containing all the lobby data
        #[napi]
        pub fn get_full_data(&self) -> HashMap<String, String> {
            let client = crate::client::get_client();

            let mut data = HashMap::new();

            let count = client.matchmaking().lobby_data_count(self.lobby_id);
            for i in 0..count {
                let maybe_lobby_data = client.matchmaking().lobby_data_by_index(self.lobby_id, i);

                if let Some((key, value)) = maybe_lobby_data {
                    data.insert(key, value);
                }
            }

            data
        }

        /// Merge current lobby data with provided data in a single batch
        /// @returns true if all data was set successfully
        #[napi]
        pub fn merge_full_data(&self, data: HashMap<String, String>) -> bool {
            let matchmaking = crate::client::get_client().matchmaking();
            data.iter()
                .map(|(key, value)| matchmaking.set_lobby_data(self.lobby_id, key, value))
                .all(|x| x)
        }

        /// Publishes a value about the local user to the rest of the lobby.
        ///
        /// This is the counterpart to `setData` that every member can call — `setData`
        /// is the owner's alone. It is how a player announces things about themselves:
        /// ready state, chosen side, loaded progress. Everyone else sees a
        /// `LobbyDataUpdate` whose `member` is this player, then reads it back with
        /// `getMemberData`.
        ///
        /// {@link https://partner.steamgames.com/doc/api/ISteamMatchmaking#SetLobbyMemberData}
        #[napi]
        pub fn set_member_data(&self, key: String, value: String) {
            let (Ok(key), Ok(value)) = (CString::new(key), CString::new(value)) else {
                return;
            };

            unsafe {
                steamworks::sys::SteamAPI_ISteamMatchmaking_SetLobbyMemberData(
                    steam_matchmaking(),
                    self.lobby_id.raw(),
                    key.as_ptr(),
                    value.as_ptr(),
                );
            }
        }

        /// Reads a value another member published about themselves with
        /// `setMemberData`. Returns null when that member never set the key.
        ///
        /// There is no way to enumerate a member's keys — Steam only answers by name —
        /// so both sides have to agree on the key names up front.
        #[napi]
        pub fn get_member_data(&self, steam_id64: BigInt, key: String) -> Option<String> {
            let Ok(key) = CString::new(key) else {
                return None;
            };

            unsafe {
                let value = steamworks::sys::SteamAPI_ISteamMatchmaking_GetLobbyMemberData(
                    steam_matchmaking(),
                    self.lobby_id.raw(),
                    steam_id64.get_u64().1,
                    key.as_ptr(),
                );

                if value.is_null() {
                    return None;
                }

                let value = std::ffi::CStr::from_ptr(value).to_string_lossy().into_owned();
                (!value.is_empty()).then_some(value)
            }
        }

        /// Broadcasts a message to every member, routed through Steam's back-end.
        ///
        /// Slow and bandwidth-limited compared to P2P, but it needs no session setup
        /// and it reaches members who have not connected to anyone yet — which makes it
        /// the right channel for lobby coordination and the wrong one for game traffic.
        ///
        /// Recipients get a `LobbyChatMsg` callback carrying a `chatId`, which they
        /// pass to `getChatEntry` to read the bytes.
        #[napi]
        pub fn send_chat_message(&self, data: Buffer) -> bool {
            let client = crate::client::get_client();
            client
                .matchmaking()
                .send_lobby_chat_message(self.lobby_id, &data)
                .is_ok()
        }

        /// Reads the message a `LobbyChatMsg` callback announced. Only valid inside the
        /// handler for that callback — Steam recycles the entry right after.
        #[napi]
        pub fn get_chat_entry(&self, chat_id: i32) -> Buffer {
            let client = crate::client::get_client();
            let mut buffer = vec![0; MAX_CHAT_ENTRY_SIZE];

            let read = client
                .matchmaking()
                .get_lobby_chat_entry(self.lobby_id, chat_id, &mut buffer)
                .len();

            buffer.truncate(read);
            buffer.into()
        }
    }

    /// The safe wrapper covers neither member data nor lobby ownership changes, so
    /// those go through the raw interface pointer. Fetching it is a plain accessor,
    /// not an allocation, so there is nothing to cache.
    fn steam_matchmaking() -> *mut steamworks::sys::ISteamMatchmaking {
        unsafe { steamworks::sys::SteamAPI_SteamMatchmaking_v009() }
    }

    /// Steam caps how long a filter key may be, and an over-long one is dropped
    /// silently rather than rejected — which would show up as a search that quietly
    /// ignores the filter. Fail the search instead.
    fn lobby_key(key: &str) -> Result<LobbyKey<'_>, Error> {
        LobbyKey::try_new(key).map_err(|error| Error::from_reason(format!("{error}: {key}")))
    }

    #[napi]
    pub async fn create_lobby(lobby_type: LobbyType, max_members: u32) -> Result<Lobby, Error> {
        let client = crate::client::get_client();

        let (tx, rx) = oneshot::channel();

        client.matchmaking().create_lobby(
            match lobby_type {
                LobbyType::Private => steamworks::LobbyType::Private,
                LobbyType::FriendsOnly => steamworks::LobbyType::FriendsOnly,
                LobbyType::Public => steamworks::LobbyType::Public,
                LobbyType::Invisible => steamworks::LobbyType::Invisible,
            },
            max_members,
            |result| {
                tx.send(result).unwrap();
            },
        );

        rx.await
            .unwrap()
            .map(|lobby_id| Lobby {
                id: BigInt::from(lobby_id.raw()),
                lobby_id,
            })
            .map_err(|e| Error::from_reason(e.to_string()))
    }

    #[napi]
    pub async fn join_lobby(lobby_id: BigInt) -> Result<Lobby, Error> {
        let client = crate::client::get_client();

        let (tx, rx) = oneshot::channel();

        client.matchmaking().join_lobby(
            steamworks::LobbyId::from_raw(lobby_id.get_u64().1),
            |result| {
                tx.send(result).unwrap();
            },
        );

        rx.await
            .unwrap()
            .map(|lobby_id| Lobby {
                id: BigInt::from(lobby_id.raw()),
                lobby_id,
            })
            .map_err(|_| Error::from_reason("Failed to join lobby".to_string()))
    }

    /// Searches for joinable lobbies.
    ///
    /// Without a filter Steam decides what to return from the whole world, so a lobby
    /// browser should almost always pass one — at minimum a `count` and a string
    /// filter on a key the game publishes, so other games' lobbies never show up.
    #[napi]
    pub async fn get_lobbies(filter: Option<LobbyFilter>) -> Result<Vec<Lobby>, Error> {
        let client = crate::client::get_client();

        if let Some(filter) = filter {
            let matchmaking = client.matchmaking();

            // Keys are borrowed by the filter types, so each one has to outlive its
            // own call — hence applying them one at a time instead of building a
            // single LobbyListFilter.
            for string in filter.string.into_iter().flatten() {
                matchmaking.add_request_lobby_list_string_filter(StringFilter(
                    lobby_key(&string.key)?,
                    &string.value,
                    match string.exclude.unwrap_or(false) {
                        true => StringFilterKind::Exclude,
                        false => StringFilterKind::Include,
                    },
                ));
            }

            for number in filter.number.into_iter().flatten() {
                matchmaking.add_request_lobby_list_numerical_filter(NumberFilter(
                    lobby_key(&number.key)?,
                    number.value,
                    match number.comparison {
                        LobbyComparison::Equal => ComparisonFilter::Equal,
                        LobbyComparison::NotEqual => ComparisonFilter::NotEqual,
                        LobbyComparison::GreaterThan => ComparisonFilter::GreaterThan,
                        LobbyComparison::GreaterThanOrEqual => ComparisonFilter::GreaterThanEqualTo,
                        LobbyComparison::LessThan => ComparisonFilter::LessThan,
                        LobbyComparison::LessThanOrEqual => ComparisonFilter::LessThanEqualTo,
                    },
                ));
            }

            for near in filter.near_value.into_iter().flatten() {
                matchmaking
                    .add_request_lobby_list_near_value_filter(NearFilter(lobby_key(&near.key)?, near.value));
            }

            if let Some(open_slots) = filter.open_slots {
                matchmaking.set_request_lobby_list_slots_available_filter(open_slots as u8);
            }

            if let Some(distance) = filter.distance {
                matchmaking.set_request_lobby_list_distance_filter(match distance {
                    LobbyDistance::Close => DistanceFilter::Close,
                    LobbyDistance::Default => DistanceFilter::Default,
                    LobbyDistance::Far => DistanceFilter::Far,
                    LobbyDistance::Worldwide => DistanceFilter::Worldwide,
                });
            }

            if let Some(count) = filter.count {
                matchmaking.set_request_lobby_list_result_count_filter(count as u64);
            }
        }

        let (tx, rx) = oneshot::channel();

        client.matchmaking().request_lobby_list(|lobbies| {
            tx.send(lobbies).unwrap();
        });

        rx.await
            .unwrap()
            .map(|lobbies| {
                lobbies
                    .iter()
                    .map(|lobby_id| Lobby {
                        id: BigInt::from(lobby_id.raw()),
                        lobby_id: *lobby_id,
                    })
                    .collect()
            })
            .map_err(|e| Error::from_reason(e.to_string()))
    }
}
