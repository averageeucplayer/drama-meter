import { invoke } from "@tauri-apps/api/core";
import type { LoadResult } from "./types";

export const load = async (): Promise<LoadResult> => {
    return invoke<LoadResult>("load");
}