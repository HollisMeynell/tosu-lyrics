export const getMusicInfoUrl = (title: string) => `https://c.y.qq.com/soso/fcgi-bin/client_search_cp?p=1&n=10&format=json&w=${title}`
export const getLyricUrl = (songID: number|string) => `https://c.y.qq.com/lyric/fcgi-bin/fcg_query_lyric_new.fcg?songmid=${songID}&format=json&nobase64=1`
export const LyricUrlHeader = {
    "Referer": "https://y.qq.com/portal/player.html"
}
