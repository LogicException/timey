export const REFRESH_TIMERS_KEY = Symbol('refreshTimers');

export type RefreshTimers = () => Promise<void>;
