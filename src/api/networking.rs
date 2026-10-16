use napi_derive::napi;

#[napi]
pub mod networking {
    use std::collections::HashMap;
    use std::sync::Mutex;

    use napi::{
        bindgen_prelude::{BigInt, Buffer},
        threadsafe_function::{ErrorStrategy, ThreadsafeFunction, ThreadsafeFunctionCallMode},
        Error, JsFunction,
    };
    use steamworks::networking_messages::SessionRequest;
    use steamworks::networking_types::{NetworkingIdentity, SendFlags};
    use steamworks::{sys, ClientManager, SteamId};

    use crate::api::localplayer::PlayerSteamId;

    lazy_static! {
        /// Session requests waiting for a JS-side verdict. The napi callback fires
        /// asynchronously, so each request has to survive here until
        /// `acceptSessionRequest` or `rejectSessionRequest` consumes it.
        /// Dropping a request rejects it.
        static ref SESSION_REQUESTS: Mutex<HashMap<u64, SessionRequest<ClientManager>>> =
            Mutex::new(HashMap::new());
    }

    #[napi(object)]
    pub struct NetworkingMessage {
        pub data: Buffer,
        pub steam_id: PlayerSteamId,
    }

    /// The delivery guarantee for a message
    #[napi]
    pub enum SendType {
        /// May be dropped or arrive out of order.
        /// A single datagram on the wire: can't be larger than ~1200 bytes.
        Unreliable,
        /// Delivered exactly once and in order, fragmented and
        /// reassembled as needed. Can't be larger than 512 KB.
        Reliable,
    }

    /// Send a message to another user. A session with them is opened implicitly
    /// if needed; delivery starts once they accept it. A session that failed is
    /// reopened by the next message, and whatever it had not delivered is lost.
    #[napi]
    pub fn send_message_to_user(
        steam_id64: BigInt,
        send_type: SendType,
        data: Buffer,
        channel: u32,
    ) -> Result<(), Error> {
        let delivery = match send_type {
            SendType::Unreliable => SendFlags::UNRELIABLE,
            SendType::Reliable => SendFlags::RELIABLE,
        };

        let client = crate::client::get_client();
        client
            .networking_messages()
            .send_message_to_user(
                NetworkingIdentity::new_steam_id(SteamId::from_raw(steam_id64.get_u64().1)),
                // Without this, once a session fails every later send to that user
                // fails too, until someone closes the session by hand.
                delivery | SendFlags::AUTO_RESTART_BROKEN_SESSION,
                &data,
                channel,
            )
            .map_err(|error| Error::from_reason(error.to_string()))
    }

    /// Read up to `batch_size` messages pending on `channel`.
    /// Returns fewer, possibly none, when the queue runs out.
    #[napi]
    pub fn receive_messages_on_channel(channel: u32, batch_size: u32) -> Vec<NetworkingMessage> {
        let client = crate::client::get_client();
        client
            .networking_messages()
            .receive_messages_on_channel(channel, batch_size as usize)
            .iter()
            .filter_map(|message| {
                // Only steam-id peers can reach this API; anything else is dropped.
                message
                    .identity_peer()
                    .steam_id()
                    .map(|steam_id| NetworkingMessage {
                        data: message.data().to_vec().into(),
                        steam_id: PlayerSteamId::from_steamid(steam_id),
                    })
            })
            .collect()
    }

    /// Register the handler deciding who may open a session. Steam holds the
    /// peer's first messages until the request is accepted, so the handler must
    /// answer with `acceptSessionRequest` or `rejectSessionRequest`.
    ///
    /// Calling this again replaces the previous handler.
    #[napi(ts_args_type = "handler: (steamId64: bigint) => void")]
    pub fn on_session_request(handler: JsFunction) {
        let threadsafe_handler: ThreadsafeFunction<u64, ErrorStrategy::Fatal> = handler
            .create_threadsafe_function(0, |ctx| Ok(vec![ctx.value]))
            .unwrap();

        let client = crate::client::get_client();
        client
            .networking_messages()
            .session_request_callback(move |request| {
                let Some(steam_id) = request.remote().steam_id() else {
                    // Not a steam-id peer; dropping the request rejects it.
                    return;
                };

                // Replacing a stale request from the same peer drops, and
                // thereby rejects, the old one.
                SESSION_REQUESTS
                    .lock()
                    .unwrap()
                    .insert(steam_id.raw(), request);

                threadsafe_handler.call(steam_id.raw(), ThreadsafeFunctionCallMode::Blocking);
            });
    }

