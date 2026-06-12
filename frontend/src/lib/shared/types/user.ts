/** Types pour la gestion des utilisateurs (Story 1.12). */

export type Role = 'Admin' | 'Comptable' | 'Consultation';

export interface UserResponse {
	id: number;
	username: string;
	role: Role;
	active: boolean;
	/**
	 * Email de recovery (Story 17-4a backend / 17-4d UI). Nullable : un compte
	 * sans email n'est pas recouvrable par le flux self-service (break-glass
	 * #121). ⚠️ Le `PUT /users/:id` a une sémantique de REMPLACEMENT (dette
	 * ECH2-2) : toujours renvoyer `email` dans le corps, sinon il est effacé.
	 */
	email: string | null;
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
