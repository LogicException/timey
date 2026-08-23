export type DefaultView = 'day' | 'week';

export function parseDefaultView(value: string | undefined): DefaultView {
	return value === 'week' ? 'week' : 'day';
}

export function homePath(view: DefaultView): '/day' | '/week' {
	return view === 'week' ? '/week' : '/day';
}
