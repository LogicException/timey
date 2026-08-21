export type Role = 'admin' | 'user';

export type User = {
	id: number;
	username: string;
	role: Role;
};

export type AuthConfig = {
	local: boolean;
	oidc: boolean;
};

export type NamedItem = {
	id: number;
	name: string;
	archived: boolean;
};

export type Entry = {
	id: number;
	task_id: number | null;
	project_id: number | null;
	aufgabe_id: number | null;
	task_name: string | null;
	project_name: string | null;
	aufgabe_name: string | null;
	start_at: string;
	end_at: string | null;
	status: 'running' | 'complete' | 'needs_task';
};

export type WorkSnapshot = {
	session_id: number | null;
	status: 'running' | 'paused' | 'stopped' | null;
	local_date: string;
	elapsed_seconds: number;
};

export type ApiError = {
	error: string;
};
