use napi_derive::napi;

#[napi]
pub mod friends {
    use napi::bindgen_prelude::{BigInt, Buffer};
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
}
