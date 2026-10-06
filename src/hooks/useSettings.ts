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
 * 全局 PATCH 串行链。
 *
 * 设置接口每次响应都是**完整快照**，所以必须保证同一时刻只有一个 PATCH 在途；
 * 否则并发的多个 controller 之间会用旧快照互相覆盖（例如刚改好的字体被对齐方式
 * 的响应还原回旧值）。
 */
let patchChain: Promise<unknown> = Promise.resolve();

/** 只通过 HTTP 交互：成功用服务端返回值覆盖本地，失败保持本地不变。 */
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
            // **全局串行化**：`saving()` 只是本实例的锁，而项目里有多个
            // createSettingsController 实例（文字样式 / 阴影 / 在线展示端 / 共用字体
            // 开关），它们可以并发 PATCH。而每个响应都是**完整的服务端设置快照**，
            // 两个 PATCH 同时在途时，后到的响应可能携带更旧的快照，把刚改好的字段
            // （例如 font.first / font.second）覆盖回去 —— 这正是"改完字体再切对齐
            // 方式，字体被还原"的原因。串行后响应按发送顺序应用，最后一次应用的
            // 就是包含全部改动的状态。
            const run = patchChain.then(() => patchSettings(patch));
            patchChain = run.catch(() => undefined);
            const dto = await run;
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
