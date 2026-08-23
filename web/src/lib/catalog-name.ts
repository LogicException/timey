export function isReservedTaskName(name: string): boolean {
	return name.trim().toLowerCase() === 'unbestimmt';
}

export function deleteTaskConfirmText(taskName: string): string {
	return `„${taskName}“ wirklich löschen? Vorhandene Einträge zu diesem Task werden auf den internen Task „unbestimmt“ umgebucht.`;
}

export function renameIfChanged(original: string, next: string): string | null {
	const trimmed = next.trim();
	if (trimmed.length === 0 || trimmed === original) {
		return null;
	}
	return trimmed;
}
