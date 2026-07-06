/**
 * Client API pour les personnes de contact d'une entreprise (#213, CRM).
 * Miroir de `crates/kesh-api/src/routes/contact_persons.rs`. Purement informatif.
 */

import { apiClient } from '$lib/shared/utils/api-client';

export interface ContactPerson {
	id: number;
	contactId: number;
	firstName: string;
	lastName: string;
	role: string | null;
	email: string | null;
	phone: string | null;
	version: number;
}

export interface ContactPersonBody {
	firstName: string;
	lastName: string;
	role?: string | null;
	email?: string | null;
	phone?: string | null;
	version?: number;
}

export async function listContactPersons(contactId: number): Promise<ContactPerson[]> {
	return apiClient.get<ContactPerson[]>(`/api/v1/contacts/${contactId}/persons`);
}

export async function createContactPerson(
	contactId: number,
	body: ContactPersonBody
): Promise<ContactPerson> {
	return apiClient.post<ContactPerson>(`/api/v1/contacts/${contactId}/persons`, body);
}

export async function updateContactPerson(
	id: number,
	body: ContactPersonBody
): Promise<ContactPerson> {
	return apiClient.put<ContactPerson>(`/api/v1/contact-persons/${id}`, body);
}

export async function deleteContactPerson(id: number): Promise<void> {
	await apiClient.delete(`/api/v1/contact-persons/${id}`);
}
