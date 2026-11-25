// Story 1582 (#1582) — le titre de l'onglet porte le nom de la comptabilité ouverte.
// `titleWithCompany` est pure ; `watchTabTitle` observe `document.head` (jsdom).

import { describe, it, expect, afterEach, vi } from 'vitest';
import { titleWithCompany, watchTabTitle } from './tab-title';

const N = 'Dupont Sàrl';

describe('titleWithCompany (D5)', () => {
	const table: Array<[string, string, string]> = [
		['suffixe « - Kesh »', 'Accueil - Kesh', `Kesh — ${N} — Accueil`],
		['suffixe, page à plusieurs mots', 'Écritures comptables - Kesh', `Kesh — ${N} — Écritures comptables`],
		['suffixe « — Kesh » (cadratin)', 'Contacts — Kesh', `Kesh — ${N} — Contacts`],
		['préfixe « Kesh — »', 'Kesh — Rapports', `Kesh — ${N} — Rapports`],
		['demi-cadratin U+2013', 'Export global de vos données – Kesh', `Kesh — ${N} — Export global de vos données`],
		['tiret interne gardé', 'Paramètres — Facturation — Kesh', `Kesh — ${N} — Paramètres — Facturation`],
		['« Kesh » seul', 'Kesh', `Kesh — ${N}`],
		['titre vide', '', `Kesh — ${N}`],
		['titre sans « Kesh »', 'Erreur 404', `Kesh — ${N} — Erreur 404`],
	];
	for (const [label, raw, attendu] of table) {
		it(label, () => {
			expect(titleWithCompany(raw, N)).toBe(attendu);
		});
	}

	it('est idempotente : un seul nom, quel que soit le nombre de passes', () => {
		for (const raw of ['Accueil - Kesh', 'Kesh', '', 'Kesh — Rapports', 'Paramètres — Facturation — Kesh']) {
			const une = titleWithCompany(raw, N);
			expect(titleWithCompany(une, N)).toBe(une);
			expect(une.split(N).length - 1).toBe(1);
		}
	});

	it('nom contenant « — » et « Kesh »', () => {
		const nom = 'Atelier Kesh — SA';
		const une = titleWithCompany('Contacts - Kesh', nom);
		expect(une).toBe(`Kesh — ${nom} — Contacts`);
		expect(titleWithCompany(une, nom)).toBe(une);
	});

	it('le nom n’est jamais interprété comme un motif', () => {
		for (const nom of ['A$&B', 'X (Y) [Z]', '$1 + .* ^$']) {
			expect(titleWithCompany('Contacts - Kesh', nom)).toBe(`Kesh — ${nom} — Contacts`);
		}
	});
});

describe('watchTabTitle (D6)', () => {
	let stop: (() => void) | null = null;
	afterEach(() => {
		stop?.();
		stop = null;
		document.title = '';
	});

	/** Espion sur le setter de `document.title` : compte les écritures. */
	function espionner(): { count: () => number; restore: () => void } {
		const desc = Object.getOwnPropertyDescriptor(Document.prototype, 'title')!;
		let n = 0;
		Object.defineProperty(document, 'title', {
			configurable: true,
			get: () => desc.get!.call(document),
			set: (v: string) => {
				n++;
				desc.set!.call(document, v);
			},
		});
		return {
			count: () => n,
			restore: () => {
				delete (document as unknown as Record<string, unknown>).title;
			},
		};
	}

	it('décore le titre déjà présent à l’appel (premier apply)', () => {
		document.title = 'Accueil - Kesh';
		stop = watchTabTitle(N);
		expect(document.title).toBe(`Kesh — ${N} — Accueil`);
	});

	it('décore un titre posé ensuite par une page, puis un second', async () => {
		document.title = 'Kesh';
		stop = watchTabTitle(N);
		expect(document.title).toBe(`Kesh — ${N}`);
		document.title = 'Contacts — Kesh';
		await vi.waitFor(() => expect(document.title).toBe(`Kesh — ${N} — Contacts`));
		document.title = 'Écritures comptables - Kesh';
		await vi.waitFor(() => expect(document.title).toBe(`Kesh — ${N} — Écritures comptables`));
	});

	it('une écriture par changement de page : pas de boucle', async () => {
		document.title = 'Accueil - Kesh';
		const spy = espionner();
		try {
			stop = watchTabTitle(N);
			const apresMontage = spy.count();
			document.title = 'Contacts — Kesh'; // +1 (la page)
			await vi.waitFor(() => expect(document.title).toBe(`Kesh — ${N} — Contacts`));
			// Laisser passer d'éventuels tours d'observateur : le compte ne bouge plus.
			await new Promise((r) => setTimeout(r, 50));
			// la page (1) + la décoration (1)
			expect(spy.count() - apresMontage).toBe(2);
		} finally {
			stop?.();
			stop = null;
			spy.restore();
		}
	});

	it('la fonction de retour restaure le titre brut et déconnecte', async () => {
		document.title = 'Contacts — Kesh';
		stop = watchTabTitle(N);
		expect(document.title).toBe(`Kesh — ${N} — Contacts`);
		// Laisser l'observateur voir sa propre écriture : le titre brut mémorisé ne doit pas
		// devenir le titre décoré (garde anti-boucle, M3).
		await new Promise((r) => setTimeout(r, 50));
		stop();
		stop = null;
		expect(document.title).toBe('Contacts — Kesh');
		document.title = 'Autre - Kesh';
		await new Promise((r) => setTimeout(r, 50));
		expect(document.title).toBe('Autre - Kesh');
	});

	it('un titre posé par un tiers avant le nettoyage est laissé tel quel', () => {
		document.title = 'Contacts — Kesh';
		stop = watchTabTitle(N);
		document.title = 'Titre du tiers';
		stop();
		stop = null;
		expect(document.title).toBe('Titre du tiers');
	});

	it('un changement de nom (nettoyage puis nouvel appel) ne laisse pas deux noms', () => {
		document.title = 'Contacts — Kesh';
		const un = watchTabTitle('Société A');
		expect(document.title).toBe('Kesh — Société A — Contacts');
		un();
		stop = watchTabTitle('Société B');
		expect(document.title).toBe('Kesh — Société B — Contacts');
		expect(document.title).not.toContain('Société A');
	});
});
