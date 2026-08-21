import { goto } from '$app/navigation';
import type { ApiError } from './types.ts';

export class HttpError extends Error {
	status: number;
	constructor(status: number, message: string) {
		super(message);
		this.status = status;
	}
}

export async function api<T>(path: string, init: RequestInit = {}): Promise<T> {
	const headers = new Headers(init.headers);
	if (init.body && !headers.has('content-type')) {
		headers.set('content-type', 'application/json');
	}
	const response = await fetch(path, {
		...init,
		headers,
		credentials: 'include'
	});

	if (response.status === 401 && !path.startsWith('/api/auth/')) {
		await goto('/login');
		throw new HttpError(401, 'nicht angemeldet');
	}

	if (!response.ok) {
		let message = `Fehler ${response.status}`;
		try {
			const body = (await response.json()) as ApiError;
			if (body.error) message = body.error;
		} catch {
			// keep status message
		}
		throw new HttpError(response.status, message);
	}

	if (response.status === 204) {
		return undefined as T;
	}
	const contentType = response.headers.get('content-type') ?? '';
	if (contentType.includes('application/json')) {
		return (await response.json()) as T;
	}
	return (await response.text()) as T;
}
