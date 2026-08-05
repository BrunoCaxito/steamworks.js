import client = require('./client')

export const enum ChatMemberStateChange {
    /** This user has joined or is joining the lobby. */
    Entered,
    /** This user has left or is leaving the lobby. */
    Left,
    /** User disconnected without leaving the lobby first. */
    Disconnected,
    /** The user has been kicked. */
    Kicked,
    /** The user has been kicked and banned. */
    Banned,
}

export const enum ChatEntryType {
    Invalid = 'Invalid',
    ChatMsg = 'ChatMsg',
    Typing = 'Typing',
    InviteGame = 'InviteGame',
    Emote = 'Emote',
    LeftConversation = 'LeftConversation',
    Entered = 'Entered',
    WasKicked = 'WasKicked',
    WasBanned = 'WasBanned',
    Disconnected = 'Disconnected',
    HistoricalChat = 'HistoricalChat',
    LinkBlocked = 'LinkBlocked',
}

export interface CallbackReturns {
    [client.callback.SteamCallback.PersonaStateChange]: {
        steam_id: bigint
        flags: { bits: number }
    }
    [client.callback.SteamCallback.SteamServersConnected]: {}
    [client.callback.SteamCallback.SteamServersDisconnected]: {
        reason: number
    }
    [client.callback.SteamCallback.SteamServerConnectFailure]: {
        reason: number
        still_retrying: boolean
    }
    [client.callback.SteamCallback.LobbyDataUpdate]: {
        lobby: bigint
        member: bigint
        success: boolean
    }
    [client.callback.SteamCallback.LobbyChatUpdate]: {
        lobby: bigint
        user_changed: bigint
        making_change: bigint
        member_state_change: ChatMemberStateChange
    }
    /** A member sent a message through `lobby.sendChatMessage`. Read it with `lobby.getChatEntry(chat_id)` from inside the handler. */
    [client.callback.SteamCallback.LobbyChatMsg]: {
        lobby: bigint
        user: bigint
        chat_entry_type: ChatEntryType
        chat_id: number
    }
    [client.callback.SteamCallback.P2PSessionRequest]: {
        remote: bigint
    }
    [client.callback.SteamCallback.P2PSessionConnectFail]: {
        remote: bigint
        error: number
    }
    [client.callback.SteamCallback.GameLobbyJoinRequested]: {
        lobby_steam_id: bigint
        friend_steam_id: bigint
    }
    /** The Steam overlay opened or closed on top of the game. Pause on `active`. */
    [client.callback.SteamCallback.GameOverlayActivated]: {
        active: boolean
    }
    [client.callback.SteamCallback.MicroTxnAuthorizationResponse]: {
        app_id: number
        order_id: number | bigint
        authorized: boolean
    }
}
