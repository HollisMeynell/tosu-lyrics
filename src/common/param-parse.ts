import Cache from "./cache.ts";

declare global {
    interface Window {
        ignoreCORS?: boolean
    }
}

const param = {
    console: false
}

export function consoleShow() {
    return param.console
}

export function paramParse(params: URLSearchParams) {
    if (params.get('cors')) {
        window.ignoreCORS = true;
    }
    if (params.get('clear-cache')) {
        try {
            Cache.clearLyricsByLocalStorage()
        } catch (e) {
            console.error(e)
        }
        try {
            Cache.clearLyricsByIndexedDB()
        } catch (e) {
            console.error(e)
        }
    }
    param.console = !!params.get('console')
}
