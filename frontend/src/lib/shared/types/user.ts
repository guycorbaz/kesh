/** Types pour la gestion des utilisateurs (Story 1.12). */

export type Role = 'Admin' | 'Comptable' | 'Consultation';

export interface UserResponse {
	id: number;
	username: string;
	role: Role;
	active: boolean;
	version: number;
	createdAt: string;
	updatedAt: string;
}

export interface UserListResponse {
	items: UserResponse[];
	total: number;
	offset: number;
	limit: number;
}
