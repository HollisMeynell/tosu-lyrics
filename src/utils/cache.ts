// 对各缓存类型的CURD操作进行封装，提供统一的接口

import { Lyric } from "@/common/music-api.ts";
import { LyricLine } from "@/types/config-global";

const STORE_NAME = "lyrics";
const DB_INDEX_NAME = "nameIndex";
const DB_VERSION = 1.1;

let lyricsDB: IDBDatabase | undefined;

interface CacheData {
    bid: number;
    name: string;
    length: number;
    lyrics: LyricLine[];
}

function initIndexedDB(): Promise<IDBDatabase> {
    return new Promise((resolve, reject) => {
        const dbRequest = indexedDB.open(STORE_NAME, DB_VERSION);

        dbRequest.onsuccess = (event) => {
            const target = event.target as IDBOpenDBRequest;
            lyricsDB = target.result;
            resolve(lyricsDB);
        };

        dbRequest.onerror = () => {
            console.error("Failed to open lyrics database!");
            reject(new Error("Failed to open lyrics database"));
        };

        dbRequest.onupgradeneeded = (event) => {
            const target = event.target as IDBOpenDBRequest;
            const db = target.result;
            if (!db.objectStoreNames.contains(STORE_NAME)) {
                const store = db.createObjectStore(STORE_NAME, {
                    keyPath: ["bid"],
                });
                store.createIndex(DB_INDEX_NAME, "name", { unique: false });
            }
        };
    });
}

interface StorageAdapter {
    setLyrics(
        bid: number,
        name: string,
        length: number,
        lyrics: Lyric
    ): Promise<void>;

    getLyrics(bid: number): Promise<LyricLine[] | undefined>;

    getLyricsByTitle(
        title: string
    ): Promise<undefined | { lyrics: LyricLine[]; length: number }[]>;

    /**
     * 获取歌词缓存列表
     */
    getLyricsList(): Promise<string[]>;

    clearLyrics(bid?: number): Promise<void>;
}

class IndexedDBAdapter implements StorageAdapter {
    private db: IDBDatabase;

    constructor(db: IDBDatabase) {
        this.db = db;
    }

    /**
     * 设置歌词缓存
     */
    setLyrics(
        bid: number,
        name: string,
        length: number,
        lyrics: Lyric
    ): Promise<void> {
        return new Promise((resolve, reject) => {
            const transaction = this.db.transaction([STORE_NAME], "readwrite");
            const store = transaction.objectStore(STORE_NAME);
            const data: CacheData = {
                bid,
                name,
                length,
                lyrics: lyrics.lyrics,
            };
            const request = store.put(data);

            request.onsuccess = () => resolve();
            request.onerror = () => reject(request.error);
        });
    }

    /**
     * 获取歌词缓存
     */
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

    async getLyricsByTitle(title: string): Promise<CacheData[] | undefined> {
        return new Promise((resolve, reject) => {
            const transaction = this.db.transaction([STORE_NAME], "readonly");
            const store = transaction.objectStore(STORE_NAME);
            const index = store.index(DB_INDEX_NAME);
            const result = index.getAll(title);
            result.onsuccess = () => {
                resolve(result.result);
            };
            result.onerror = () => {
                reject(result.error);
            };
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

    clearLyrics(bid?: number): Promise<void> {
        return new Promise((resolve, reject) => {
            const transaction = this.db.transaction([STORE_NAME], "readwrite");
            const store = transaction.objectStore(STORE_NAME);
            let request;
            if (bid) {
                request = store.delete(bid);
            } else {
                request = store.clear();
            }
            request.onsuccess = () => resolve();
            request.onerror = () => reject(request.error);
        });
    }
}

/**
 * 放弃使用 localStorage 缓存歌词, 主要原因是 localStorage 容量太小,
 * 大量歌词导致溢出以及性能问题, 而且没有搜索功能, 处理曲名搜索非常慢
 * 同时 chromium 在 23 版本(2012年) 以及 IE 10 就支持, 几乎不用考虑 obs 不支持的情况
 * url: https://caniuse.com/indexeddb
 */

let storageAdapter: StorageAdapter | undefined;
let initialized = false;

// 初始化存储适配器
const getStorageAdapter = async (): Promise<StorageAdapter | undefined> => {
    if (!initialized) {
        try {
            const db = await initIndexedDB();
            storageAdapter = new IndexedDBAdapter(db);
        } catch {
            console.warn("IndexedDB initialization failed, disable cache.");
        }
        initialized = true;
    }
    return storageAdapter;
};

export default {
    storageAdapter,
    getStorageAdapter,
    setLyricsCache: async (
        bid: number,
        name: string,
        length: number,
        lyrics: Lyric
    ) => {
        return storageAdapter?.setLyrics(bid, name, length, lyrics);
    },
    getLyricsCache: async (bid: number) => {
        return storageAdapter?.getLyrics(bid);
    },
    getLyricsCacheList: async () => {
        return storageAdapter?.getLyricsList();
    },
    clearLyricsCache: async () => {
        return storageAdapter?.clearLyrics();
    },
};
