/**
 * Types de la feature `projects` (Epic 19 — compta analytique par projet, Story 19-1).
 *
 * Hiérarchie à 2 niveaux : `parentId === null` → projet racine ; sinon sous-projet
 * (le parent est toujours une racine). Le champ `version` porte le verrou optimiste.
 */

export type ProjectResponse = {
	id: number;
	/** `null` = projet racine ; sinon id du projet parent (une racine). */
	parentId: number | null;
	/** Identifiant court unique par company (ex. `RENOV-CHALET`). */
	code: string;
	name: string;
	description: string | null;
	archived: boolean;
	/** Date ISO `YYYY-MM-DD` ou `null`. */
	startDate: string | null;
	endDate: string | null;
	/** Verrou optimiste — à renvoyer au `PUT`/archive. */
	version: number;
	createdAt: string;
};

/** Corps de `POST /api/v1/projects`. */
export type NewProjectPayload = {
	parentId?: number | null;
	code: string;
	name: string;
	description?: string | null;
	startDate?: string | null;
	endDate?: string | null;
};

/** Corps de `PUT /api/v1/projects/{id}`. */
export type UpdateProjectPayload = {
	parentId?: number | null;
	code: string;
	name: string;
	description?: string | null;
	startDate?: string | null;
	endDate?: string | null;
	version: number;
};
