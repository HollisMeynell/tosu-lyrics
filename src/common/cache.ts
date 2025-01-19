import {Lyric} from "./music-api.ts";

const LYRICS_PREFIX = "lyrics_";
const CONFIG_KEY = "l_config";
const STORE_NAME = "lyrics";

let lyricsDB: IDBDatabase | undefined;

const dbRequest = indexedDB.open(STORE_NAME, 1);

dbRequest.onsuccess = (event) => {
    const target = event.target as EventTarget & { result: IDBDatabase };
    lyricsDB = target.result;
}

dbRequest.onerror = () => {
    console.error("Failed to open lyrics database, use local storage instead.");
}

dbRequest.onupgradeneeded = (event) => {
    if (event.target == null) return;
    const target = event.target as EventTarget & { result: IDBDatabase };
    const db = target.result;
    if (!db.objectStoreNames.contains(STORE_NAME)) {
        db.createObjectStore(STORE_NAME, {keyPath: "name"});
    }
    lyricsDB = db;
}

function setLyrics(name: string, lyrics: Lyric) {
    if (lyricsDB) {
        saveLyricsByIndexedDB(name, lyrics);
    } else {
        saveLyricsByLocalStorage(name, lyrics);
    }
}

async function getLyrics(name: string): Promise<Lyric | undefined> {
    if (lyricsDB) {
        return getLyricsByIndexedDB(name);
    } else {
        return getLyricsByLocalStorage(name);
    }
}

async function getLyricsList(): Promise<string[]> {
    if (lyricsDB) {
        return getLyricsListByIndexedDB()
    } else {
        return getLyricsListByLocalStorage()
    }
}

function clearLyrics() {
    if (lyricsDB) {
        clearLyricsByIndexedDB();
    } else {
        clearLyricsByLocalStorage();
    }
}

function saveLyricsByIndexedDB(name: string, lyrics: Lyric) {
    if (!lyricsDB) return;
    const transaction = lyricsDB.transaction([STORE_NAME], 'readwrite');
    const store = transaction.objectStore(STORE_NAME);
    const request = store.put({name, lyrics: lyrics.lyrics});

    request.onerror = function (event) {
        console.error('Error saving lyrics:', event);
    };
}

async function getLyricsByIndexedDB(name: string): Promise<Lyric | undefined> {
    if (!lyricsDB) return;
    const db = lyricsDB
    new Promise((resolve, reject) => {
        const transaction = db.transaction([STORE_NAME], 'readonly');
        const store = transaction.objectStore(STORE_NAME);
        const request = store.get(name);
        request.onsuccess = () => {
            if (request.result) {
                const lyric = new Lyric();
                lyric.lyrics = request.result.lyrics;
                resolve(lyric);
            } else {
                resolve(undefined);
            }
        };
        request.onerror = () => {
            reject(request.error);
        };
    });
}

async function getLyricsListByIndexedDB(): Promise<string[]> {
    if (!lyricsDB) return [];
    const db = lyricsDB;
    return new Promise((resolve, reject) => {
        const transaction = db.transaction([STORE_NAME], 'readonly');
        const store = transaction.objectStore(STORE_NAME);
        const request = store.getAllKeys();
        request.onsuccess = () => {
            resolve(request.result as string[]);
        };
        request.onerror = () => {
            reject(request.error);
        };
    });
}

function clearLyricsByIndexedDB() {
    if (!lyricsDB) return;
    const transaction = lyricsDB.transaction([STORE_NAME], 'readwrite');
    const store = transaction.objectStore(STORE_NAME);
    const result = store.clear();
    result.onerror = function (event) {
        console.error('Error clearing lyrics:', event);
    };
}

function saveLyricsByLocalStorage(name: string, lyrics: Lyric) {
    const key = `${LYRICS_PREFIX}${name}`;
    const dataString = JSON.stringify(lyrics.lyrics);
    localStorage.setItem(key, dataString);
}

function getLyricsByLocalStorage(name: string): Lyric | undefined {
    const key = `${LYRICS_PREFIX}${name}`;
    const dataString = localStorage.getItem(key);
    if (!dataString) return undefined;
    const data = JSON.parse(dataString);
    const lyric = new Lyric();
    lyric.lyrics = data;
    return lyric;
}

function getLyricsListByLocalStorage(): string[] {
    const lyricsList = [];
    for (let i = 0; i < localStorage.length; i++) {
        const key = localStorage.key(i);
        if (key && key.startsWith(LYRICS_PREFIX)) {
            lyricsList.push(key.substring(LYRICS_PREFIX.length));
        }
    }
    return lyricsList;
}

function clearLyricsByLocalStorage() {
    for (let i = 0; i < localStorage.length; i++) {
        const key = localStorage.key(i);
        if (key && key.startsWith(LYRICS_PREFIX)) {
            localStorage.removeItem(key);
        }
    }
}

function setConfig<T>(obj: T) {
    localStorage.setItem(CONFIG_KEY, JSON.stringify(obj));
}

function getConfig<T>(): T | undefined {
    const dataString = localStorage.getItem(CONFIG_KEY);
    if (!dataString) return undefined;
    return JSON.parse(dataString);
}

export default {
    setLyricsCache: setLyrics,
    getLyricsCache: getLyrics,
    getLyricsCacheList: getLyricsList,
    clearLyricsCache: clearLyrics,

    clearLyricsByLocalStorage,
    clearLyricsByIndexedDB,

    setConfig,
    getConfig,
}
