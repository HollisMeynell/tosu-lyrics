// 对各缓存类型的CURD操作进行封装，提供统一的接口

import { Lyric } from "@/common/music-api.ts";
import { LyricLine } from "@/types/config-global";

const LYRICS_PREFIX = "lyrics_";
const CONFIG_KEY = "l_config";
const STORE_NAME = "lyrics";
const DB_VERSION = 1;

let lyricsDB: IDBDatabase | undefined;

function initIndexedDB(): Promise<IDBDatabase> {
    return new Promise((resolve, reject) => {
        const dbRequest = indexedDB.open(STORE_NAME, DB_VERSION);

        dbRequest.onsuccess = (event) => {
            const target = event.target as IDBOpenDBRequest;
            lyricsDB = target.result;
            resolve(lyricsDB);
        };

        dbRequest.onerror = () => {
            console.error(
                "Failed to open lyrics database, use local storage instead."
            );
            reject(new Error("Failed to open lyrics database"));
        };

        dbRequest.onupgradeneeded = (event) => {
            const target = event.target as IDBOpenDBRequest;
            const db = target.result;
            if (!db.objectStoreNames.contains(STORE_NAME)) {
                db.createObjectStore(STORE_NAME, { keyPath: "name" });
            }
        };
    });
}

function setConfig<T>(obj: T) {
    localStorage.setItem(CONFIG_KEY, JSON.stringify(obj));
}

function getConfig<T>(): T | undefined {
    const dataString = localStorage.getItem(CONFIG_KEY);
    if (!dataString) return undefined;
    return JSON.parse(dataString);
}

interface StorageAdapter {
    setLyrics(bid: number, name: string, lyrics: Lyric): Promise<void>;
    getLyrics(bid: number): Promise<LyricLine[] | undefined>;
    getLyricsList(): Promise<string[]>;
    clearLyrics(): Promise<void>;
}

class IndexedDBAdapter implements StorageAdapter {
    private db: IDBDatabase;

    constructor(db: IDBDatabase) {
        this.db = db;
    }

    setLyrics(bid: number, name: string, lyrics: Lyric): Promise<void> {
        return new Promise((resolve, reject) => {
            const transaction = this.db.transaction([STORE_NAME], "readwrite");
            const store = transaction.objectStore(STORE_NAME);
            const request = store.put({ bid, name, lyrics: lyrics.lyrics });

            request.onsuccess = () => resolve();
            request.onerror = () => reject(request.error);
        });
    }

    getLyrics(bid: number): Promise<LyricLine[] | undefined> {
        return new Promise((resolve, reject) => {
            const transaction = this.db.transaction([STORE_NAME], "readonly");
            const store = transaction.objectStore(STORE_NAME);
            const request = store.get(bid);

            request.onsuccess = () => {
                if (request.result) {
                    resolve(request.result.lyrics);
                } else {
                    resolve(undefined);
                }
            };
            request.onerror = () => reject(request.error);
        });
    }

    getLyricsList(): Promise<string[]> {
        return new Promise((resolve, reject) => {
            const transaction = this.db.transaction([STORE_NAME], "readonly");
            const store = transaction.objectStore(STORE_NAME);
            const request = store.getAllKeys();

            request.onsuccess = () => resolve(request.result as string[]);
            request.onerror = () => reject(request.error);
        });
    }

    clearLyrics(): Promise<void> {
        return new Promise((resolve, reject) => {
            const transaction = this.db.transaction([STORE_NAME], "readwrite");
            const store = transaction.objectStore(STORE_NAME);
            const request = store.clear();

            request.onsuccess = () => resolve();
            request.onerror = () => reject(request.error);
        });
    }
}

class LocalStorageAdapter implements StorageAdapter {
    setLyrics(bid: number, _: string, lyrics: Lyric): Promise<void> {
        const key = `${LYRICS_PREFIX}${bid}`;
        const dataString = JSON.stringify(lyrics.lyrics);
        localStorage.setItem(key, dataString);
        return Promise.resolve();
    }

    getLyrics(bid: number): Promise<LyricLine[] | undefined> {
        const key = `${LYRICS_PREFIX}${bid}`;
        const dataString = localStorage.getItem(key);
        if (!dataString) return Promise.resolve(undefined);
        const data = JSON.parse(dataString);
        // const lyric = new Lyric();
        // lyric.lyrics = data;
        // return Promise.resolve(lyric);
        return Promise.resolve(data);
    }

    getLyricsList(): Promise<string[]> {
        const lyricsList = Object.keys(localStorage)
            .filter((key) => key.startsWith(LYRICS_PREFIX))
            .map((key) => key.substring(LYRICS_PREFIX.length));
        return Promise.resolve(lyricsList);
    }

    clearLyrics(): Promise<void> {
        Object.keys(localStorage)
            .filter((key) => key.startsWith(LYRICS_PREFIX))
            .forEach((key) => localStorage.removeItem(key));
        return Promise.resolve();
    }
}

let storageAdapter: StorageAdapter | undefined;

// 初始化存储适配器
const getStorageAdapter = async (): Promise<StorageAdapter> => {
    if (!storageAdapter) {
        try {
            const db = await initIndexedDB();
            storageAdapter = new IndexedDBAdapter(db);
        } catch {
            console.warn(
                "IndexedDB initialization failed, falling back to localStorage."
            );
            storageAdapter = new LocalStorageAdapter();
        }
    }
    return storageAdapter;
};

export default {
    setLyricsCache: async (bid: number, name: string, lyrics: Lyric) => {
        const adapter = await getStorageAdapter();
        return adapter.setLyrics(bid, name, lyrics);
    },
    getLyricsCache: async (bid: number) => {
        const adapter = await getStorageAdapter();
        return adapter.getLyrics(bid);
    },
    getLyricsCacheList: async () => {
        const adapter = await getStorageAdapter();
        return adapter.getLyricsList();
    },
    clearLyricsCache: async () => {
        const adapter = await getStorageAdapter();
        return adapter.clearLyrics();
    },

    setConfig,
    getConfig,
};
