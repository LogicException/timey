export type TaskIdFallback = 'first' | 'none';

export function suggestedTaskId(
	tasks: ReadonlyArray<{ id: number }>,
	defaultTaskId: number | null | undefined,
	fallback: TaskIdFallback = 'first'
): number | null {
	if (defaultTaskId != null && tasks.some((task) => task.id === defaultTaskId)) {
		return defaultTaskId;
	}
	if (fallback === 'first') {
		return tasks[0]?.id ?? null;
	}
	return null;
}