    /// Accept a session request announced by `onSessionRequest`.
    /// Returns false when no request from that peer is waiting, or when Steam
    /// no longer has a connection from them to accept.
    #[napi]
    pub fn accept_session_request(steam_id64: BigInt) -> bool {
        let steam_id64 = steam_id64.get_u64().1;

        match SESSION_REQUESTS.lock().unwrap().remove(&steam_id64) {
            Some(request) => request.accept(),
            None => false,
        }
    }

    /// Reject a session request announced by `onSessionRequest`.
    #[napi]
    pub fn reject_session_request(steam_id64: BigInt) {
        if let Some(request) = SESSION_REQUESTS
            .lock()
            .unwrap()
            .remove(&steam_id64.get_u64().1)
        {
            request.reject();
        }
    }

    /// Tears down the session with a peer and drops anything still queued for
    /// them. Every accepted session needs this when the peer leaves — Steam keeps
    /// the session, and its buffers, alive until someone closes it.
    #[napi]
    pub fn close_session_with_user(steam_id64: BigInt) -> bool {
        let steam_id64 = steam_id64.get_u64().1;

        // A request never answered would otherwise pin its session forever.
        SESSION_REQUESTS.lock().unwrap().remove(&steam_id64);

        // The safe wrapper only closes a session while rejecting a request, so an
        // established session has to be closed through the raw binding.
        unsafe {
            sys::SteamAPI_ISteamNetworkingMessages_CloseSessionWithUser(
                sys::SteamAPI_SteamNetworkingMessages_SteamAPI_v002(),
                &steam_identity(steam_id64),
            )
        }
    }

    /// The current ping to a peer in milliseconds, or null when there is no live
    /// session with them.
    ///
    /// Sends nothing. A session runs on a connection that already times its own
    /// round trip — it has to, to keep itself alive and notice a timeout — so
    /// this only reads the number that connection is already keeping.
    #[napi]
    pub fn get_session_ping(steam_id64: BigInt) -> Option<i32> {
        let steam_id64 = steam_id64.get_u64().1;

        unsafe {
            // Both out-parameters are handed a struct even though only the ping
            // is read: two stack structs cost nothing, and the flat API's rules
            // about which of them may be null are not worth depending on.
            let mut info: sys::SteamNetConnectionInfo_t = std::mem::zeroed();
            let mut status: sys::SteamNetConnectionRealTimeStatus_t = std::mem::zeroed();

            let state = sys::SteamAPI_ISteamNetworkingMessages_GetSessionConnectionInfo(
                sys::SteamAPI_SteamNetworkingMessages_SteamAPI_v002(),
                &steam_identity(steam_id64),
                &mut info,
                &mut status,
            );

            // Only a connected session has a round trip to report. Every other
            // state — no session, still connecting, broken — has no number, and
            // reporting the 0 left in the struct would read as a perfect link.
            match state {
                sys::ESteamNetworkingConnectionState::k_ESteamNetworkingConnectionState_Connected => {
                    Some(status.m_nPing)
                }
                _ => None,
            }
        }
    }

    /// A player's identity in the form the raw networking calls take it.
    fn steam_identity(steam_id64: u64) -> sys::SteamNetworkingIdentity {
        unsafe {
            let mut identity = sys::SteamNetworkingIdentity {
                m_eType: sys::ESteamNetworkingIdentityType::k_ESteamNetworkingIdentityType_Invalid,
                m_cbSize: 0,
                __bindgen_anon_1: sys::SteamNetworkingIdentity__bindgen_ty_2 { m_steamID64: 0 },
            };
            sys::SteamAPI_SteamNetworkingIdentity_Clear(&mut identity);
            sys::SteamAPI_SteamNetworkingIdentity_SetSteamID64(&mut identity, steam_id64);
            identity
        }
    }
}
