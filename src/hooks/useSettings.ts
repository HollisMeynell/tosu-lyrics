import { createSignal } from "solid-js";
import { ApiError, fetchSettings, patchSettings } from "@/services/settingsService";
import { applySettings } from "@/stores/settingsStore";
import { SettingsDto, SettingsPatch } from "@/types/globalTypes";

export const describeError = (e: unknown): string => {
    if (e instanceof ApiError) return `${e.message}（${e.code}）`;
    if (e instanceof Error) return e.message;
    return String(e);
};

/**
 * 控制台设置表单的状态机（B-03）。
 *
 * 只通过 HTTP 与后端交互：
 * - 成功 → 用**服务端返回值**覆盖本地状态（不保留乐观值）
 * - 失败 → 本地状态保持不变，把错误暴露给页面
 */
export const createSettingsController = () => {
    const [loading, setLoading] = createSignal(true);
    const [saving, setSaving] = createSignal(false);
    const [error, setError] = createSignal<string | undefined>(undefined);
    /** 最近一次**服务端确认**的完整设置；页面需要"当前值"时读它 */
    const [settings, setSettings] = createSignal<SettingsDto | null>(null);

    const load = async () => {
        setLoading(true);
        setError(undefined);
        try {
            const dto = await fetchSettings();
            setSettings(dto);
            applySettings(dto);
        } catch (e) {
            setError(describeError(e));
        } finally {
            setLoading(false);
        }
    };

    const update = async (patch: SettingsPatch): Promise<boolean> => {
        if (saving()) return false;
        setSaving(true);
        setError(undefined);
        try {
            const dto = await patchSettings(patch);
            setSettings(dto);
            applySettings(dto);
            return true;
        } catch (e) {
            setError(describeError(e));
            return false;
        } finally {
            setSaving(false);
        }
    };

    return { settings, loading, saving, error, load, update };
};
