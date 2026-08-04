// Story 16-1b — AC14-bis : libellé du compte de produit sur les écrans de
// détail. Les deux écrans (facture, avoir) délèguent à `account-label`, dont la
// seule différence de règle porte sur le cas `null`.

import { describe, it, expect, vi } from 'vitest';

vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_key: string, fallback: string) => fallback,
}));
import type { AccountResponse } from './accounts.types';
import { invoiceRevenueAccountLabel, creditNoteRevenueAccountLabel } from './account-label';

function acc(
	partial: Partial<AccountResponse> & { id: number; number: string; name: string }
): AccountResponse {
	return {
		companyId: 1,
		accountType: 'Revenue',
		active: true,
		role: null,
		postable: true,
		parentId: null,
		version: 1,
		createdAt: '',
		updatedAt: '',
		...partial,
	} as AccountResponse;
}

const DEFAULT_REVENUE = acc({ id: 3000, number: '3000', name: 'Ventes' });
const SERVICES = acc({ id: 3200, number: '3200', name: 'Honoraires' });
/** Un compte archivé DEPUIS la validation — le piège de D11, côté lecture. */
const ARCHIVED = acc({ id: 3900, number: '3900', name: 'Ventes closes', active: false });

const ACCOUNTS = [DEFAULT_REVENUE, SERVICES, ARCHIVED];

describe('écran FACTURE — invoiceRevenueAccountLabel (AC6-bis / AC14-bis)', () => {
	it('une ligne avec compte affiche numéro + libellé', () => {
		expect(invoiceRevenueAccountLabel(ACCOUNTS, SERVICES.id, DEFAULT_REVENUE.id, true)).toBe(
			'3200 — Honoraires'
		);
	});

	it('une ligne sans compte nomme le défaut société AVEC la mention « (défaut) »', () => {
		expect(invoiceRevenueAccountLabel(ACCOUNTS, null, DEFAULT_REVENUE.id, true)).toBe(
			'3000 — Ventes (défaut)'
		);
	});

	it("un compte ARCHIVÉ référencé garde son libellé — non-régression de D11 en lecture", () => {
		// Le piège de D11 se rejoue à l'identique ici : sans `fetchAccounts(true)`,
		// le compte n'est pas dans la liste et la cellule tombe sur `#3900`.
		expect(invoiceRevenueAccountLabel(ACCOUNTS, ARCHIVED.id, DEFAULT_REVENUE.id, true)).toBe(
			'3900 — Ventes closes'
		);
	});

	it('un compte non résoluble retombe sur `#id`, jamais sur du vide', () => {
		expect(invoiceRevenueAccountLabel([], SERVICES.id, DEFAULT_REVENUE.id, true)).toBe('#3200');
	});

	it("sur une facture NON brouillon, une ligne à null affiche un tiret, PAS « (défaut) »", () => {
		// L'écriture est déjà passée : annoncer un repli qui n'aura pas lieu serait
		// le même mensonge que celui qu'on refuse côté avoir. Population réelle —
		// le backfill de 16-1a-bis laisse délibérément à `NULL` les pièces dont
		// l'écriture a été retouchée, et exclut les `cancelled`.
		// (Passe 2 de revue, Edge Case Hunter.)
		const out = invoiceRevenueAccountLabel(ACCOUNTS, null, DEFAULT_REVENUE.id, false);
		expect(out).toBe('—');
		expect(out).not.toContain('défaut');
		expect(out).not.toContain('3000');
	});

	it("sans défaut société configuré, une ligne à null affiche un tiret", () => {
		// Rien de vrai à afficher : ne pas inventer de compte cible.
		expect(invoiceRevenueAccountLabel(ACCOUNTS, null, null, true)).toBe('—');
	});
});

describe('écran AVOIR — creditNoteRevenueAccountLabel (AC6-bis / AC14-bis)', () => {
	it('une ligne avec compte affiche numéro + libellé', () => {
		expect(creditNoteRevenueAccountLabel(ACCOUNTS, SERVICES.id)).toBe('3200 — Honoraires');
	});

	it('une ligne à null affiche un TIRET, JAMAIS la mention « (défaut) »', () => {
		// L'écriture d'avoir est déjà passée : annoncer un repli serait un mensonge
		// sur une pièce comptable. Cas résiduel — avoir antérieur au déploiement
		// que le backfill de 16-1a-bis n'a pas pu reprendre.
		const out = creditNoteRevenueAccountLabel(ACCOUNTS, null);
		expect(out).toBe('—');
		expect(out).not.toContain('défaut');
		expect(out).not.toContain('3000');
	});

	it('un compte archivé référencé garde son libellé', () => {
		expect(creditNoteRevenueAccountLabel(ACCOUNTS, ARCHIVED.id)).toBe('3900 — Ventes closes');
	});

	it('les deux écrans peuvent légitimement afficher DEUX comptes différents', () => {
		// Couple antérieur au déploiement dont le défaut société a changé entre la
		// validation et l'émission : les deux pièces ont réellement mouvementé des
		// comptes distincts (16-1a-bis D-B7). Aucune harmonisation, aucun marqueur.
		const surFacture = invoiceRevenueAccountLabel(ACCOUNTS, DEFAULT_REVENUE.id, SERVICES.id, true);
		const surAvoir = creditNoteRevenueAccountLabel(ACCOUNTS, SERVICES.id);
		expect(surFacture).toBe('3000 — Ventes');
		expect(surAvoir).toBe('3200 — Honoraires');
		expect(surFacture).not.toBe(surAvoir);
	});
});
