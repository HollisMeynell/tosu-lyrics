import { createSignal } from "solid-js";

export const [updateTrigger, setUpdateTrigger] = createSignal(0);

class BlackList {
    public set: Set<string> = new Set();

    inBlackList(title: string) {
        return this.set.has(title);
    }

    reset(data?: Set<string> | string[] | null) {
        if (!data) {
            this.set = new Set();
        } else if (data instanceof Set) {
            this.set = new Set(data);
        } else if (Array.isArray(data)) {
            this.set = new Set(data);
        } else {
            // Handle invalid input
            console.warn('Invalid input for BlackList reset. Using empty Set.');
            this.set = new Set();
        }
        setUpdateTrigger((v) => v + 1);
    }

    add(title: string) {
        if (this.set.has(title)) return;
        this.set.add(title);
        setUpdateTrigger((v) => v + 1);
    }

    remove(title: string) {
        if (!this.set.has(title)) return;
        this.set.delete(title);
        setUpdateTrigger((v) => v + 1);
    }
}

export const blackList = new BlackList();