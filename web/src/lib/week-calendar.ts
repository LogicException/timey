export function weekTimeGridLayout(): { eventMinHeight: number } {
	return { eventMinHeight: 1 };
}

export type CalendarEventColors = {
	backgroundColor: string;
	borderColor: string;
	textColor: string;
};

const FALLBACK_BACKGROUND = '#2f5d45';
const FALLBACK_BORDER = '#3f9d6c';
const LIGHT_TEXT = '#ece7dc';
const DARK_TEXT = '#121512';
const LIGHT_BACKGROUND_LUMINANCE = 0.45;

export function calendarEventColors(taskColor: string | null | undefined): CalendarEventColors {
	if (!taskColor) {
		return {
			backgroundColor: FALLBACK_BACKGROUND,
			borderColor: FALLBACK_BORDER,
			textColor: LIGHT_TEXT
		};
	}
	const luminance = relativeLuminance(taskColor);
	return {
		backgroundColor: taskColor,
		borderColor: taskColor,
		textColor: luminance > LIGHT_BACKGROUND_LUMINANCE ? DARK_TEXT : LIGHT_TEXT
	};
}

function relativeLuminance(hex: string): number {
	const rgb = rgbFromHex(hex);
	if (!rgb) return 0;
	return 0.2126 * channelToLinear(rgb.r) + 0.7152 * channelToLinear(rgb.g) + 0.0722 * channelToLinear(rgb.b);
}

function rgbFromHex(hex: string): { r: number; g: number; b: number } | null {
	if (!/^#[0-9A-Fa-f]{6}$/.test(hex)) return null;
	return {
		r: Number.parseInt(hex.slice(1, 3), 16),
		g: Number.parseInt(hex.slice(3, 5), 16),
		b: Number.parseInt(hex.slice(5, 7), 16)
	};
}

function channelToLinear(channel: number): number {
	const srgb = channel / 255;
	if (srgb <= 0.04045) return srgb / 12.92;
	return ((srgb + 0.055) / 1.055) ** 2.4;
}
