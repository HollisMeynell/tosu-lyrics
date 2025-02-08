import { Settings } from "@/types/globalTypes";
import { BACKEND_CONFIG_URL } from "@/config/constants";

class ConfigService {
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
                console.error(`Failed to save config: ${response.status}`);
            }
        } catch (error) {
            console.error("Failed to save config:", error);
        }
    }
}

export const configService = new ConfigService();
