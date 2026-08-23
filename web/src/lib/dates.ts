export const TZ = 'Europe/Berlin';

function pad(value: number): string {
	return String(value).padStart(2, '0');
}

function berlinParts(date: Date): Record<string, string> {
	const formatter = new Intl.DateTimeFormat('en-US', {
		timeZone: TZ,
		hourCycle: 'h23',
		year: 'numeric',
		month: '2-digit',
		day: '2-digit',
		hour: '2-digit',
		minute: '2-digit',
		second: '2-digit'
	});
	return Object.fromEntries(formatter.formatToParts(date).map((part) => [part.type, part.value]));
}

export function formatBerlinDate(date: Date): string {
	const parts = berlinParts(date);
	return `${parts.year}-${parts.month}-${parts.day}`;
}

export function formatBerlinTime(date: Date): string {
	const parts = berlinParts(date);
	return `${parts.hour}:${parts.minute}`;
}

export function berlinLocalToUtc(date: string, hours: number, minutes: number): Date {
	const utcMillis = Date.parse(`${date}T${pad(hours)}:${pad(minutes)}:00.000Z`);
	const parts = berlinParts(new Date(utcMillis));
	const asBerlin = Date.UTC(
		Number(parts.year),
		Number(parts.month) - 1,
		Number(parts.day),
		Number(parts.hour),
		Number(parts.minute),
		Number(parts.second)
	);
	return new Date(utcMillis - (asBerlin - utcMillis));
}

export function isoToBerlinDate(iso: string): string {
	return formatBerlinDate(new Date(iso));
}

export function isoToBerlinHoursMinutes(iso: string): { hours: number; minutes: number } {
	const parts = berlinParts(new Date(iso));
	return { hours: Number(parts.hour), minutes: Number(parts.minute) };
}

export function addDays(date: string, amount: number): string {
	const [year, month, day] = date.split('-').map(Number);
	const utc = Date.UTC(year, month - 1, day + amount);
	const next = new Date(utc);
	return `${next.getUTCFullYear()}-${pad(next.getUTCMonth() + 1)}-${pad(next.getUTCDate())}`;
}

export function startOfWeek(date: string): string {
	const [year, month, day] = date.split('-').map(Number);
	const utc = new Date(Date.UTC(year, month - 1, day));
	const weekday = utc.getUTCDay(); // 0 Sun
	const offset = weekday === 0 ? -6 : 1 - weekday;
	return addDays(date, offset);
}

export function endOfWeek(date: string): string {
	return addDays(startOfWeek(date), 6);
}

export function startOfMonth(date: string): string {
	return `${date.slice(0, 7)}-01`;
}

export function endOfMonth(date: string): string {
	const [year, month] = date.split('-').map(Number);
	const last = new Date(Date.UTC(year, month, 0)).getUTCDate();
	return `${year}-${pad(month)}-${pad(last)}`;
}

export type RangePreset =
	| 'today'
	| 'yesterday'
	| 'this_week'
	| 'last_week'
	| 'this_month'
	| 'last_month'
	| 'custom';

export type DateRange = { from: string; to: string };

export function rangeForPreset(preset: RangePreset, today: string): DateRange {
	switch (preset) {
		case 'today':
			return { from: today, to: today };
		case 'yesterday': {
			const yesterday = addDays(today, -1);
			return { from: yesterday, to: yesterday };
		}
		case 'this_week':
			return { from: startOfWeek(today), to: endOfWeek(today) };
		case 'last_week': {
			const last = addDays(startOfWeek(today), -1);
			return { from: startOfWeek(last), to: endOfWeek(last) };
		}
		case 'this_month':
			return { from: startOfMonth(today), to: endOfMonth(today) };
		case 'last_month': {
			const lastMonthDay = addDays(startOfMonth(today), -1);
			return { from: startOfMonth(lastMonthDay), to: endOfMonth(lastMonthDay) };
		}
		case 'custom':
			return { from: today, to: today };
	}
}

export function showsManualDateFields(preset: RangePreset): boolean {
	return preset === 'custom';
}

export const PRESET_LABELS: Record<RangePreset, string> = {
	today: 'Heute',
	yesterday: 'Gestern',
	this_week: 'Diese Woche',
	last_week: 'Letzte Woche',
	this_month: 'Dieser Monat',
	last_month: 'Letzter Monat',
	custom: 'Individuell'
};
