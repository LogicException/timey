export const REFRESH_TIMERS_KEY = Symbol('refreshTimers');
export const WORK_CHANGED_KEY = Symbol('workChanged');

export type RefreshTimers = () => Promise<void>;

export type WorkChangedListener = () => void | Promise<void>;

export type WorkChangedBus = {
	subscribe(listener: WorkChangedListener): () => void;
	notify(): Promise<void>;
};

export function createWorkChangedBus(): WorkChangedBus {
	const listeners = new Set<WorkChangedListener>();
	return {
		subscribe(listener) {
			listeners.add(listener);
			return () => {
				listeners.delete(listener);
			};
		},
		async notify() {
			await Promise.allSettled(
				[...listeners].map(async (listener) => {
					await listener();
				})
			);
		}
	};
}
