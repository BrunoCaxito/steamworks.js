export declare function init(appId?: number | undefined | null): void
export declare function restartAppIfNecessary(appId: number): boolean
export declare function runCallbacks(): void
export interface PlayerSteamId {
  steamId64: bigint
  steamId32: string
  accountId: number
}
export declare namespace achievement {
  export function activate(achievement: string): boolean
  export function isActivated(achievement: string): boolean
  export function clear(achievement: string): boolean
  export function names(): Array<string>
}
export declare namespace apps {
  export function isSubscribedApp(appId: number): boolean
  export function isAppInstalled(appId: number): boolean
  export function isDlcInstalled(appId: number): boolean
  export function isSubscribedFromFreeWeekend(): boolean
  export function isVacBanned(): boolean
  export function isCybercafe(): boolean
  export function isLowViolence(): boolean
  export function isSubscribed(): boolean
  export function appBuildId(): number
  export function appInstallDir(appId: number): string
  export function appOwner(): PlayerSteamId
  export function availableGameLanguages(): Array<string>
  export function currentGameLanguage(): string
  export function currentBetaName(): string | null
}
export declare namespace auth {
  /**
   * @param steamId64 - The user steam id or game server steam id. Use as NetworkIdentity of the remote system that will authenticate the ticket. If it is peer-to-peer then the user steam ID. If it is a game server, then the game server steam ID may be used if it was obtained from a trusted 3rd party
   * @param timeoutSeconds - The number of seconds to wait for the ticket to be validated. Default value is 10 seconds.
   */
  export function getSessionTicketWithSteamId(steamId64: bigint, timeoutSeconds?: number | undefined | null): Promise<Ticket>
  /**
   * @param ip - The string of IPv4 or IPv6 address. Use as NetworkIdentity of the remote system that will authenticate the ticket.
   * @param timeoutSeconds - The number of seconds to wait for the ticket to be validated. Default value is 10 seconds.
   */
  export function getSessionTicketWithIp(ip: string, timeoutSeconds?: number | undefined | null): Promise<Ticket>
  export function getAuthTicketForWebApi(identity: string, timeoutSeconds?: number | undefined | null): Promise<Ticket>
  export class Ticket {
    cancel(): void
    getBytes(): Buffer
  }
}
export declare namespace callback {
  export const enum SteamCallback {
    PersonaStateChange = 0,
    SteamServersConnected = 1,
    SteamServersDisconnected = 2,
    SteamServerConnectFailure = 3,
    LobbyDataUpdate = 4,
    LobbyChatUpdate = 5,
    LobbyChatMsg = 6,
    P2PSessionRequest = 7,
    P2PSessionConnectFail = 8,
    GameLobbyJoinRequested = 9,
    GameOverlayActivated = 10,
    MicroTxnAuthorizationResponse = 11
  }
  export function register<C extends keyof import('./callbacks').CallbackReturns>(steamCallback: C, handler: (value: import('./callbacks').CallbackReturns[C]) => void): Handle
  export class Handle {
    disconnect(): void
  }
}
export declare namespace cloud {
  export function isEnabledForAccount(): boolean
  export function isEnabledForApp(): boolean
  export function setEnabledForApp(enabled: boolean): void
  export function readFile(name: string): string
  export function writeFile(name: string, content: string): boolean
  export function deleteFile(name: string): boolean
  export function fileExists(name: string): boolean
  export function listFiles(): Array<FileInfo>
  export class FileInfo {
    name: string
    size: bigint
  }
}
export declare namespace friends {
  export const enum AvatarSize {
    Small = 0,
    Medium = 1,
    Large = 2
  }
  export interface Avatar {
    /** Raw RGBA pixel data, 4 bytes per pixel, row-major from the top-left. */
    data: Buffer
    width: number
    height: number
  }
  /**
   * A user's Steam-wide availability, as shown next to their name in the friends
   * list. It says nothing about whether they are in this game.
   */
  export const enum PersonaState {
    Offline = 0,
    Online = 1,
    Busy = 2,
    Away = 3,
    Snooze = 4,
    LookingToTrade = 5,
    LookingToPlay = 6
  }
  /**
   * Which set of users `getFriends` walks. These are separate lists, not a
   * hierarchy: `Immediate` is the ordinary friends list, everything else is a
   * different relationship entirely.
   */
  export const enum FriendFilter {
    /** The regular friends list. This is what a friend picker wants. */
    Immediate = 0,
    Blocked = 1,
    /** Users who have sent the local user a friend request. */
    FriendshipRequested = 2,
    /** Users the local user has sent a friend request to. */
    RequestingFriendship = 3,
    RequestingInfo = 4,
    Ignored = 5,
    IgnoredFriend = 6,
    /** Members of Steam groups the local user belongs to. */
    ClanMember = 7,
    /** Players on the same game server as the local user. */
    OnGameServer = 8,
    /** Members of a Steam chat room the local user is in. */
    ChatMember = 9,
    All = 10
  }
  /**
   * Everything Steam will tell you about a user's profile.
   *
   * Only valid for users Steam already knows about — friends, members of the same
   * lobby, players on the same game server. For anyone else the fields come back
   * empty until `requestUserInformation` resolves.
   */
  export interface Persona {
    steamId: PlayerSteamId
    /** Public profile name. */
    name: string
    /**
     * Private nickname the local user gave this player, when there is one. Never
     * visible to anyone else.
     */
    nickname?: string
    state: PersonaState
    /**
     * Steam community level. Comes back as 0 until Steam has cached it, which
     * for a stranger means after a `PersonaStateChange`.
     */
    level: number
  }
  /** What a user is playing right now. */
  export interface GamePlayed {
    /**
     * The app the user is in. Compare it against your own app id to tell
     * "playing this game" from "playing something else".
     */
    appId: number
    /**
     * The lobby the user is in, when the game published one. Pass it straight to
     * `matchmaking.joinLobby` to follow a friend in.
     */
    lobbyId?: bigint
  }
  /**
   * Reads the cached profile of any user Steam knows about.
   *
   * Steam only knows about users the local user shares a "source" with: friends,
   * members of the same lobby, players on the same game server. For anyone else,
   * call `requestUserInformation` first and read this once `PersonaStateChange`
   * fires — until then the name comes back empty and the level as 0.
   *
   * {@link https://partner.steamgames.com/doc/api/ISteamFriends#GetFriendPersonaName}
   */
  export function getPersona(steamId64: bigint): Persona
  /**
   * Reads the cached profiles of several users in one call.
   *
   * Same caching rules as `getPersona`. Batching matters when the caller is across
   * an IPC boundary: a lobby of eight is one round trip instead of eight.
   */
  export function getPersonas(steamIds64: Array<bigint>): Array<Persona>
  /**
   * Lists the users in one of the local user's relationship lists, with their
   * profiles already resolved — Steam always has these cached.
   */
  export function getFriends(filter: FriendFilter): Array<Persona>
  /**
   * Lists the local user's recent teammates — everyone Steam has recorded as
   * "played with" recently, across games.
   */
  export function getCoplayFriends(): Array<Persona>
  /** Whether the given user is on the local user's friends list. */
  export function isFriend(steamId64: bigint): boolean
  /**
   * What the given user is playing, or null when they are not in a game.
   *
   * The lobby id it reports is the hook for "join a friend's game": it is set for
   * any player whose game published a lobby, whether or not they are a friend.
   */
  export function getGamePlayed(steamId64: bigint): GamePlayed | null
  /**
   * Reads one rich presence value a user's game published about them.
   *
   * Returns null when the key is unset. Rich presence only crosses between players
   * of the same app, so this is always your own game's data.
   *
   * {@link https://partner.steamgames.com/doc/api/ISteamFriends#GetFriendRichPresence}
   */
  export function getRichPresence(steamId64: bigint, key: string): string | null
  /**
   * Lists the rich presence keys a user currently has set, so a caller can read
   * them without knowing the publishing side's key names up front.
   */
  export function getRichPresenceKeys(steamId64: bigint): Array<string>
  /**
   * Gets the avatar of any user Steam already knows about, in raw RGBA format.
   *
   * Steam only knows about users the local user shares a "source" with: friends,
   * members of the same lobby, players on the same game server, etc. For anyone
   * else, call `requestUserInformation` first.
   *
   * Returns `null` when the avatar is not cached yet. In that case, register a
   * `PersonaStateChange` callback, wait for it to fire for this steam id, then
   * call this function again. Do not busy-loop.
   *
   * {@link https://partner.steamgames.com/doc/api/ISteamFriends#GetLargeFriendAvatar}
   */
  export function getAvatar(steamId64: bigint, size: AvatarSize): Avatar | null
  /**
   * Asks Steam to cache the persona name and avatar of a user it does not know
   * about yet.
   *
   * @param nameOnly - When true, the avatar is not downloaded. Downloading
   * avatars is slow and churns the local cache, so pass true if you only need
   * the name.
   *
   * @returns true if the information is being requested, in which case a
   * `PersonaStateChange` callback will fire once it arrives. Returns false if
   * Steam already has everything, meaning `getAvatar` can be called right away.
   */
  export function requestUserInformation(steamId64: bigint, nameOnly: boolean): boolean
  /**
   * Sends a game invite through Steam. The invitee gets a chat notification and,
   * on accepting, their client launches the game with `connectString` on the
   * command line — or fires `GameLobbyJoinRequested` if it is already running.
   *
   * {@link https://partner.steamgames.com/doc/api/ISteamFriends#InviteUserToGame}
   */
  export function inviteUserToGame(steamId64: bigint, connectString: string): void
  /**
   * Records that the local user played with this player, which is what puts them
   * on both players' "recently played with" lists. Only works while both are in
   * the game together, so call it when a match starts, not when it ends.
   */
  export function setPlayedWith(steamId64: bigint): void
}
export declare namespace input {
  export const enum InputType {
    Unknown = 'Unknown',
    SteamController = 'SteamController',
    XBox360Controller = 'XBox360Controller',
    XBoxOneController = 'XBoxOneController',
    GenericGamepad = 'GenericGamepad',
    PS4Controller = 'PS4Controller',
    AppleMFiController = 'AppleMFiController',
    AndroidController = 'AndroidController',
    SwitchJoyConPair = 'SwitchJoyConPair',
    SwitchJoyConSingle = 'SwitchJoyConSingle',
    SwitchProController = 'SwitchProController',
    MobileTouch = 'MobileTouch',
    PS3Controller = 'PS3Controller',
    PS5Controller = 'PS5Controller',
    SteamDeckController = 'SteamDeckController'
  }
  export interface AnalogActionVector {
    x: number
    y: number
  }
  export function init(): void
  export function getControllers(): Array<Controller>
  export function getActionSet(actionSetName: string): bigint
  export function getDigitalAction(actionName: string): bigint
  export function getAnalogAction(actionName: string): bigint
  export function shutdown(): void
  export class Controller {
    activateActionSet(actionSetHandle: bigint): void
    isDigitalActionPressed(actionHandle: bigint): boolean
    getAnalogActionVector(actionHandle: bigint): AnalogActionVector
    getType(): InputType
    getHandle(): bigint
  }
}
export declare namespace localplayer {
  export function getSteamId(): PlayerSteamId
  export function getName(): string
  export function getLevel(): number
  /** @returns the 2 digit ISO 3166-1-alpha-2 format country code which client is running in, e.g. "US" or "UK". */
  export function getIpCountry(): string
  /**
   * Publishes a value other players of this game can read back with
   * `friends.getRichPresence`. Passing no value clears that one key.
   */
  export function setRichPresence(key: string, value?: string | undefined | null): void
  /**
   * Clears every rich presence key at once. Worth calling when leaving a lobby or
   * match, so the local player stops advertising a session that is over.
   */
  export function clearRichPresence(): void
}
export declare namespace matchmaking {
  export const enum LobbyType {
    Private = 0,
    FriendsOnly = 1,
    Public = 2,
    Invisible = 3
  }
  /** How a numeric lobby filter compares a lobby's value against the requested one. */
  export const enum LobbyComparison {
    Equal = 0,
    NotEqual = 1,
    GreaterThan = 2,
    GreaterThanOrEqual = 3,
    LessThan = 4,
    LessThanOrEqual = 5
  }
  /**
   * How far afield the lobby search reaches. Anything wider than `Default` trades
   * latency for population.
   */
  export const enum LobbyDistance {
    Close = 0,
    Default = 1,
    Far = 2,
    Worldwide = 3
  }
  /**
   * Matches lobbies whose data at `key` equals — or, with `exclude`, differs from —
   * `value`. The lobby has to have published that key with `setData`.
   */
  export interface LobbyStringFilter {
    key: string
    value: string
    /** Defaults to false, meaning "must equal". */
    exclude?: boolean
  }
  /** Matches lobbies whose data at `key` compares against `value` as requested. */
  export interface LobbyNumberFilter {
    key: string
    value: number
    comparison: LobbyComparison
  }
  /**
   * Does not filter anything out — sorts the results by how close their value at
   * `key` is to `value`. Use it for skill-based ordering.
   */
  export interface LobbyNearFilter {
    key: string
    value: number
  }
  /**
   * Narrows a lobby search. Without one, Steam returns whatever it likes from the
   * whole world, which is rarely what a lobby browser wants.
   *
   * Every filter applies to the *next* search only — they are consumed by the call.
   */
  export interface LobbyFilter {
    string?: Array<LobbyStringFilter>
    number?: Array<LobbyNumberFilter>
    nearValue?: Array<LobbyNearFilter>
    /** Only lobbies with at least this many free slots. */
    openSlots?: number
    distance?: LobbyDistance
    /** Caps how many lobbies come back. */
    count?: number
  }
  export function createLobby(lobbyType: LobbyType, maxMembers: number): Promise<Lobby>
  export function joinLobby(lobbyId: bigint): Promise<Lobby>
  /**
   * Searches for joinable lobbies.
   *
   * Without a filter Steam decides what to return from the whole world, so a lobby
   * browser should almost always pass one — at minimum a `count` and a string
   * filter on a key the game publishes, so other games' lobbies never show up.
   */
  export function getLobbies(filter?: LobbyFilter | undefined | null): Promise<Array<Lobby>>
  export class Lobby {
    id: bigint
    join(): Promise<Lobby>
    leave(): void
    openInviteDialog(): void
    getMemberCount(): bigint
    getMemberLimit(): bigint | null
    getMembers(): Array<PlayerSteamId>
    getOwner(): PlayerSteamId
    setJoinable(joinable: boolean): boolean
    /**
     * Changes who can find and join the lobby after it was created. Owner only.
     *
     * {@link https://partner.steamgames.com/doc/api/ISteamMatchmaking#SetLobbyType}
     */
    setType(lobbyType: LobbyType): boolean
    /**
     * Hands ownership to another member — the host migration path when the owner
     * leaves on purpose. Owner only, and the new owner must already be in the
     * lobby. When an owner drops without calling this, Steam picks a successor
     * itself and everyone gets a `LobbyDataUpdate`.
     */
    setOwner(steamId64: bigint): boolean
    /**
     * Invites a user straight to this lobby, no overlay involved. They get a Steam
     * notification and, on accepting, a `GameLobbyJoinRequested` callback fires in
     * their client. Use `openInviteDialog` instead when the player should pick the
     * invitee themselves.
     */
    inviteUser(steamId64: bigint): boolean
    getData(key: string): string | null
    setData(key: string, value: string): boolean
    deleteData(key: string): boolean
    /** Get an object containing all the lobby data */
    getFullData(): Record<string, string>
    /**
     * Merge current lobby data with provided data in a single batch
     * @returns true if all data was set successfully
     */
    mergeFullData(data: Record<string, string>): boolean
    /**
     * Publishes a value about the local user to the rest of the lobby.
     *
     * This is the counterpart to `setData` that every member can call — `setData`
     * is the owner's alone. It is how a player announces things about themselves:
     * ready state, chosen side, loaded progress. Everyone else sees a
     * `LobbyDataUpdate` whose `member` is this player, then reads it back with
     * `getMemberData`.
     *
     * {@link https://partner.steamgames.com/doc/api/ISteamMatchmaking#SetLobbyMemberData}
     */
    setMemberData(key: string, value: string): void
    /**
     * Reads a value another member published about themselves with
     * `setMemberData`. Returns null when that member never set the key.
     *
     * There is no way to enumerate a member's keys — Steam only answers by name —
     * so both sides have to agree on the key names up front.
     */
    getMemberData(steamId64: bigint, key: string): string | null
    /**
     * Broadcasts a message to every member, routed through Steam's back-end.
     *
     * Slow and bandwidth-limited compared to P2P, but it needs no session setup
     * and it reaches members who have not connected to anyone yet — which makes it
     * the right channel for lobby coordination and the wrong one for game traffic.
     *
     * Recipients get a `LobbyChatMsg` callback carrying a `chatId`, which they
     * pass to `getChatEntry` to read the bytes.
     */
    sendChatMessage(data: Buffer): boolean
    /**
     * Reads the message a `LobbyChatMsg` callback announced. Only valid inside the
     * handler for that callback — Steam recycles the entry right after.
     */
    getChatEntry(chatId: number): Buffer
  }
}
export declare namespace networking {
  export interface P2PPacket {
    data: Buffer
    size: number
    steamId: PlayerSteamId
  }
  /** The method used to send a packet */
  export const enum SendType {
    /**
     * Send the packet directly over udp.
     *
     * Can't be larger than 1200 bytes
     */
    Unreliable = 0,
    /**
     * Like `Unreliable` but doesn't buffer packets
     * sent before the connection has started.
     */
    UnreliableNoDelay = 1,
    /**
     * Reliable packet sending.
     *
     * Can't be larger than 1 megabyte.
     */
    Reliable = 2,
    /**
     * Like `Reliable` but applies the nagle
     * algorithm to packets being sent
     */
    ReliableWithBuffering = 3
  }
  export function sendP2PPacket(steamId64: bigint, sendType: SendType, data: Buffer): boolean
  export function isP2PPacketAvailable(): number
  export function readP2PPacket(size: number): P2PPacket
  export function acceptP2PSession(steamId64: bigint): void
  /**
   * Tears down the P2P session with a peer and drops anything still queued for
   * them. Every accepted session needs this when the peer leaves — Steam keeps the
   * session, and its buffers, alive until someone closes it.
   */
  export function closeP2PSession(steamId64: bigint): void
}
export declare namespace overlay {
  export const enum Dialog {
    Friends = 0,
    Community = 1,
    Players = 2,
    Settings = 3,
    OfficialGameGroup = 4,
    Stats = 5,
    Achievements = 6
  }
  export const enum StoreFlag {
    None = 0,
    AddToCart = 1,
    AddToCartAndShow = 2
  }
  export function activateDialog(dialog: Dialog): void
  export function activateDialogToUser(dialog: Dialog, steamId64: bigint): void
  export function activateInviteDialog(lobbyId: bigint): void
  export function activateToWebPage(url: string): void
  export function activateToStore(appId: number, flag: StoreFlag): void
}
export declare namespace stats {
  export function getInt(name: string): number | null
  export function setInt(name: string, value: number): boolean
  export function store(): boolean
  export function resetAll(achievementsToo: boolean): boolean
}
export declare namespace utils {
  export function getAppId(): number
  export function getServerRealTime(): number
  export function isSteamRunningOnSteamDeck(): boolean
  export const enum GamepadTextInputMode {
    Normal = 0,
    Password = 1
  }
  export const enum GamepadTextInputLineMode {
    SingleLine = 0,
    MultipleLines = 1
  }
  /** @returns the entered text, or null if cancelled or could not show the input */
  export function showGamepadTextInput(inputMode: GamepadTextInputMode, inputLineMode: GamepadTextInputLineMode, description: string, maxCharacters: number, existingText?: string | undefined | null): Promise<string | null>
  export const enum FloatingGamepadTextInputMode {
    SingleLine = 0,
    MultipleLines = 1,
    Email = 2,
    Numeric = 3
  }
  /** @returns true if the floating keyboard was shown, otherwise, false */
  export function showFloatingGamepadTextInput(keyboardMode: FloatingGamepadTextInputMode, x: number, y: number, width: number, height: number): Promise<boolean>
}
export declare namespace workshop {
  export interface UgcResult {
    itemId: bigint
    needsToAcceptAgreement: boolean
  }
  export const enum UgcItemVisibility {
    Public = 0,
    FriendsOnly = 1,
    Private = 2,
    Unlisted = 3
  }
  export interface UgcUpdate {
    title?: string
    description?: string
    changeNote?: string
    previewPath?: string
    contentPath?: string
    tags?: Array<string>
    visibility?: UgcItemVisibility
  }
  export interface InstallInfo {
    folder: string
    sizeOnDisk: bigint
    timestamp: number
  }
  export interface DownloadInfo {
    current: bigint
    total: bigint
  }
  export const enum UpdateStatus {
    Invalid = 0,
    PreparingConfig = 1,
    PreparingContent = 2,
    UploadingContent = 3,
    UploadingPreviewFile = 4,
    CommittingChanges = 5
  }
  export interface UpdateProgress {
    status: UpdateStatus
    progress: bigint
    total: bigint
  }
  export function createItem(appId?: number | undefined | null): Promise<UgcResult>
  export function updateItem(itemId: bigint, updateDetails: UgcUpdate, appId?: number | undefined | null): Promise<UgcResult>
  export function updateItemWithCallback(itemId: bigint, updateDetails: UgcUpdate, appId: number | undefined | null, successCallback: (data: UgcResult) => void, errorCallback: (err: any) => void, progressCallback?: (data: UpdateProgress) => void, progressCallbackIntervalMs?: number | undefined | null): void
  /**
   * Subscribe to a workshop item. It will be downloaded and installed as soon as possible.
   *
   * {@link https://partner.steamgames.com/doc/api/ISteamUGC#SubscribeItem}
   */
  export function subscribe(itemId: bigint): Promise<void>
  /**
   * Unsubscribe from a workshop item. This will result in the item being removed after the game quits.
   *
   * {@link https://partner.steamgames.com/doc/api/ISteamUGC#UnsubscribeItem}
   */
  export function unsubscribe(itemId: bigint): Promise<void>
  /**
   * Gets the current state of a workshop item on this client. States can be combined.
   *
   * @returns a number with the current item state, e.g. 9
   * 9 = 1 (The current user is subscribed to this item) + 8 (The item needs an update)
   *
   * {@link https://partner.steamgames.com/doc/api/ISteamUGC#GetItemState}
   * {@link https://partner.steamgames.com/doc/api/ISteamUGC#EItemState}
   */
  export function state(itemId: bigint): number
  /**
   * Gets info about currently installed content on the disc for workshop item.
   *
   * @returns an object with the the properties {folder, size_on_disk, timestamp}
   *
   * {@link https://partner.steamgames.com/doc/api/ISteamUGC#GetItemInstallInfo}
   */
  export function installInfo(itemId: bigint): InstallInfo | null
  /**
   * Get info about a pending download of a workshop item.
   *
   * @returns an object with the properties {current, total}
   *
   * {@link https://partner.steamgames.com/doc/api/ISteamUGC#GetItemDownloadInfo}
   */
  export function downloadInfo(itemId: bigint): DownloadInfo | null
  /**
   * Download or update a workshop item.
   *
   * @param highPriority - If high priority is true, start the download in high priority mode, pausing any existing in-progress Steam downloads and immediately begin downloading this workshop item.
   * @returns true or false
   *
   * {@link https://partner.steamgames.com/doc/api/ISteamUGC#DownloadItem}
   */
  export function download(itemId: bigint, highPriority: boolean): boolean
  /**
   * Get all subscribed workshop items.
   * @returns an array of subscribed workshop item ids
   */
  export function getSubscribedItems(): Array<bigint>
  export function deleteItem(itemId: bigint): Promise<void>
  export const enum UGCQueryType {
    RankedByVote = 0,
    RankedByPublicationDate = 1,
    AcceptedForGameRankedByAcceptanceDate = 2,
    RankedByTrend = 3,
    FavoritedByFriendsRankedByPublicationDate = 4,
    CreatedByFriendsRankedByPublicationDate = 5,
    RankedByNumTimesReported = 6,
    CreatedByFollowedUsersRankedByPublicationDate = 7,
    NotYetRated = 8,
    RankedByTotalVotesAsc = 9,
    RankedByVotesUp = 10,
    RankedByTextSearch = 11,
    RankedByTotalUniqueSubscriptions = 12,
    RankedByPlaytimeTrend = 13,
    RankedByTotalPlaytime = 14,
    RankedByAveragePlaytimeTrend = 15,
    RankedByLifetimeAveragePlaytime = 16,
    RankedByPlaytimeSessionsTrend = 17,
    RankedByLifetimePlaytimeSessions = 18,
    RankedByLastUpdatedDate = 19
  }
  export const enum UGCType {
    Items = 0,
    ItemsMtx = 1,
    ItemsReadyToUse = 2,
    Collections = 3,
    Artwork = 4,
    Videos = 5,
    Screenshots = 6,
    AllGuides = 7,
    WebGuides = 8,
    IntegratedGuides = 9,
    UsableInGame = 10,
    ControllerBindings = 11,
    GameManagedItems = 12,
    All = 13
  }
  export const enum UserListType {
    Published = 0,
    VotedOn = 1,
    VotedUp = 2,
    VotedDown = 3,
    Favorited = 4,
    Subscribed = 5,
    UsedOrPlayed = 6,
    Followed = 7
  }
  export const enum UserListOrder {
    CreationOrderAsc = 0,
    CreationOrderDesc = 1,
    TitleAsc = 2,
    LastUpdatedDesc = 3,
    SubscriptionDateDesc = 4,
    VoteScoreDesc = 5,
    ForModeration = 6
  }
  export interface WorkshopItemStatistic {
    numSubscriptions?: bigint
    numFavorites?: bigint
    numFollowers?: bigint
    numUniqueSubscriptions?: bigint
    numUniqueFavorites?: bigint
    numUniqueFollowers?: bigint
    numUniqueWebsiteViews?: bigint
    reportScore?: bigint
    numSecondsPlayed?: bigint
    numPlaytimeSessions?: bigint
    numComments?: bigint
    numSecondsPlayedDuringTimePeriod?: bigint
    numPlaytimeSessionsDuringTimePeriod?: bigint
  }
  export interface WorkshopItem {
    publishedFileId: bigint
    creatorAppId?: number
    consumerAppId?: number
    title: string
    description: string
    owner: PlayerSteamId
    /** Time created in unix epoch seconds format */
    timeCreated: number
    /** Time updated in unix epoch seconds format */
    timeUpdated: number
    /** Time when the user added the published item to their list (not always applicable), provided in Unix epoch format (time since Jan 1st, 1970). */
    timeAddedToUserList: number
    visibility: UgcItemVisibility
    banned: boolean
    acceptedForUse: boolean
    tags: Array<string>
    tagsTruncated: boolean
    url: string
    numUpvotes: number
    numDownvotes: number
    numChildren: number
    previewUrl?: string
    statistics: WorkshopItemStatistic
  }
  export interface WorkshopPaginatedResult {
    items: Array<WorkshopItem | undefined | null>
    returnedResults: number
    totalResults: number
    wasCached: boolean
  }
  export interface WorkshopItemsResult {
    items: Array<WorkshopItem | undefined | null>
    wasCached: boolean
  }
  export interface WorkshopItemQueryConfig {
    cachedResponseMaxAge?: number
    includeMetadata?: boolean
    includeLongDescription?: boolean
    includeAdditionalPreviews?: boolean
    onlyIds?: boolean
    onlyTotal?: boolean
    language?: string
    matchAnyTag?: boolean
    requiredTags?: Array<string>
    excludedTags?: Array<string>
    searchText?: string
    rankedByTrendDays?: number
  }
  export interface AppIDs {
    creator?: number
    consumer?: number
  }
  export function getItem(item: bigint, queryConfig?: WorkshopItemQueryConfig | undefined | null): Promise<WorkshopItem | null>
  export function getItems(items: Array<bigint>, queryConfig?: WorkshopItemQueryConfig | undefined | null): Promise<WorkshopItemsResult>
  export function getAllItems(page: number, queryType: UGCQueryType, itemType: UGCType, creatorAppId: number, consumerAppId: number, queryConfig?: WorkshopItemQueryConfig | undefined | null): Promise<WorkshopPaginatedResult>
  export function getUserItems(page: number, accountId: number, listType: UserListType, itemType: UGCType, sortOrder: UserListOrder, appIds: AppIDs, queryConfig?: WorkshopItemQueryConfig | undefined | null): Promise<WorkshopPaginatedResult>
}
