use napi_derive::napi;

#[napi]
pub mod friends {
    use crate::api::localplayer::PlayerSteamId;
    use napi::bindgen_prelude::{BigInt, Buffer};
    use std::ffi::{CStr, CString};
    use steamworks::SteamId;

    #[napi]
    pub enum AvatarSize {
        Small,
        Medium,
        Large,
    }

    #[napi(object)]
    pub struct Avatar {
        /// Raw RGBA pixel data, 4 bytes per pixel, row-major from the top-left.
        pub data: Buffer,
        pub width: u32,
        pub height: u32,
    }

    /// A user's Steam-wide availability, as shown next to their name in the friends
    /// list. It says nothing about whether they are in this game.
    #[napi]
    pub enum PersonaState {
        Offline,
        Online,
        Busy,
        Away,
        Snooze,
        LookingToTrade,
        LookingToPlay,
    }

    /// Which set of users `getFriends` walks. These are separate lists, not a
    /// hierarchy: `Immediate` is the ordinary friends list, everything else is a
    /// different relationship entirely.
    #[napi]
    pub enum FriendFilter {
        /// The regular friends list. This is what a friend picker wants.
        Immediate,
        Blocked,
        /// Users who have sent the local user a friend request.
        FriendshipRequested,
        /// Users the local user has sent a friend request to.
        RequestingFriendship,
        RequestingInfo,
        Ignored,
        IgnoredFriend,
        /// Members of Steam groups the local user belongs to.
        ClanMember,
        /// Players on the same game server as the local user.
        OnGameServer,
        /// Members of a Steam chat room the local user is in.
        ChatMember,
        All,
    }

    /// Everything Steam will tell you about a user's profile.
    ///
    /// Only valid for users Steam already knows about — friends, members of the same
    /// lobby, players on the same game server. For anyone else the fields come back
    /// empty until `requestUserInformation` resolves.
    #[napi(object)]
    pub struct Persona {
        pub steam_id: PlayerSteamId,
        /// Public profile name.
        pub name: String,
        /// Private nickname the local user gave this player, when there is one. Never
        /// visible to anyone else.
        pub nickname: Option<String>,
        pub state: PersonaState,
        /// Steam community level. Comes back as 0 until Steam has cached it, which
        /// for a stranger means after a `PersonaStateChange`.
        pub level: u32,
    }

    /// What a user is playing right now.
    #[napi(object)]
    pub struct GamePlayed {
        /// The app the user is in. Compare it against your own app id to tell
        /// "playing this game" from "playing something else".
        pub app_id: u32,
        /// The lobby the user is in, when the game published one. Pass it straight to
        /// `matchmaking.joinLobby` to follow a friend in.
        pub lobby_id: Option<BigInt>,
    }

    /// The safe wrapper covers neither Steam levels nor rich presence reads, so those
    /// go through the raw interface pointer. Fetching it is a plain accessor, not an
    /// allocation, so there is nothing to cache.
    fn steam_friends() -> *mut steamworks::sys::ISteamFriends {
        unsafe { steamworks::sys::SteamAPI_SteamFriends_v017() }
    }

    /// Steam returns "" rather than null for an unknown string, and the two mean the
    /// same thing to a caller: nothing to show.
    unsafe fn owned_string(raw: *const std::os::raw::c_char) -> Option<String> {
        if raw.is_null() {
            return None;
        }

        let value = CStr::from_ptr(raw).to_string_lossy().into_owned();
        (!value.is_empty()).then_some(value)
    }

    fn persona_of<Manager>(friend: &steamworks::Friend<Manager>) -> Persona {
        let steam_id = friend.id();

        Persona {
            steam_id: PlayerSteamId::from_steamid(steam_id),
            name: friend.name(),
            nickname: friend.nick_name(),
            state: match friend.state() {
                steamworks::FriendState::Offline => PersonaState::Offline,
                steamworks::FriendState::Online => PersonaState::Online,
                steamworks::FriendState::Busy => PersonaState::Busy,
                steamworks::FriendState::Away => PersonaState::Away,
                steamworks::FriendState::Snooze => PersonaState::Snooze,
                steamworks::FriendState::LookingToTrade => PersonaState::LookingToTrade,
                steamworks::FriendState::LookingToPlay => PersonaState::LookingToPlay,
            },
            level: unsafe {
                steamworks::sys::SteamAPI_ISteamFriends_GetFriendSteamLevel(
                    steam_friends(),
                    steam_id.raw(),
                )
            } as u32,
        }
    }

