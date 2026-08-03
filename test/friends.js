const { init, SteamCallback } = require('../index.js')

const client = init(480)

const steamId = client.localplayer.getSteamId().steamId64

function tryAvatar(label) {
    const avatar = client.friends.getAvatar(steamId, client.friends.AvatarSize.Medium)
    if (avatar) {
        console.log(`${label}: ${avatar.width}x${avatar.height}, ${avatar.data.length} bytes`)
        return true
    }

    console.log(`${label}: not cached yet`)
    return false
}

if (!tryAvatar('immediate')) {
    // Not cached. Ask Steam for it and wait for the callback instead of polling.
    const requested = client.friends.requestUserInformation(steamId, false)
    console.log('requestUserInformation returned: ' + requested)

    // `init` already pumps the callbacks, so we only have to listen.
    const handle = client.callback.register(SteamCallback.PersonaStateChange, (value) => {
        if (value.steam_id !== steamId) {
            return
        }

        if (tryAvatar('after PersonaStateChange')) {
            handle.disconnect()
            clearTimeout(timeout)
        }
    })

    const timeout = setTimeout(() => {
        console.log('gave up waiting for the avatar')
        handle.disconnect()
    }, 10000)
}
