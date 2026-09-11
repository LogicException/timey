export const DEFAULT_SLOT_MINUTES = 60;

export function effectiveSlotMinutes(value: number | null | undefined): number {
	return value ?? DEFAULT_SLOT_MINUTES;
}

export type SlotMinutesParse =
	| { ok: true; value: number | null }
	| { ok: false };

export function parseSlotMinutesInput(raw: string): SlotMinutesParse {
	const trimmed = raw.trim();
	if (trimmed === '') {
		return { ok: true, value: null };
	}
	if (!/^[1-9]\d*$/.test(trimmed)) {
		return { ok: false };
	}
	return { ok: true, value: Number(trimmed) };
}

