export const AUDIO_URL = "http://127.0.0.1:24050/files/beatmap/audio"
export const WS_URL = "ws://127.0.0.1:24050/websocket/v2"
export const PROXY_URL = "/api/proxy"
export const TIME_DIFF_FILERT = (songLength: number, audioLength: number) => {
    const diff = Math.abs(songLength - audioLength)
    // 相差 5s 以内
    return diff < 8000
}
export const WS_DELAY_TIME = 100
export const WAIT_AUDIO_METADATA = 1000
