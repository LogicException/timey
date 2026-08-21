import type { NamedItem } from './types';

export function filterNamedItems(items: NamedItem[], query: string): NamedItem[] {
	const needle = query.trim().toLowerCase();
	if (!needle) return items;
	return items.filter((item) => item.name.toLowerCase().includes(needle));
}

export function toggleId(ids: number[], id: number): number[] {
	return ids.includes(id) ? ids.filter((value) => value !== id) : [...ids, id];
}
