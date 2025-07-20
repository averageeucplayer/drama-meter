import { getContext, setContext } from "svelte";
import type { Writable } from "svelte/store";
import type { AppContext } from "./types";

export const setAppContext = (appContext: Writable<AppContext>) => setContext("app-context", appContext);

export const getAppContext = () => getContext<Writable<AppContext>>("app-context");