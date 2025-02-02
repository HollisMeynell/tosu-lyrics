import { Settings } from "@/types/config-global";
import { BACKEND_CONFIG_URL } from "@/config/constants";
import debounce from "lodash/debounce";

class ConfigService {
    private debouncedSave: (config: Settings) => void;

    constructor() {
        // 500ms 后执行保存，如果期间有新的调用则重置计时
        this.debouncedSave = debounce(
            this.saveConfigImmediately.bind(this),
            1000
        );
    }

    async fetchConfig(): Promise<Settings> {
        const response = await fetch(BACKEND_CONFIG_URL);
        if (!response.ok) {
            throw new Error(`Failed to fetch config: ${response.status}`);
        }
        return response.json();
    }

    private async saveConfigImmediately(config: Settings): Promise<void> {
        try {
            const response = await fetch(BACKEND_CONFIG_URL, {
                method: "PUT",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify(config),
            });

            if (!response.ok) {
                throw new Error(`Failed to save config: ${response.status}`);
            }
        } catch (error) {
            console.error("Failed to save config:", error);
            throw error;
        }
    }

    saveConfig(config: Settings): void {
        this.debouncedSave(config);
    }
}

export const configService = new ConfigService();
