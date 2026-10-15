// Story 30-2b (#685, AC14) — fournisseurs proposés : clé de nom, termes de recherche,
// éligibilité et ordre. « Un appariement propose, il ne crée jamais » (CLAUDE.md).

import { describe, it, expect } from 'vitest';
import type { ContactResponse } from '$lib/features/contacts/contacts.types';
import { proposeSuppliers, searchTermsFor, supplierNameKey } from './supplier-proposals';

function contact(id: number, name: string, extra: Partial<ContactResponse> = {}): ContactResponse {
	return {
		id,
		companyId: 1,
		contactType: 'Entreprise',
		name,
		firstName: null,
		lastName: null,
		isClient: false,
		isSupplier: true,
		address: null,
		addressStructured: { street: '', building: '', postalCode: '', city: '', country: 'CH' },
		email: null,
		phone: null,
		ideNumber: null,
		clientNumber: null,
		defaultPaymentTerms: null,
		defaultPaymentTermsDays: null,
		defaultPaymentTermsLabel: null,
		language: null,
		salutation: 'Neutre',
		active: true,
		version: 1,
		createdAt: '2026-01-01T00:00:00',
		updatedAt: '2026-01-01T00:00:00',
		...extra,
	};
}

describe('supplierNameKey', () => {
	it('retire les formes juridiques comme suites de mots, la plus longue d’abord', () => {
		expect(supplierNameKey('DUBARDE S.à r.l.')).toEqual(['dubarde']);
		expect(supplierNameKey('Dubarde SàRL')).toEqual(['dubarde']);
		expect(supplierNameKey('Dubarde Sàrl Genève')).toEqual(['dubarde', 'geneve']);
		expect(supplierNameKey('Rossi S.a.g.l.')).toEqual(['rossi']);
		expect(supplierNameKey('Bianchi S.r.l.')).toEqual(['bianchi']);
		expect(supplierNameKey('Weber GmbH & Co. KG')).toEqual(['weber']);
	});

	it('développe les trémas des deux côtés : « Müller » et « Mueller » ont la même clé', () => {
		expect(supplierNameKey('Müller AG')).toEqual(['mueller']);
		expect(supplierNameKey('Mueller AG')).toEqual(['mueller']);
		expect(supplierNameKey('Weiß')).toEqual(['weiss']);
		// Angle mort écrit : sans tréma ni « e », « Muller » reste distinct.
		expect(supplierNameKey('Muller SA')).toEqual(['muller']);
	});

	it('retire les mots de liaison, et un nom fait de formes juridiques a une clé vide', () => {
		expect(supplierNameKey('Dupont et Fils SA')).toEqual(['dupont', 'fils']);
		expect(supplierNameKey('Meier und Partner AG')).toEqual(['meier', 'partner']);
		expect(supplierNameKey('SA')).toEqual([]);
		expect(supplierNameKey('')).toEqual([]);
	});
});

describe('searchTermsFor', () => {
	it('ajoute, pour un mot à ae/oe/ue, la variante à tréma et la variante sans e', () => {
		expect(searchTermsFor('Müller AG')).toBe('mueller müller muller');
	});

	it('ne cherche pas les mots de moins de trois caractères', () => {
		expect(searchTermsFor('Li AB')).toBe('');
		expect(searchTermsFor('Dubarde S.à r.l.')).toBe('dubarde');
	});
});

describe('proposeSuppliers', () => {
	it('propose sur égalité de clé (« Müller AG » / « Mueller AG », « Dubarde SàRL » / « DUBARDE S.à r.l. »)', () => {
		const p = proposeSuppliers([], [contact(1, 'Mueller AG')], 'Müller AG');
		expect(p.map((x) => [x.contact.id, x.reason])).toEqual([[1, 'name']]);
		const q = proposeSuppliers([], [contact(2, 'DUBARDE S.à r.l.')], 'Dubarde SàRL');
		expect(q.map((x) => x.contact.id)).toEqual([2]);
	});

	it('propose sur inclusion (« DUBARDE S.à r.l. » contre « Dubarde Sàrl Genève »)', () => {
		const p = proposeSuppliers([], [contact(1, 'Dubarde Sàrl Genève')], 'DUBARDE S.à r.l.');
		expect(p.map((x) => x.contact.id)).toEqual([1]);
	});

	it('ne propose pas un contact qui ne partage qu’une forme juridique ou un mot de liaison', () => {
		expect(proposeSuppliers([], [contact(1, 'Nicolet Sàrl')], 'Zurcher Sàrl')).toEqual([]);
		expect(proposeSuppliers([], [contact(2, 'Favre et Cie')], 'Rochat et Cie')).toEqual([]);
	});

	it('classe l’IBAN d’abord, puis l’égalité, puis l’inclusion ; trois au plus, sans doublon', () => {
		const iban = contact(10, 'Autre Nom SA');
		const egal = contact(11, 'Dubarde SA');
		const inclus = contact(12, 'Dubarde Genève SA');
		const inclus2 = contact(13, 'Dubarde Lausanne SA');
		const p = proposeSuppliers(
			[iban],
			[inclus, inclus2, egal, contact(10, 'Autre Nom SA')],
			'Dubarde Sàrl',
		);
		expect(p.map((x) => [x.contact.id, x.reason])).toEqual([
			[10, 'iban'],
			[11, 'name'],
			[12, 'name'],
		]);
	});

	it('départage les égalités de clé par `rank`', () => {
		const p = proposeSuppliers(
			[],
			[contact(2, 'Dubarde Lausanne'), contact(1, 'Dubarde Genève')],
			'Dubarde',
		);
		expect(p.map((x) => x.contact.id)).toEqual([1, 2]);
	});

	it('écarte un contact archivé ou non fournisseur que la recherche aurait rendu', () => {
		const p = proposeSuppliers(
			[],
			[contact(1, 'Dubarde SA', { active: false }), contact(2, 'Dubarde SA', { isSupplier: false })],
			'Dubarde SA',
		);
		expect(p).toEqual([]);
	});

	it('sans nom de créancier, ne propose que par IBAN', () => {
		const p = proposeSuppliers([contact(1, 'X SA')], [contact(2, 'Y SA')], null);
		expect(p.map((x) => x.contact.id)).toEqual([1]);
	});
});
