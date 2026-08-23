import type { NamedItem } from './types';

export type NamedSelectOption = {
	id: number;
	name: string;
};

export function namedSelectOptions(
	items: NamedItem[],
	value: number | null,
	currentLabel: string | null
): NamedSelectOption[] {
	const visible = items
		.filter((item) => !item.archived && !item.system)
		.map((item) => ({ id: item.id, name: item.name }));
	if (value == null || visible.some((item) => item.id === value)) {
		return visible;
	}
	const current = items.find((item) => item.id === value);
	if (current) {
		return [{ id: current.id, name: current.name }, ...visible];
	}
	if (currentLabel) {
		return [{ id: value, name: currentLabel }, ...visible];
	}
	return visible;
}
