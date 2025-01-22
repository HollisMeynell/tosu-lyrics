export const getMusicInfoUrl = (title: string) => `https://music.163.com/api/search/get/?s=${title}&type=1&limit=5`
export const getLyricUrl = (songID: number|string) => `https://music.163.com/api/song/lyric?os=pc&id=${songID}&lv=1&kv=1&tv=1`
export const splitReg = /(?=\[\d+:\d+\.\d+])/;
export const parseReg = /\[(\d+):(\d+\.\d+)](.*)/