    /// Reads the cached profile of any user Steam knows about.
    ///
    /// Steam only knows about users the local user shares a "source" with: friends,
    /// members of the same lobby, players on the same game server. For anyone else,
    /// call `requestUserInformation` first and read this once `PersonaStateChange`
    /// fires — until then the name comes back empty and the level as 0.
    ///
    /// {@link https://partner.steamgames.com/doc/api/ISteamFriends#GetFriendPersonaName}
    #[napi]
    pub fn get_persona(steam_id64: BigInt) -> Persona {
        let client = crate::client::get_client();
        persona_of(
            &client
                .friends()
                .get_friend(SteamId::from_raw(steam_id64.get_u64().1)),
        )
    }

    /// Reads the cached profiles of several users in one call.
    ///
    /// Same caching rules as `getPersona`. Batching matters when the caller is across
    /// an IPC boundary: a lobby of eight is one round trip instead of eight.
    #[napi]
    pub fn get_personas(steam_ids64: Vec<BigInt>) -> Vec<Persona> {
        let client = crate::client::get_client();
        let friends = client.friends();

        steam_ids64
            .into_iter()
            .map(|steam_id64| persona_of(&friends.get_friend(SteamId::from_raw(steam_id64.get_u64().1))))
            .collect()
    }

    /// Lists the users in one of the local user's relationship lists, with their
    /// profiles already resolved — Steam always has these cached.
    #[napi]
    pub fn get_friends(filter: FriendFilter) -> Vec<Persona> {
        let client = crate::client::get_client();
        client
            .friends()
            .get_friends(match filter {
                FriendFilter::Immediate => steamworks::FriendFlags::IMMEDIATE,
                FriendFilter::Blocked => steamworks::FriendFlags::BLOCKED,
                FriendFilter::FriendshipRequested => steamworks::FriendFlags::FRIENDSHIP_REQUESTED,
                FriendFilter::RequestingFriendship => steamworks::FriendFlags::REQUESTING_FRIENDSHIP,
                FriendFilter::RequestingInfo => steamworks::FriendFlags::REQUESTING_INFO,
                FriendFilter::Ignored => steamworks::FriendFlags::IGNORED,
                FriendFilter::IgnoredFriend => steamworks::FriendFlags::IGNORED_FRIEND,
                FriendFilter::ClanMember => steamworks::FriendFlags::CLAN_MEMBER,
                FriendFilter::OnGameServer => steamworks::FriendFlags::ON_GAME_SERVER,
                FriendFilter::ChatMember => steamworks::FriendFlags::CHAT_MEMBER,
                FriendFilter::All => steamworks::FriendFlags::ALL,
            })
            .iter()
            .map(persona_of)
            .collect()
    }

    /// Lists the local user's recent teammates — everyone Steam has recorded as
    /// "played with" recently, across games.
    #[napi]
    pub fn get_coplay_friends() -> Vec<Persona> {
        let client = crate::client::get_client();
        client
            .friends()
            .get_coplay_friends()
            .iter()
            .map(persona_of)
            .collect()
    }

    /// Whether the given user is on the local user's friends list.
    #[napi]
    pub fn is_friend(steam_id64: BigInt) -> bool {
        let client = crate::client::get_client();
        client
            .friends()
            .get_friend(SteamId::from_raw(steam_id64.get_u64().1))
            .has_friend(steamworks::FriendFlags::IMMEDIATE)
    }

    /// What the given user is playing, or null when they are not in a game.
    ///
    /// The lobby id it reports is the hook for "join a friend's game": it is set for
    /// any player whose game published a lobby, whether or not they are a friend.
    #[napi]
    pub fn get_game_played(steam_id64: BigInt) -> Option<GamePlayed> {
        let client = crate::client::get_client();
        client
            .friends()
            .get_friend(SteamId::from_raw(steam_id64.get_u64().1))
            .game_played()
            .map(|game| GamePlayed {
                app_id: game.game.app_id().0,
                // Steam reports a zeroed lobby id for a player who is in a game but
                // not in any lobby.
                lobby_id: (game.lobby.raw() != 0).then(|| BigInt::from(game.lobby.raw())),
            })
    }

