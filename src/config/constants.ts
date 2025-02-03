export const BACKEND_CONFIG_URL = "http://127.0.0.1:41280/api/config";
export const BACKEND_WEBSOCKET_URL = "http://127.0.0.1:41280/api/ws";
export let PROXY_URL = "http://127.0.0.1:41280/api/proxy"
export const AUDIO_URL = "http://127.0.0.1:24050/files/beatmap/audio";
export const WS_URL = "ws://127.0.0.1:24050/websocket/v2";

export const WS_QUERY_TIMEOUT = 5000;

export const SEARCH_MUSIC_URL = (adaptor: string,title: string) =>{
    if(adaptor === "QQ"){
        return `https://c.y.qq.com/soso/fcgi-bin/client_search_cp?p=1&n=10&format=json&w=${title}`;
    }else if(adaptor === "Netease"){
        return `https://music.163.com/api/search/get/?s=${title}&type=1&limit=5`;
    }
    return "";
}

export const GET_LYRIC_URL = (adaptor: string,songID: number | string) =>{
    if(adaptor === "QQ"){
        return `https://c.y.qq.com/lyric/fcgi-bin/fcg_query_lyric_new.fcg?songmid=${songID}&format=json&nobase64=1`;
    }else if(adaptor === "Netease"){
        return `https://music.163.com/api/song/lyric?id=${songID}&lv=1&kv=1&tv=-1`;
    }
    return "";
}

export const setProxyUrl = (url: string) => {
    PROXY_URL = url;
}
