export function formatWorkDuration(totalSeconds: number): string {
	const seconds = Math.max(0, Math.floor(totalSeconds));
	const hours = Math.floor(seconds / 3600);
	const minutes = Math.floor((seconds % 3600) / 60);
	const hourLabel = hours === 1 ? 'Stunde' : 'Stunden';
	const minuteLabel = minutes === 1 ? 'Minute' : 'Minuten';
	return `${hours} ${hourLabel} ${minutes} ${minuteLabel}`;
}

export function formatHm(totalSeconds: number): string {
	const seconds = Math.max(0, Math.floor(totalSeconds));
	const hours = Math.floor(seconds / 3600);
	const minutes = Math.floor((seconds % 3600) / 60);
	return `${hours}:${String(minutes).padStart(2, '0')}`;
}

export function durationBetween(startIso: string, endIso: string | null): number {
	if (!endIso) return 0;
	return Math.max(0, (Date.parse(endIso) - Date.parse(startIso)) / 1000);
}
