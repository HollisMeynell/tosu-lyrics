export const getMusicInfoUrl = (title: string) => `https://music.163.com/api/search/get/?s=${title}&type=1&limit=5`
export const getLyricUrl = (songID: number) => `https://music.163.com/api/song/lyric?os=pc&id=${songID}&lv=1&kv=1&tv=1`
export const AUDIO_URL = "http://127.0.0.1:24050/files/beatmap/audio"
export const WS_URL = "ws://127.0.0.1:24050/websocket/v2"
export const PROXY_URL = "/api/proxy"
export const MAX_TIME = 5000
