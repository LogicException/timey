import { api, HttpError } from './api.ts';
import type { AuthConfig, User } from './types.ts';

export async function fetchAuthConfig(): Promise<AuthConfig> {
	return api('/api/auth/config');
}

export async function login(username: string, password: string): Promise<User> {
	return api('/api/auth/login', {
		method: 'POST',
		body: JSON.stringify({ username, password })
	});
}

export async function logout(): Promise<void> {
	await api('/api/auth/logout', { method: 'POST' });
}

export async function fetchMe(): Promise<User | null> {
	try {
		return await api('/api/auth/me');
	} catch (err) {
		if (err instanceof HttpError && err.status === 401) {
			return null;
		}
		throw err;
	}
}