    /// Reads one rich presence value a user's game published about them.
    ///
    /// Returns null when the key is unset. Rich presence only crosses between players
    /// of the same app, so this is always your own game's data.
    ///
    /// {@link https://partner.steamgames.com/doc/api/ISteamFriends#GetFriendRichPresence}
    #[napi]
    pub fn get_rich_presence(steam_id64: BigInt, key: String) -> Option<String> {
        let Ok(key) = CString::new(key) else {
            return None;
        };

        unsafe {
            owned_string(steamworks::sys::SteamAPI_ISteamFriends_GetFriendRichPresence(
                steam_friends(),
                steam_id64.get_u64().1,
                key.as_ptr(),
            ))
        }
    }

    /// Lists the rich presence keys a user currently has set, so a caller can read
    /// them without knowing the publishing side's key names up front.
    #[napi]
    pub fn get_rich_presence_keys(steam_id64: BigInt) -> Vec<String> {
        let steam_id64 = steam_id64.get_u64().1;
        let friends = steam_friends();

        unsafe {
            let count =
                steamworks::sys::SteamAPI_ISteamFriends_GetFriendRichPresenceKeyCount(friends, steam_id64);

            (0..count)
                .filter_map(|index| {
                    owned_string(
                        steamworks::sys::SteamAPI_ISteamFriends_GetFriendRichPresenceKeyByIndex(
                            friends, steam_id64, index,
                        ),
                    )
                })
                .collect()
        }
    }

    /// Gets the avatar of any user Steam already knows about, in raw RGBA format.
    ///
    /// Steam only knows about users the local user shares a "source" with: friends,
    /// members of the same lobby, players on the same game server, etc. For anyone
    /// else, call `requestUserInformation` first.
    ///
    /// Returns `null` when the avatar is not cached yet. In that case, register a
    /// `PersonaStateChange` callback, wait for it to fire for this steam id, then
    /// call this function again. Do not busy-loop.
    ///
    /// {@link https://partner.steamgames.com/doc/api/ISteamFriends#GetLargeFriendAvatar}
    #[napi]
    pub fn get_avatar(steam_id64: BigInt, size: AvatarSize) -> Option<Avatar> {
        let client = crate::client::get_client();
        let friend = client
            .friends()
            .get_friend(SteamId::from_raw(steam_id64.get_u64().1));

        let (data, dimension) = match size {
            AvatarSize::Small => (friend.small_avatar(), 32),
            AvatarSize::Medium => (friend.medium_avatar(), 64),
            AvatarSize::Large => (friend.large_avatar(), 184),
        };

        data.map(|data| Avatar {
            data: data.into(),
            width: dimension,
            height: dimension,
        })
    }

    /// Asks Steam to cache the persona name and avatar of a user it does not know
    /// about yet.
    ///
    /// @param nameOnly - When true, the avatar is not downloaded. Downloading
    /// avatars is slow and churns the local cache, so pass true if you only need
    /// the name.
    ///
    /// @returns true if the information is being requested, in which case a
    /// `PersonaStateChange` callback will fire once it arrives. Returns false if
    /// Steam already has everything, meaning `getAvatar` can be called right away.
    #[napi]
    pub fn request_user_information(steam_id64: BigInt, name_only: bool) -> bool {
        let client = crate::client::get_client();
        client
            .friends()
            .request_user_information(SteamId::from_raw(steam_id64.get_u64().1), name_only)
    }

    /// Sends a game invite through Steam. The invitee gets a chat notification and,
    /// on accepting, their client launches the game with `connectString` on the
    /// command line — or fires `GameLobbyJoinRequested` if it is already running.
    ///
    /// {@link https://partner.steamgames.com/doc/api/ISteamFriends#InviteUserToGame}
    #[napi]
    pub fn invite_user_to_game(steam_id64: BigInt, connect_string: String) {
        let client = crate::client::get_client();
        client
            .friends()
            .get_friend(SteamId::from_raw(steam_id64.get_u64().1))
            .invite_user_to_game(&connect_string);
    }

    /// Records that the local user played with this player, which is what puts them
    /// on both players' "recently played with" lists. Only works while both are in
    /// the game together, so call it when a match starts, not when it ends.
    #[napi]
    pub fn set_played_with(steam_id64: BigInt) {
        let client = crate::client::get_client();
        client
            .friends()
            .get_friend(SteamId::from_raw(steam_id64.get_u64().1))
            .set_played_with();
    }
}
