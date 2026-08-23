export function renameIfChanged(original: string, next: string): string | null {
	const trimmed = next.trim();
	if (trimmed.length === 0 || trimmed === original) {
		return null;
	}
	return trimmed;
}
