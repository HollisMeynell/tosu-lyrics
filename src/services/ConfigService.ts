import { Settings } from "@/types/config-global";
import { BACKEND_CONFIG_URL } from "@/config/constants";

class ConfigService {
    // 这里不适合使用防抖来处理全部的保存
    // 如果用户在极短时间内多次修改多个属性, 此时只有最后一个属性被修改
    // private debouncedSave: (config: Settings) => void ;

    async fetchConfig(): Promise<Settings> {
        const response = await fetch(BACKEND_CONFIG_URL);
        if (!response.ok) {
            throw new Error(`Failed to fetch config: ${response.status}`);
        }
        return response.json();
    }

    async saveConfig(config: Settings): Promise<void> {
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
}

export const configService = new ConfigService();
