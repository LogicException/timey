export function isReservedTaskName(name: string): boolean {
	return name.trim().toLowerCase() === 'unbestimmt';
}

export function renameIfChanged(original: string, next: string): string | null {
	const trimmed = next.trim();
	if (trimmed.length === 0 || trimmed === original) {
		return null;
	}
	return trimmed;
}
