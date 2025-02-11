// 对各缓存类型的CURD操作进行封装，提供统一的接口

import { Lyric } from "@/services/managers/lyricManager";

import { LyricRawLine } from "@/types/lyricTypes.ts";

const STORE_NAME = "lyrics";
const DB_INDEX_NAME = "nameIndex";
const DB_VERSION = 2;

let lyricsDB: IDBDatabase | undefined;

interface CacheData {
    sid: number;
    name: string;
    length: number;
    lyrics: LyricRawLine[];
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
            if (db.objectStoreNames.contains(STORE_NAME)) {
                // 删除旧的对象存储
                db.deleteObjectStore(STORE_NAME);
                console.log("数据库升级, 删除旧的对象存储");
            }

            const store = db.createObjectStore(STORE_NAME, {
                keyPath: ["sid"],
            });
            store.createIndex(DB_INDEX_NAME, "name", { unique: false });
        };
    });
}

interface StorageAdapter {
    setLyrics(
        sid: number,
        name: string,
        length: number,
        lyrics: Lyric
    ): Promise<void>;

    getLyrics(sid: number): Promise<LyricRawLine[] | undefined>;

    getLyricsByTitle(
        title: string
    ): Promise<undefined | { lyrics: LyricRawLine[]; length: number }[]>;

    /**
     * 获取歌词缓存列表
     */
    getLyricsList(): Promise<string[]>;

    clearLyrics(key?: number | string): Promise<void>;
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
        sid: number,
        name: string,
        length: number,
        lyrics: Lyric
    ): Promise<void> {
        return new Promise((resolve, reject) => {
            const transaction = this.db.transaction([STORE_NAME], "readwrite");
            const store = transaction.objectStore(STORE_NAME);
            const data: CacheData = {
                sid,
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
    getLyrics(sid: number): Promise<LyricRawLine[] | undefined> {
        return new Promise((resolve, reject) => {
            const transaction = this.db.transaction([STORE_NAME], "readonly");
            const store = transaction.objectStore(STORE_NAME);
            const request = store.get(sid);

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
            const result = index.getAll(IDBKeyRange.only(title));
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

    clearLyrics(key?: number | string): Promise<void> {
        return new Promise((resolve, reject) => {
            const transaction = this.db.transaction([STORE_NAME], "readwrite");
            const store = transaction.objectStore(STORE_NAME);
            let request;
            if (key == null) {
                request = store.clear();
                request.onsuccess = () => resolve();
            } else if (typeof key === "number") {
                request = store.delete(key);
                request.onsuccess = () => resolve();
            } else {
                const index = store.index(DB_INDEX_NAME);
                request = index.openCursor(IDBKeyRange.only(key));
                request.onsuccess = (event) => {
                    // @ts-expect-error ts sb
                    const cursor = event.target.result;
                    if (cursor) {
                        cursor.delete();
                        cursor.continue();
                    } else {
                        resolve();
                    }
                };
            }
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
        sid: number,
        name: string,
        length: number,
        lyrics: Lyric
    ) => {
        return storageAdapter?.setLyrics(sid, name, length, lyrics);
    },
    getLyricsCache: async (sid: number) => {
        return storageAdapter?.getLyrics(sid);
    },
    getLyricsByTitle: async (title: string, time: number) => {
        let result = await storageAdapter?.getLyricsByTitle(title);
        if (result == null || result.length == 0) return undefined;
        result = result
            .map((l) => {
                return {
                    ...l,
                    diff: Math.abs(time - l.length),
                };
            })
            .sort((a, b) => a.diff - b.diff);
        return result[0].lyrics;
    },
    getLyricsCacheList: async () => {
        return storageAdapter?.getLyricsList();
    },
    clearLyricsCache: async () => {
        return storageAdapter?.clearLyrics();
    },
};
