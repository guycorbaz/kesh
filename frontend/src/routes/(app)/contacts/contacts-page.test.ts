/**
 * Preuves de composant des sondes anti-doublon — Story 22-2b (#301).
 *
 * ⚠️ **Chaque test nomme la MUTATION qu'il attrape.** Trois passes de revue en
 * prose de la Story 22-2 avaient laissé passer des preuves qui ne
 * discriminaient rien — dont une assertion « sur l'argument » **satisfaite par
 * l'appel que la page fait déjà au montage**. D'où la discipline de ce fichier :
 * on filtre les appels de la sonde sur `search`, et on assert l'objet EXACT.
 *
 * Patron : `products-page.test.ts` — mocks hoistés AVANT l'import du composant.
 */

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, fireEvent } from '@testing-library/svelte';
import type { ContactResponse } from '$lib/features/contacts/contacts.types';

vi.mock('$app/environment', () => ({ browser: true }));
vi.mock('$app/navigation', () => ({ goto: vi.fn() }));
vi.mock('$app/state', () => ({ page: { url: new URL('http://localhost/contacts') } }));

vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	// Les arguments sont interpolés comme le fait le vrai `i18nMsg`.
	i18nMsg: (_k: string, fallback: string, args?: Record<string, string | number>) =>
		args ? fallback.replace(/\{\s*\$(\w+)\s*\}/g, (_, n) => String(args[n] ?? '')) : fallback
}));

vi.mock('$lib/shared/utils/notify', () => ({
	notifyError: vi.fn(),
	notifySuccess: vi.fn()
}));

const listContactsMock = vi.fn();
const createContactMock = vi.hoisted(() => vi.fn());
// Story v0200-corrections-mineures (#1342, AC7) — la charge d'une MODIFICATION se lit
// (un contact étranger dont on retire l'adresse part avec `country: 'CH'`).
const updateContactMock = vi.hoisted(() => vi.fn());
vi.mock('$lib/features/contacts/contacts.api', () => ({
	listContacts: (q?: unknown) => listContactsMock(q),
	createContact: (p: unknown) => createContactMock(p),
	updateContact: (id: unknown, p: unknown) => updateContactMock(id, p),
	archiveContact: vi.fn()
}));

import Page from './+page.svelte';

// ---------------------------------------------------------------------------
// Fabriques et utilitaires
// ---------------------------------------------------------------------------

function c(partial: Partial<ContactResponse> & { id: number; name: string }): ContactResponse {
	return {
		companyId: 1,
		contactType: 'Entreprise',
		firstName: null,
		lastName: null,
		isClient: true,
		isSupplier: false,
		address: null,
		addressStructured: { street: '', building: '', postalCode: '', city: '', country: 'CH' },
		email: null,
		phone: null,
		ideNumber: null,
		clientNumber: null,
		defaultPaymentTerms: null,
		defaultPaymentTermsDays: null,
		language: null,
		salutation: 'Neutre',
		active: true,
		version: 1,
		createdAt: '',
		updatedAt: '',
		...partial
	} as unknown as ContactResponse;
}

const VIDE = { items: [], total: 0, limit: 20, offset: 0 };

/** Laisse passer `onMount`, les `$effect` et la temporisation de 300 ms. */
async function settle(ms = 400) {
	await new Promise((r) => setTimeout(r, ms));
}

/**
 * Les appels de la SONDE, isolés de ceux de la liste.
 *
 * ⚠️ **C'est la clé de tout ce fichier.** `toHaveBeenCalledWith(objectContaining(…))`
 * est satisfait par `loadContacts()`, que la page émet au montage avec
 * exactement `limit: 20` et `includeArchived: false` — et `vi.clearAllMocks()`
 * s'exécute AVANT le `render`. Une assertion non filtrée ne garde donc rien.
 */
function sondes(pred: (q: Record<string, unknown>) => boolean) {
	return listContactsMock.mock.calls
		.map(([q]) => q as Record<string, unknown>)
		// ⚠️ `q.search !== ''` est une DÉFENSE EN PROFONDEUR, pas une nécessité
		// actuelle : `loadContacts()` écrit `if (effectiveSearch.trim())
		// query.search = …`, donc l'appel de montage OMET la clé. Mais rien ne
		// garantit qu'il en sera toujours ainsi, et si quelqu'un le fait passer à
		// un `search: ''` inconditionnel, le test « aucune requête sous le seuil »
		// rougirait pour une raison SANS RAPPORT avec le seuil qu'il mesure.
		// Relevé en passe 3 de revue de code.
		.filter((q) => q && typeof q.search === 'string' && q.search !== '' && pred(q));
}

/**
 * Ouvre « Nouveau contact » et rend le formulaire.
 *
 * ⚠️ Les champs se désignent par leur **identifiant** (`#form-name`, `#form-ide`…)
 * et non par leur libellé : la page porte un « Type » de FILTRE et un « Type »
 * de FORMULAIRE, et `getByLabelText` ne les départage pas.
 */
async function ouvrirCreation() {
	const bouton = [...document.querySelectorAll('button')].find((b) =>
		/Nouveau contact/i.test(b.textContent ?? '')
	);
	if (!bouton) throw new Error('bouton « Nouveau contact » introuvable');
	await fireEvent.click(bouton);
	await settle(20);
}

/**
 * Un champ du formulaire, par son identifiant.
 *
 * ⚠️ On interroge le `document`, pas le conteneur du `render` : `Dialog.Content`
 * se rend dans un **portail** attaché au `body`, hors de l'arbre rendu.
 */
function champ(id: string): HTMLElement {
	const el = document.querySelector<HTMLElement>(`#${id}`);
	if (!el) throw new Error(`champ #${id} introuvable`);
	return el;
}

/** La zone d'avertissement, par son `data-testid` — toujours présente dans le DOM. */
function zone(testid: string): HTMLElement {
	const el = document.querySelector<HTMLElement>(`[data-testid="${testid}"]`);
	if (!el) throw new Error(`zone ${testid} introuvable`);
	return el;
}

/** Saisit une valeur dans un champ et laisse la temporisation s'écouler. */
async function saisir(id: string, valeur: string, ms = 400) {
	await fireEvent.input(champ(id), { target: { value: valeur } });
	await settle(ms);
}

/** Ouvre en ÉDITION la fiche du premier contact de la liste. */
async function ouvrirEdition() {
	const bouton = [...document.querySelectorAll('button')].find(
		(b) => b.getAttribute('aria-label') === 'Modifier'
	);
	if (!bouton) throw new Error('bouton « Modifier » introuvable');
	await fireEvent.click(bouton);
	await settle(20);
}

/** Répond à la sonde selon le terme cherché ; le reste rend une liste vide. */
function repondre(
	table: Record<string, { items: ContactResponse[]; total?: number } | 'rejette'>,
	liste: ContactResponse[] = []
) {
	listContactsMock.mockImplementation((q: Record<string, unknown>) => {
		if (typeof q?.search !== 'string' || q.search === '') {
			// L'appel de la LISTE, que la page émet au montage. ⚠️ Il OMET la clé
			// `search` — `loadContacts()` ne l'écrit que si le terme est non vide —
			// donc c'est le `typeof … !== 'string'` qui l'attrape, jamais le
			// `=== ''`. Ce dernier est gardé par symétrie avec `sondes()`, pas
			// parce qu'il se produirait : la rédaction précédente laissait croire
			// le contraire (relevé en passe 3 de revue de code).
			return Promise.resolve({ ...VIDE, items: liste, total: liste.length });
		}
		const r = table[q.search];
		if (!r) return Promise.resolve(VIDE);
		if (r === 'rejette') return Promise.reject(new Error('500'));
		return Promise.resolve({ ...VIDE, items: r.items, total: r.total ?? r.items.length });
	});
}

/** Ferme le dialogue de formulaire par « Annuler ». */
async function fermer() {
	const bouton = [...document.querySelectorAll('button')].find(
		(b) => (b.textContent ?? '').trim() === 'Annuler'
	);
	if (!bouton) throw new Error('bouton « Annuler » introuvable');
	await fireEvent.click(bouton);
	await settle(20);
}

/** Frappe SANS laisser la temporisation s'écouler — pour compter les départs. */
async function frapper(id: string, valeur: string) {
	await fireEvent.input(champ(id), { target: { value: valeur } });
}

/** Une promesse qu'on résout à la main, pour maîtriser l'ORDRE des réponses. */
function differe<T>() {
	let resoudre!: (v: T) => void;
	const promesse = new Promise<T>((r) => (resoudre = r));
	return { promesse, resoudre };
}

const JEAN_X = [
	c({ id: 1, name: 'Jean Bernard' }),
	c({ id: 2, name: 'Jean Dupont' }),
	c({ id: 3, name: 'Jean Favre' }),
	c({ id: 4, name: 'Jean Martin' }),
	c({ id: 5, name: 'Jean Rochat' }),
	c({ id: 6, name: 'Jean Zwahlen' })
];

beforeEach(() => {
	vi.clearAllMocks();
	listContactsMock.mockResolvedValue(VIDE);
});

afterEach(() => {
	vi.useRealTimers();
});

// ---------------------------------------------------------------------------
// AC-b1 — la sonde « nom proche »
// ---------------------------------------------------------------------------

describe('AC-b1 — sonde « nom proche »', () => {
	it('preuve 4 : l’argument de LA SONDE, isolé et EXACT', async () => {
		// MUTATION : « envoyer le terme brut » (`Coop-Vaud`) ⇒ rouvre #314 ; ou
		// « inverser includeArchived » ⇒ propositions polluées par des fiches
		// mortes. Les deux survivent à une assertion NON filtrée, satisfaite par
		// l'appel que la page émet au montage.
		render(Page);
		await settle(20);
		await ouvrirCreation();
		await saisir('form-name', 'Coop-Vaud');

		const appels = sondes((q) => q.search === 'Coop Vaud');
		expect(appels).toHaveLength(1);
		expect(appels[0]).toEqual({ search: 'Coop Vaud', limit: 20, includeArchived: false });
	});

	it('preuve 2 : une Personne arme sur prénom + nom', async () => {
		// MUTATION : « ne lire que la raison sociale » ⇒ dispositif entièrement
		// mort pour les personnes physiques, sans rien casser ni rougir.
		render(Page);
		await settle(20);
		await ouvrirCreation();

		await fireEvent.change(champ('form-type'), { target: { value: 'Personne' } });
		await settle(20);
		await fireEvent.input(champ('form-firstname'), { target: { value: 'Jean' } });
		await saisir('form-lastname', 'Dupont');

		expect(sondes((q) => q.search === 'Jean Dupont')).toHaveLength(1);
	});

	it('preuve 2-bis : le champ PRÉNOM réarme à lui seul', async () => {
		// MUTATION : « retirer `oninput` du champ prénom ».
		//
		// ⚠️ La preuve 2 la laisse SURVIVRE, et c'est le banc qui l'a montré —
		// pas la relecture, qui avait conclu l'inverse. Elle frappe les DEUX
		// champs : le câblage restant suffit à composer le même terme, parce que
		// la minuterie n'expire qu'APRÈS la seconde frappe. Elle passe par
		// coïncidence de calendrier, pas par discrimination.
		//
		// Un seul champ frappé, donc, pour que le câblage de CE champ soit la
		// seule cause possible du départ.
		render(Page);
		await settle(20);
		await ouvrirCreation();

		await fireEvent.change(champ('form-type'), { target: { value: 'Personne' } });
		// ⚠️ Laisser la sonde du CHANGEMENT DE TYPE partir ET aboutir avant de
		// compter. Sans cela, elle expire 300 ms plus tard en lisant le champ
		// qu'on vient de remplir, et c'est ELLE qu'on mesure — le banc a montré
		// que la mutation survivait, cette preuve tombant dans le piège même
		// qu'elle était censée fermer.
		await settle(400);
		listContactsMock.mockClear();
		await saisir('form-firstname', 'Jeanne');

		expect(sondes((q) => q.search === 'Jeanne')).toHaveLength(1);
	});

	it('preuve 2-ter : le champ NOM DE FAMILLE réarme à lui seul', async () => {
		// MUTATION : « retirer `oninput` du champ nom de famille ». Même angle
		// mort, même remède : un seul champ frappé.
		render(Page);
		await settle(20);
		await ouvrirCreation();

		await fireEvent.change(champ('form-type'), { target: { value: 'Personne' } });
		// ⚠️ Laisser la sonde du CHANGEMENT DE TYPE partir ET aboutir avant de
		// compter. Sans cela, elle expire 300 ms plus tard en lisant le champ
		// qu'on vient de remplir, et c'est ELLE qu'on mesure — le banc a montré
		// que la mutation survivait, cette preuve tombant dans le piège même
		// qu'elle était censée fermer.
		await settle(400);
		listContactsMock.mockClear();
		await saisir('form-lastname', 'Rochat');

		expect(sondes((q) => q.search === 'Rochat')).toHaveLength(1);
	});

	it('preuve 1 : les trois éléments d’affichage sont rendus', async () => {
		// MUTATION : « ne rendre que c.name » ⇒ supprime le mécanisme de
		// discrimination de la story (D-b10) sans faire rougir rien d'autre.
		repondre({
			Dubarde: {
				items: [
					c({
						id: 9,
						name: 'Dubarde Sàrl',
						clientNumber: 'CLI-42',
						addressStructured: {
							street: '',
							building: '',
							postalCode: '',
							city: 'Lausanne',
							country: 'CH'
						}
					})
				]
			}
		});
		render(Page);
		await settle(20);
		await ouvrirCreation();
		await saisir('form-name', 'Dubarde');

		const texte = zone('contact-duplicate-nearby').textContent ?? '';
		expect(texte).toContain('Dubarde Sàrl');
		expect(texte).toContain('Lausanne');
		expect(texte).toContain('CLI-42');
	});

	it('preuve 3 : sans adresse ni numéro, l’email sert à reconnaître', async () => {
		repondre({
			Dubarde: { items: [c({ id: 9, name: 'Dubarde Sàrl', email: 'a@dubarde.ch' })] }
		});
		render(Page);
		await settle(20);
		await ouvrirCreation();
		await saisir('form-name', 'Dubarde');

		const texte = zone('contact-duplicate-nearby').textContent ?? '';
		expect(texte).toContain('a@dubarde.ch');
		expect(texte).not.toMatch(/—\s*—/); // aucun tiret orphelin
	});

	it('preuve 6 : la chaîne de repli ÉPUISÉE distingue quand même deux homonymes', async () => {
		// MUTATION : « s'arrêter à l'email » ⇒ deux lignes RIGOUREUSEMENT
		// identiques, ce que l'invariant de D-b10 interdit. Le père et le fils,
		// même ménage, sans numéro ni email.
		repondre({
			Dupont: {
				items: [c({ id: 11, name: 'Jean Dupont' }), c({ id: 12, name: 'Jean Dupont' })]
			}
		});
		render(Page);
		await settle(20);
		await ouvrirCreation();
		await saisir('form-name', 'Dupont');

		const lignes = [...zone('contact-duplicate-nearby').querySelectorAll('li')].map(
			(li) => li.textContent
		);
		expect(lignes).toHaveLength(2);
		expect(lignes[0]).not.toBe(lignes[1]);
		expect(lignes.join()).toContain('#11');
	});

	it('preuve 7 : la bascule de type RÉARME la sonde', async () => {
		// MUTATION : « ne recalculer que sur les champs de nom » ⇒ un avertissement
		// calculé sur l'ancien type subsiste sous un champ retiré du DOM.
		repondre({ 'Dubarde Sàrl': { items: [c({ id: 9, name: 'Dubarde Sàrl' })] } });
		render(Page);
		await settle(20);
		await ouvrirCreation();
		await saisir('form-name', 'Dubarde Sàrl');
		expect(zone('contact-duplicate-nearby').textContent).toContain('Dubarde Sàrl');

		// Bascule sans rien effacer : prénom et nom sont vides ⇒ terme vide.
		await fireEvent.change(champ('form-type'), { target: { value: 'Personne' } });
		await settle();
		expect(zone('contact-duplicate-nearby').textContent?.trim()).toBe('');
	});

	it('preuve 8 : classé, coupé à CINQ, et compté', async () => {
		// TROIS mutations d'un caractère, qu'aucune autre preuve n'attrape :
		// « retirer rank(…) » ⇒ ordre alphabétique du SQL, Zwahlen évincé ;
		// « proches = retenus » ⇒ vingt propositions ; « ne pas rendre le
		// compteur » ⇒ sa seule autre mention asserte son ABSENCE.
		repondre({
			'Jean Zwahlen': {
				items: [...JEAN_X, c({ id: 7, name: 'Jean Aebi' }), c({ id: 8, name: 'Jean Berset' })],
				total: 8
			}
		});
		render(Page);
		await settle(20);
		await ouvrirCreation();
		await saisir('form-name', 'Jean Zwahlen');

		const z = zone('contact-duplicate-nearby');
		const lignes = [...z.querySelectorAll('li')];
		expect(lignes).toHaveLength(5);
		expect(lignes[0].textContent).toContain('Jean Zwahlen');
		expect(z.textContent).toContain('et 3 autres');
	});

	it('preuve 5 : une proposition est INERTE — cliquer ne touche pas le formulaire', async () => {
		// MUTATION : `onclick={() => openEdit(c)}` — un ajout de bonne foi qui
		// compile et EFFACE les dix-neuf champs de la saisie en cours (D-b9).
		repondre({ Dubarde: { items: [c({ id: 9, name: 'Dubarde Sàrl' })] } });
		render(Page);
		await settle(20);
		await ouvrirCreation();
		await saisir('form-name', 'Dubarde');

		await fireEvent.click(zone('contact-duplicate-nearby').querySelector('li')!);
		await settle(50);
		expect((champ('form-name') as HTMLInputElement).value).toBe('Dubarde');
	});
});

// ---------------------------------------------------------------------------
// AC-b2 — la sonde IDE
// ---------------------------------------------------------------------------

const PORTEUR = c({ id: 7, name: 'Dubarde Vins SA', ideNumber: 'CHE123456788' });
const PORTEUR_ARCHIVE = c({
	id: 8,
	name: 'Ancienne Maison SA',
	ideNumber: 'CHE109322551',
	active: false
});

describe('AC-b2 — sonde IDE', () => {
	it('preuve 1 : un porteur ACTIF déclenche l’avertissement franc', async () => {
		repondre({ CHE123456788: { items: [PORTEUR] } });
		render(Page);
		await settle(20);
		await ouvrirCreation();
		await saisir('form-ide', 'CHE-123.456.788');

		expect(zone('contact-duplicate-ide').textContent).toContain('Dubarde Vins SA');
	});

	it('preuve 2 : l’argument est la forme NORMALISÉE, et le porteur archivé a son propre message', async () => {
		// MUTATION : `search: formIde` ⇒ la sonde envoie `CHE-109.322.551`, que le
		// LIKE ne matche pas contre une colonne stockée normalisée. Elle serait
		// MUETTE pour tout utilisateur réel — l'UI affichant la forme séparée —
		// et l'E2E ne l'attraperait pas s'il tape la forme déjà normalisée.
		repondre({ CHE109322551: { items: [PORTEUR_ARCHIVE] } });
		render(Page);
		await settle(20);
		await ouvrirCreation();
		await saisir('form-ide', 'CHE-109.322.551');

		const appels = sondes((q) => q.includeArchived === true);
		expect(appels).toHaveLength(1);
		expect(appels[0]).toEqual({ search: 'CHE109322551', limit: 20, includeArchived: true });

		// Le libellé du porteur ARCHIVÉ est distinct : c'est lui, le recours.
		expect(zone('contact-duplicate-ide').textContent).toContain('archivé');
	});

	it('preuve 3 : un contact remonté SANS porter cet IDE ne déclenche rien', async () => {
		repondre({ CHE123456788: { items: [c({ id: 9, name: 'CHE123456788 dans le nom' })] } });
		render(Page);
		await settle(20);
		await ouvrirCreation();
		await saisir('form-ide', 'CHE-123.456.788');

		expect(zone('contact-duplicate-ide').textContent?.trim()).toBe('');
	});

	it('preuve 5 : le retrait — corriger vers un IDE LIBRE efface l’avertissement', async () => {
		// MUTATION : « n'écrire l'avertissement que dans la branche `if (holder)` »
		// ⇒ l'ancien message reste, désignant un numéro qu'on ne porte plus.
		repondre({ CHE123456788: { items: [PORTEUR] } });
		render(Page);
		await settle(20);
		await ouvrirCreation();
		await saisir('form-ide', 'CHE-123.456.788');
		expect(zone('contact-duplicate-ide').textContent).toContain('Dubarde Vins SA');

		await saisir('form-ide', 'CHE-109.322.551');
		expect(zone('contact-duplicate-ide').textContent?.trim()).toBe('');
	});
});

// ---------------------------------------------------------------------------
// AC-b5 — muet quand rien à dire
// ---------------------------------------------------------------------------

describe('AC-b5 — le dispositif est muet quand il n’a rien à dire', () => {
	it('(a) sous le seuil : AUCUNE requête', async () => {
		render(Page);
		await settle(20);
		await ouvrirCreation();
		await saisir('form-name', 'Du');

		expect(sondes(() => true)).toHaveLength(0);
	});

	it('(b) armée, carnet vide : aucun TEXTE — mais la zone reste dans le DOM', async () => {
		// ⚠️ Asserter l'absence de TEXTE, jamais celle du NŒUD : les zones sont
		// permanentes pour qu'`aria-live` fonctionne.
		render(Page);
		await settle(20);
		await ouvrirCreation();
		await saisir('form-name', 'Dubarde');

		expect(zone('contact-duplicate-nearby')).toBeTruthy();
		expect(zone('contact-duplicate-nearby').textContent?.trim()).toBe('');
	});

	it('(b-bis) DÉSARMER, C’EST EFFACER — repasser sous le seuil vide les propositions', async () => {
		// MUTATION : « sortir par un `return` nu » ⇒ l'avertissement reste à
		// l'écran sur un terme que l'utilisateur a déjà effacé.
		repondre({ Dubarde: { items: [c({ id: 9, name: 'Dubarde Sàrl' })] } });
		render(Page);
		await settle(20);
		await ouvrirCreation();
		await saisir('form-name', 'Dubarde');
		expect(zone('contact-duplicate-nearby').textContent).toContain('Dubarde Sàrl');

		await saisir('form-name', 'Du');
		expect(zone('contact-duplicate-nearby').textContent?.trim()).toBe('');
	});

	it('(b-bis) vider le champ IDE retire l’avertissement franc', async () => {
		repondre({ CHE123456788: { items: [PORTEUR] } });
		render(Page);
		await settle(20);
		await ouvrirCreation();
		await saisir('form-ide', 'CHE-123.456.788');
		expect(zone('contact-duplicate-ide').textContent).toContain('Dubarde Vins SA');

		await saisir('form-ide', '');
		expect(zone('contact-duplicate-ide').textContent?.trim()).toBe('');
	});

	it('(b-ter) DÉSARMER pendant qu’une réponse est EN VOL — elle ne repeuple rien', async () => {
		// MUTATION ATOMIQUE : « garder l'effacement, retirer le `nameSeq++` » de la
		// branche de désarmement.
		//
		// ⚠️ Le doc-comment annonce DEUX conséquences d'un `return` nu — laisser des
		// propositions périmées à l'écran, ET laisser une réponse en vol les
		// repeupler. Seule la première était éprouvée : (b-bis) laisse la réponse
		// aboutir AVANT de repasser sous le seuil. Le banc ne jouait que la mutation
		// composite, qui retire les trois lignes d'un coup. Relevé en passe 4.
		//
		// Le cas est atteignable sans acrobatie : requête plus lente que 300 ms,
		// l'utilisateur efface, la réponse périmée revient — et sans le `nameSeq++`,
		// `seq === nameSeq` tient encore et l'avertissement se rallume sur un terme
		// que l'utilisateur a supprimé.
		const lente = differe<{ items: ContactResponse[]; total: number }>();
		listContactsMock.mockImplementation((q: Record<string, unknown>) => {
			if (q?.search === 'Dubarde') return lente.promesse;
			return Promise.resolve(VIDE);
		});
		render(Page);
		await settle(20);
		await ouvrirCreation();

		await saisir('form-name', 'Dubarde');
		// La réponse n'est PAS encore arrivée : on désarme en repassant sous le seuil.
		await saisir('form-name', 'Du');
		// …et seulement MAINTENANT elle aboutit.
		lente.resoudre({ items: [c({ id: 95, name: 'Dubarde SA' })], total: 1 });
		await settle(50);

		expect(zone('contact-duplicate-nearby').textContent?.trim() ?? '').toBe('');
	});

	it('(c-bis) la sonde NOM qui échoue efface ses propositions, et l’IDE continue', async () => {
		// MUTATION : « try/catch inopérant sur la sonde NOM ».
		//
		// ⚠️ Le test (c) ci-dessous ne fait échouer que la sonde IDE : la garde
		// jumelle, côté nom, n'était NI testée NI mutée — relevé en revue de code.
		// Deux gardes symétriques dont une seule est éprouvée, c'est une garde et
		// une croyance.
		//
		// Il faut AFFICHER avant d'échouer : une promesse rejetée sur un écran
		// déjà vide ne distingue pas « effacé » de « jamais rempli ».
		const porteur = c({ id: 90, name: 'Porteur SA', ideNumber: 'CHE109322551' });
		repondre({
			Dubarde: { items: [c({ id: 91, name: 'Dubarde SA' })], total: 1 },
			Dubardex: 'rejette',
			CHE109322551: { items: [porteur], total: 1 }
		});
		render(Page);
		await settle(20);
		await ouvrirCreation();

		await saisir('form-name', 'Dubarde');
		expect(zone('contact-duplicate-nearby').textContent ?? '').toContain('Dubarde SA');

		await saisir('form-ide', 'CHE-109.322.551');
		await saisir('form-name', 'Dubardex');

		// La sonde nom a échoué : elle se tait, et n'a laissé aucune trace.
		expect(zone('contact-duplicate-nearby').textContent?.trim() ?? '').toBe('');
		// L'autre sonde, elle, continue de dire ce qu'elle sait.
		expect(zone('contact-duplicate-ide').textContent ?? '').toContain('Porteur SA');
	});

	it('(c) une sonde qui ÉCHOUE efface son avertissement, et l’autre continue', async () => {
		// MUTATION : « retirer le try/catch » ⇒ la promesse rejetée laisse
		// `ideHolder` à sa valeur PRÉCÉDENTE, et l'avertissement d'un IDE qu'on ne
		// porte plus reste à l'écran.
		// ⚠️ Une première rédaction assertait seulement « rien ne s'affiche » sur
		// une sonde vierge : sans `try/catch`, rien ne s'affichait non plus, et la
		// mutation SURVIVAIT. Il faut donc afficher d'abord, PUIS échouer.
		repondre({
			CHE123456788: { items: [PORTEUR] },
			CHE109322551: 'rejette',
			Dubarde: { items: [c({ id: 9, name: 'Dubarde Sàrl' })] }
		});
		render(Page);
		await settle(20);
		await ouvrirCreation();
		await saisir('form-ide', 'CHE-123.456.788');
		expect(zone('contact-duplicate-ide').textContent).toContain('Dubarde Vins SA');

		await saisir('form-ide', 'CHE-109.322.551'); // celle-ci échoue
		await saisir('form-name', 'Dubarde');

		expect(zone('contact-duplicate-ide').textContent?.trim()).toBe('');
		expect(zone('contact-duplicate-nearby').textContent).toContain('Dubarde Sàrl');
	});
});

// ---------------------------------------------------------------------------
// AC-b7 — l’édition, sans se signaler soi-même
// ---------------------------------------------------------------------------

describe('AC-b7 — l’édition ne se signale pas elle-même', () => {
	const MOI = c({ id: 42, name: 'Dubarde Sàrl', ideNumber: 'CHE123456788' });

	it('preuve 1 : le contact édité ne figure pas dans ses propres propositions', async () => {
		// MUTATION : ne pas passer `editing?.id ?? null` à `excludeSelf`.
		repondre({ 'Dubarde Sàrl SA': { items: [MOI], total: 1 } }, [MOI]);
		render(Page);
		await settle(30);
		await ouvrirEdition();
		await saisir('form-name', 'Dubarde Sàrl SA');

		expect(zone('contact-duplicate-nearby').querySelectorAll('li')).toHaveLength(0);
	});

	it('preuve 2 : le compteur « et N autres » SOUSTRAIT le contact édité', async () => {
		// MUTATION : `total − affiches.length`, sans soustraire soi.
		// ⚠️ **Il faut des propositions AFFICHÉES**, sans quoi le bloc du compteur
		// n'est pas rendu du tout et la mutation reste invisible. Trouvé en jouant
		// le banc : la première rédaction, sur une liste vide, SURVIVAIT.
		// Ici : total 3 = soi + 2 autres ⇒ 2 affichés, et 3 − 1 − 2 = 0.
		repondre(
			{
				'Dubarde Sàrl SA': {
					items: [MOI, c({ id: 51, name: 'Dubarde Bois SA' }), c({ id: 52, name: 'Dubarde Vin' })],
					total: 3
				}
			},
			[MOI]
		);
		render(Page);
		await settle(30);
		await ouvrirEdition();
		await saisir('form-name', 'Dubarde Sàrl SA');

		const z = zone('contact-duplicate-nearby');
		expect(z.querySelectorAll('li')).toHaveLength(2);
		expect(z.textContent).not.toContain('autre');
	});

	it('AC-b2 preuve 4 : RETAPER son propre IDE en édition ne crie pas sur soi-même', async () => {
		// MUTATION : retirer la garde `c.id !== editingId` de `findIdeHolder`.
		// ⚠️ **Il FAUT retaper le champ.** Une première rédaction se contentait
		// d'ouvrir la fiche : la sonde ne partait jamais, le test passait pour la
		// mauvaise raison, et la mutation SURVIVAIT — trouvée en jouant le banc.
		repondre({ CHE123456788: { items: [MOI], total: 1 } }, [MOI]);
		render(Page);
		await settle(30);
		await ouvrirEdition();
		await saisir('form-ide', 'CHE-123.456.788');

		expect(zone('contact-duplicate-ide').textContent?.trim()).toBe('');
	});
});

// ---------------------------------------------------------------------------
// AC-b3 — les sondes ne changent RIEN à l’état du bouton
// ---------------------------------------------------------------------------

describe('AC-b3 — l’état du bouton est celui qu’il aurait sans les sondes', () => {
	it('un avertissement affiché sur un état PAR AILLEURS INVALIDE ne change pas le `disabled`', async () => {
		// ⚠️ État COMPOSÉ : une garde préexistante (adresse partielle) désactive
		// déjà le bouton. La sonde ne doit ni le réactiver, ni le désactiver
		// davantage — c'est une propriété DIFFÉRENTIELLE.
		repondre({ Dubarde: { items: [c({ id: 9, name: 'Dubarde Sàrl' })] } });
		render(Page);
		await settle(20);
		await ouvrirCreation();
		await saisir('form-name', 'Dubarde');
		await fireEvent.input(champ('form-street'), { target: { value: 'Rue du Lac 3' } });
		await settle(50);

		const enregistrer = [...document.querySelectorAll('button')].find(
			(b) => b.getAttribute('type') === 'submit'
		) as HTMLButtonElement;
		expect(zone('contact-duplicate-nearby').textContent).toContain('Dubarde Sàrl');
		expect(enregistrer.disabled).toBe(true); // à cause de l'ADRESSE, pas de la sonde

		// ⚠️ **Le MESSAGE rendu est celui de l'adresse**, pas un propos de doublon.
		// Sans cette assertion, une mutation qui écrirait une phrase de doublon
		// dans `formValidation` laisserait `disabled` à `true` et cette preuve
		// VERTE — or c'est exactement le geste que l'AC dit vouloir empêcher.
		// Le mot `formValidation` n'apparaissait pas une seule fois dans les
		// tests ; relevé en passe 2 de revue de code.
		//
		// ⚠️ Depuis #1349 (story v0200-corrections-mineures, D4), `formValidation`
		// s'affiche aussi AU-DESSUS DES BOUTONS, en ligne discrète
		// (`contact-form-blocked-reason`, `text-muted-foreground`). On SOUMET quand
		// même, pour lire le `formError` qu'en fait `submitForm` — le paragraphe
		// `text-destructive` ci-dessous —, ce que l'utilisateur se verrait
		// reprocher s'il atteignait la soumission.
		const formulaire = document.querySelector('form') as HTMLFormElement;
		await fireEvent.submit(formulaire);
		await settle(30);

		// ⚠️ On lit le PARAGRAPHE D'ERREUR, pas `document.body` : la zone
		// d'avertissement porte le titre « Contacts **déjà enregistrés** qui
		// pourraient correspondre », qu'une assertion sur le corps entier
		// confondrait avec une fuite dans `formValidation`. Première rédaction de
		// cette preuve, corrigée sur son propre échec.
		const erreur = document.querySelector('p.text-destructive');
		expect(erreur?.textContent).toBe('NPA et localité obligatoires si une adresse est saisie');
	});

	it('un avertissement sur un formulaire PAR AILLEURS VALIDE laisse le bouton ACTIF', async () => {
		// MUTATION : « disabled={… || proches.length > 0} », ou un `if (ideHolder)
		// return;` en tête de soumission.
		//
		// ⚠️ La preuve ci-dessus ne peut détecter qu'UN sens — une sonde qui
		// RÉACTIVERAIT le bouton — puisqu'elle part d'un état déjà invalide. La
		// mutation qui détruit la promesse centrale de la story, « il n'empêche
		// jamais d'enregistrer », la laisse verte. C'est l'autre moitié.
		const porteur = c({ id: 11, name: 'Porteur SA', ideNumber: 'CHE109322551' });
		repondre({
			Dubarde: { items: [c({ id: 10, name: 'Dubarde Sàrl' })], total: 1 },
			CHE109322551: { items: [porteur], total: 1 }
		});
		render(Page);
		await settle(20);
		await ouvrirCreation();
		await saisir('form-name', 'Dubarde');
		await saisir('form-ide', 'CHE-109.322.551');

		const enregistrer = [...document.querySelectorAll('button')].find(
			(b) => b.getAttribute('type') === 'submit'
		) as HTMLButtonElement;
		// Les DEUX avertissements sont à l'écran…
		expect(zone('contact-duplicate-nearby').textContent).toContain('Dubarde Sàrl');
		expect(zone('contact-duplicate-ide').textContent).toContain('Porteur SA');
		// …et le bouton reste actif. C'est toute la story en une assertion.
		expect(enregistrer.disabled).toBe(false);
	});
});

// ---------------------------------------------------------------------------
// AC-b4 — la temporisation, la garde d'ordre, et la remise à zéro
// ---------------------------------------------------------------------------

describe('AC-b4 — rien ne part avant que la frappe se calme', () => {
	it('preuve 1 : vingt caractères frappés d’affilée ne partent qu’UNE fois', async () => {
		// MUTATION : « ne pas annuler la minuterie précédente » (retirer le
		// `clearTimeout` de `scheduleNameProbe`) ⇒ vingt départs au lieu d'un.
		repondre({});
		render(Page);
		await settle(20);
		await ouvrirCreation();
		listContactsMock.mockClear();

		const mot = 'Dubarde Sarl SA XYZW';
		for (let i = 1; i <= mot.length; i++) await frapper('form-name', mot.slice(0, i));
		await settle(400);

		expect(sondes((q) => q.includeArchived === false).length).toBe(1);
	});

	it('preuve 2 : deux réponses résolues DANS L’ORDRE INVERSE — la périmée est ignorée', async () => {
		// MUTATION : « retirer la garde `if (seq !== nameSeq) return` » ⇒ la
		// réponse du terme ABANDONNÉ écrase celle du terme courant, et l'écran
		// affiche des propositions qui ne correspondent plus à ce qui est tapé.
		const lente = differe<{ items: ContactResponse[]; total: number }>();
		const rapide = differe<{ items: ContactResponse[]; total: number }>();
		listContactsMock.mockImplementation((q: Record<string, unknown>) => {
			if (q?.search === 'Ancien') return lente.promesse;
			if (q?.search === 'Nouveau') return rapide.promesse;
			return Promise.resolve(VIDE);
		});
		render(Page);
		await settle(20);
		await ouvrirCreation();

		await saisir('form-name', 'Ancien');
		await saisir('form-name', 'Nouveau');

		// La SECONDE aboutit d'abord…
		rapide.resoudre({ items: [c({ id: 50, name: 'Nouveau Contact' })], total: 1 });
		await settle(30);
		// …puis la PREMIÈRE, périmée, revient en retard.
		lente.resoudre({ items: [c({ id: 99, name: 'Ancien Contact' })], total: 1 });
		await settle(30);

		const texte = zone('contact-duplicate-nearby').textContent ?? '';
		expect(texte).toContain('Nouveau Contact');
		expect(texte).not.toContain('Ancien Contact');
	});

	it('preuve 3 : le test CROISÉ — les deux sondes aboutissent, chacune la sienne', async () => {
		// MUTATION : « une seule paire (minuterie, compteur) partagée » (D-b7).
		// Les preuves 1 et 2 la laissent VERTE : elles n'arment qu'une sonde.
		// Avec une paire unique, armer la seconde incrémente le compteur commun
		// et fait jeter la réponse de la première — un des deux avertissements
		// ne s'affiche jamais.
		const porteur = c({ id: 60, name: 'Porteur SA', ideNumber: 'CHE109322551' });
		repondre({
			'Dubarde SA': { items: [c({ id: 61, name: 'Dubarde SA' })], total: 1 },
			CHE109322551: { items: [porteur], total: 1 }
		});
		render(Page);
		await settle(20);
		await ouvrirCreation();

		// Armer la sonde nom, puis la sonde IDE AVANT que la première ait rendu.
		await frapper('form-name', 'Dubarde SA');
		await frapper('form-ide', 'CHE-109.322.551');
		await settle(500);

		expect(zone('contact-duplicate-nearby').textContent ?? '').toContain('Dubarde SA');
		expect(zone('contact-duplicate-ide').textContent ?? '').toContain('Porteur SA');
	});

	it('preuve 4 : valide → invalide → valide ⇒ DEUX départs, un par passage à la validité', async () => {
		// MUTATION : « sonder sans vérifier le format » ⇒ un départ à chaque
		// frappe intermédiaire, dont celles qui ne forment pas un IDE.
		repondre({});
		render(Page);
		await settle(20);
		await ouvrirCreation();
		listContactsMock.mockClear();

		await saisir('form-ide', 'CHE-109.322.551');
		await saisir('form-ide', 'CHE-109.322.55');
		await saisir('form-ide', 'CHE-123.456.788');

		expect(sondes((q) => q.includeArchived === true).length).toBe(2);
	});

	it('preuve 7 : FERMER le formulaire fait taire une sonde en vol', async () => {
		// MUTATION : « retirer le $effect qui réinitialise à la fermeture ».
		//
		// La garde de génération empêchait déjà la réponse de s'afficher, donc
		// rien de faux n'était montré — mais la requête partait quand même, pour
		// un dialogue quitté. Une temporisation qui laisse filer sa requête après
		// la fermeture ne temporise qu'à moitié.
		repondre({ Dubarde: { items: [c({ id: 75, name: 'Dubarde SA' })], total: 1 } });
		render(Page);
		await settle(20);
		await ouvrirCreation();

		await frapper('form-name', 'Dubarde');
		listContactsMock.mockClear();
		await fermer();
		await settle(500);

		expect(sondes((q) => q.search === 'Dubarde')).toHaveLength(0);
	});

	it('preuve 5 : rouvrir en CRÉATION repart d’un écran vierge', async () => {
		// MUTATION : « retirer le `$effect` de fermeture ».
		//
		// ⚠️ Cette preuve annonçait « ne rien réinitialiser dans openCreate() » —
		// une mutation qui n'a PLUS DE SITE : cet appel a été supprimé comme
		// redondant, et c'est le `$effect` sur la fermeture qui fait tout le
		// travail. Un commentaire décrivant du code disparu est pire qu'un
		// commentaire absent : il envoie chercher une garde là où il n'y en a
		// plus. Relevé en passe 2 de revue de code.
		repondre({ Dubarde: { items: [c({ id: 70, name: 'Dubarde SA' })], total: 1 } });
		render(Page);
		await settle(20);
		await ouvrirCreation();
		await saisir('form-name', 'Dubarde');
		expect(zone('contact-duplicate-nearby').textContent ?? '').toContain('Dubarde SA');

		await fermer();
		await ouvrirCreation();

		expect(zone('contact-duplicate-nearby').textContent?.trim() ?? '').toBe('');
	});

	it('preuve 6 : rouvrir en ÉDITION repart d’un écran vierge, lui aussi', async () => {
		// MUTATION : « retirer le `$effect` de fermeture » — la même que la preuve
		// 5, et c'est voulu : un seul mécanisme, trois conséquences gardées
		// séparément.
		//
		// ⚠️ Cette preuve annonçait « D-b8 exige les DEUX sites » — une exigence
		// que le code ne tient plus, et qui invitait un mainteneur à
		// RÉINTRODUIRE la redondance qu'on venait de démonter.
		//
		// ⚠️ Ce cas avait été identifié en passe 1 de validate, inscrit au tableau
		// des pièges muets — et la preuve n'avait pas été écrite. Le symptôme est
		// concret : les propositions d'une session « Créer » restent affichées sur
		// la fiche de quelqu'un d'autre, en le désignant comme un doublon.
		const autre = c({ id: 80, name: 'Autre Société' });
		repondre({ Dubarde: { items: [c({ id: 70, name: 'Dubarde SA' })], total: 1 } }, [autre]);
		render(Page);
		await settle(20);
		await ouvrirCreation();
		await saisir('form-name', 'Dubarde');
		expect(zone('contact-duplicate-nearby').textContent ?? '').toContain('Dubarde SA');

		await fermer();
		await ouvrirEdition();

		expect(zone('contact-duplicate-nearby').textContent?.trim() ?? '').toBe('');
	});
});

// ---------------------------------------------------------------------------
// Story v0174-correctifs-3 (#817, AC-3.14, AC-3.15) — le pays inconnu se signale
// ---------------------------------------------------------------------------

describe('#817 — le pays d’une adresse', () => {
	const ADRESSE = { street: 'Rue des Essais', building: '7', postalCode: '1200', city: 'Genève' };
	const avis = () => document.querySelector('[data-testid="contact-country-unknown"]');

	it('AC-3.14 : un contact au pays « SU » inconnu affiche l’avis, qui le nomme (MC-817)', async () => {
		const ancien = c({
			id: 31,
			name: 'Ancien SU',
			addressStructured: { ...ADRESSE, country: 'SU' },
			addressCountryKnown: false
		} as Partial<ContactResponse> & { id: number; name: string });
		repondre({}, [ancien]);
		render(Page);
		await settle(30);
		await ouvrirEdition();
		expect(avis()?.textContent).toContain('« SU »');
		expect(avis()?.textContent).toContain('QR-facture');
		// Revue P1 (E-1) : retapé en minuscules, c'est le même pays inconnu (le serveur le
		// tolère et le garde) : l'avis reste.
		await saisir('form-country', 'su', 20);
		expect(avis()?.textContent).toContain('« SU »');
		// Le champ changé : le serveur juge la nouvelle valeur, l'avis disparaît.
		await saisir('form-country', 'CH', 20);
		expect(avis()).toBeNull();
	});

	it('AC-3.14 : un pays connu n’affiche aucun avis', async () => {
		const connu = c({
			id: 32,
			name: 'Connu CH',
			addressStructured: { ...ADRESSE, country: 'CH' },
			addressCountryKnown: true
		} as Partial<ContactResponse> & { id: number; name: string });
		repondre({}, [connu]);
		render(Page);
		await settle(30);
		await ouvrirEdition();
		expect(avis()).toBeNull();
	});

	it('AC-3.15 : le refus du serveur pour un pays « XX » s’affiche tel quel', async () => {
		const refus = 'adresse : le pays « XX » n’est pas un code ISO 3166-1 alpha-2 connu (ex. CH, FR, DE)';
		createContactMock.mockRejectedValue({ code: 'VALIDATION_ERROR', status: 400, message: refus });
		repondre({});
		render(Page);
		await settle(20);
		await ouvrirCreation();
		await saisir('form-name', 'Essai XX', 20);
		await saisir('form-street', ADRESSE.street, 20);
		await saisir('form-npa', ADRESSE.postalCode, 20);
		await saisir('form-city', ADRESSE.city, 20);
		await saisir('form-country', 'XX', 20);
		const form = champ('form-name').closest('form') as HTMLFormElement;
		await fireEvent.submit(form);
		await settle(30);
		expect(createContactMock).toHaveBeenCalled();
		expect(document.body.textContent).toContain(refus);
	});
});

// ---------------------------------------------------------------------------
// Story 48-15 (#1331, #1334, AC4, AC7, AC8) — un texte trop long est refusé en
// le disant, jamais coupé en silence par un `maxlength`
// ---------------------------------------------------------------------------

describe('48-15 — aucun champ du contact ne coupe un texte collé', () => {
	/**
	 * Le champ est d'abord TROUVÉ (`champ()` lève sinon : une assertion d'absence
	 * sur un sélecteur vide passerait à vide), puis on exige qu'il ne porte aucun
	 * `maxlength`. ⚠️ C'est l'attribut qui porte le rouge : dans jsdom, un
	 * `maxlength` ne contraint pas une valeur posée par programme.
	 */
	function sansMaxlength(id: string) {
		const el = champ(id);
		expect(el.tagName).toBe('INPUT');
		expect(el.hasAttribute('maxlength')).toBe(false);
	}

	async function ouvrirEn(type: 'Entreprise' | 'Personne') {
		repondre({});
		render(Page);
		await settle(20);
		await ouvrirCreation();
		if (type === 'Personne') {
			await fireEvent.change(champ('form-type'), { target: { value: 'Personne' } });
			await settle(20);
		}
	}

	async function soumettre() {
		const form = champ('form-email').closest('form') as HTMLFormElement;
		await fireEvent.submit(form);
		await settle(30);
	}

	// AC4 (#1331) — l'adresse : rouge par M6 à M10, une mutation par champ.
	for (const [id, nom] of [
		['form-street', 'rue'],
		['form-building', 'numéro'],
		['form-npa', 'NPA'],
		['form-city', 'localité'],
		['form-country', 'pays']
	] as const) {
		it(`${nom} : aucun maxlength`, async () => {
			await ouvrirEn('Entreprise');
			sansMaxlength(id);
		});
	}

	// AC7 — les champs du PDF : rouge par M13 (numéro de client), M14
	// (conditions), M15 (raison sociale), M16 (délai). L'article : products-page.
	for (const [id, nom] of [
		['form-client-number', 'numéro de client'],
		['form-payment-terms', 'conditions de paiement'],
		['form-name', 'raison sociale'],
		['form-payment-terms-days', 'délai de paiement']
	] as const) {
		it(`${nom} : aucun maxlength`, async () => {
			// La raison sociale n'existe qu'en type « Entreprise » (le type par défaut).
			await ouvrirEn('Entreprise');
			sansMaxlength(id);
		});
	}

	// AC8 (#1334) — le prénom et le nom : rouge par M17, M18. Ils n'existent
	// qu'en type « Personne ».
	for (const [id, nom] of [
		['form-firstname', 'prénom'],
		['form-lastname', 'nom de famille']
	] as const) {
		it(`${nom} : aucun maxlength`, async () => {
			await ouvrirEn('Personne');
			sansMaxlength(id);
		});
	}

	// Story v0200-corrections-mineures (#1336, AC4) — l'e-mail et le téléphone, laissés par
	// la 48-15 : rouge par M5 (e-mail), M6 (téléphone). Un `it` par champ.
	it('e-mail : aucun maxlength', async () => {
		await ouvrirEn('Entreprise');
		sansMaxlength('form-email');
	});

	it('téléphone : aucun maxlength', async () => {
		await ouvrirEn('Entreprise');
		sansMaxlength('form-phone');
	});

	it('#1336 : un e-mail de 321 caractères part entier, le refus du serveur s’affiche, la saisie reste', async () => {
		const courriel = 'a'.repeat(309) + '@example.com';
		expect(courriel.length).toBe(321);
		const refus = "L'email doit faire au plus 320 caractères";
		createContactMock.mockRejectedValueOnce({ code: 'VALIDATION_ERROR', status: 400, message: refus });
		await ouvrirEn('Entreprise');
		await saisir('form-name', 'Démo SA', 20);
		await saisir('form-email', courriel, 20);
		await soumettre();
		const envoye = createContactMock.mock.calls[0][0] as { email: string };
		expect(envoye.email).toBe(courriel);
		expect(zone('contact-form-error').textContent).toBe(refus);
		expect((champ('form-email') as HTMLInputElement).value).toBe(courriel);
	});

	it('#1336 : un téléphone de 51 caractères part entier, le refus du serveur s’affiche, la saisie reste', async () => {
		const numero = '0'.repeat(51);
		const refus = 'Le téléphone doit faire au plus 50 caractères';
		createContactMock.mockRejectedValueOnce({ code: 'VALIDATION_ERROR', status: 400, message: refus });
		await ouvrirEn('Entreprise');
		await saisir('form-name', 'Démo SA', 20);
		await saisir('form-phone', numero, 20);
		await soumettre();
		const envoye = createContactMock.mock.calls[0][0] as { phone: string };
		expect(envoye.phone).toBe(numero);
		expect(zone('contact-form-error').textContent).toBe(refus);
		expect((champ('form-phone') as HTMLInputElement).value).toBe(numero);
	});

	it('AC4 : une rue de 71 caractères part entière, son refus s’affiche, la saisie reste ; raccourcie, elle s’enregistre', async () => {
		const refus = 'adresse : le champ rue dépasse 70 caractères (71)';
		createContactMock.mockRejectedValueOnce({ code: 'VALIDATION_ERROR', status: 400, message: refus });
		await ouvrirEn('Entreprise');
		await saisir('form-name', 'Démo SA', 20);
		await saisir('form-street', 'R'.repeat(71), 20);
		await saisir('form-npa', '1000', 20);
		await saisir('form-city', 'Lausanne', 20);
		await soumettre();
		const envoye = createContactMock.mock.calls[0][0] as {
			addressStructured: { street: string };
		};
		expect(envoye.addressStructured.street).toBe('R'.repeat(71));
		// Revue P1 (A6) : `formError` lui-même, non le `body` (le toast porte le même texte).
		expect(zone('contact-form-error').textContent).toBe(refus);
		expect((champ('form-street') as HTMLInputElement).value).toBe('R'.repeat(71));

		createContactMock.mockResolvedValueOnce(c({ id: 40, name: 'Démo SA' }));
		await saisir('form-street', 'R'.repeat(70), 20);
		await soumettre();
		expect(createContactMock).toHaveBeenCalledTimes(2);
		const second = createContactMock.mock.calls[1][0] as { addressStructured: { street: string } };
		expect(second.addressStructured.street).toBe('R'.repeat(70));
		expect(document.querySelector('#form-street')).toBeNull();
	});

	it('AC8 : un prénom de 71 caractères part entier, le refus qui nomme « Prénom » s’affiche, la saisie reste', async () => {
		const refus = 'Prénom : 70 caractères maximum';
		createContactMock.mockRejectedValueOnce({ code: 'VALIDATION_ERROR', status: 400, message: refus });
		await ouvrirEn('Personne');
		await saisir('form-firstname', 'P'.repeat(71), 20);
		await saisir('form-lastname', 'Dupont', 20);
		await soumettre();
		const envoye = createContactMock.mock.calls[0][0] as { firstName: string };
		expect(envoye.firstName).toBe('P'.repeat(71));
		expect(zone('contact-form-error').textContent).toBe(refus);
		expect((champ('form-firstname') as HTMLInputElement).value).toBe('P'.repeat(71));
	});

	/** Le bouton de soumission du formulaire ouvert. */
	function boutonCreer(): HTMLButtonElement {
		const form = champ('form-email').closest('form') as HTMLFormElement;
		const b = [...form.querySelectorAll('button')].find((x) => x.getAttribute('type') === 'submit');
		if (!b) throw new Error('bouton de soumission introuvable');
		return b as HTMLButtonElement;
	}

	// AC7, C-e4815-4 — l'écran juge lui-même le délai et la raison sociale : le bouton se
	// désactive, et le MOTIF s'affiche sous le champ, sans attendre une soumission que le
	// bouton désactivé n'atteindrait jamais (sinon, le refus serait muet).
	it('AC7 : un délai de quatre chiffres collé est refusé en le disant (« entre 0 et 365 »), non tronqué en « 100 »', async () => {
		await ouvrirEn('Entreprise');
		await saisir('form-name', 'Démo SA', 20);
		await saisir('form-payment-terms-days', '1000', 20);
		expect(zone('contact-payment-terms-days-error').textContent).toContain('entre 0 et 365');
		expect(boutonCreer().disabled).toBe(true);
		expect((champ('form-payment-terms-days') as HTMLInputElement).value).toBe('1000');
		await saisir('form-payment-terms-days', '30', 20);
		expect(document.querySelector('[data-testid="contact-payment-terms-days-error"]')).toBeNull();
		expect(boutonCreer().disabled).toBe(false);
	});

	it('AC7 : une raison sociale de 256 caractères collée est refusée en le disant, non coupée à 255', async () => {
		await ouvrirEn('Entreprise');
		await saisir('form-name', 'R'.repeat(256), 20);
		expect(zone('contact-name-too-long').textContent).toContain('255 caractères');
		expect(boutonCreer().disabled).toBe(true);
		expect((champ('form-name') as HTMLInputElement).value).toBe('R'.repeat(256));
		expect(createContactMock).not.toHaveBeenCalled();
	});

	// Revue P1 (E4) — les états du motif, un `it` chacun.
	const motifNom = () => document.querySelector('[data-testid="contact-name-too-long"]');
	const motifDelai = () => document.querySelector('[data-testid="contact-payment-terms-days-error"]');

	it('motif de la raison sociale : ramenée à 255, il disparaît et le bouton se réactive', async () => {
		await ouvrirEn('Entreprise');
		await saisir('form-name', 'R'.repeat(256), 20);
		expect(motifNom()).not.toBeNull();
		await saisir('form-name', 'R'.repeat(255), 20);
		expect(motifNom()).toBeNull();
		expect(boutonCreer().disabled).toBe(false);
	});

	// Revue P1 (B1 = E1 = A1) — rouge sans le correctif (mutation M22) : le bouton restait grisé,
	// sans motif, la raison sociale (et son motif) n'existant pas en type « Personne ».
	it('bascule vers « Personne » après une raison sociale de 256 : aucun refus muet, la personne se crée', async () => {
		createContactMock.mockResolvedValueOnce(c({ id: 41, name: 'Jean Dupont' }));
		await ouvrirEn('Entreprise');
		await saisir('form-name', 'R'.repeat(256), 20);
		expect(boutonCreer().disabled).toBe(true);
		await fireEvent.change(champ('form-type'), { target: { value: 'Personne' } });
		await settle(20);
		await saisir('form-firstname', 'Jean', 20);
		await saisir('form-lastname', 'Dupont', 20);
		expect(boutonCreer().disabled).toBe(false);
		await soumettre();
		expect(createContactMock).toHaveBeenCalledWith(
			expect.objectContaining({ contactType: 'Personne', firstName: 'Jean', lastName: 'Dupont' })
		);
	});

	// Revue P1 (B2, E2) — rouge sans le correctif (mutation M23) : compté en unités UTF-16, un nom
	// de 200 caractères dont 60 hors du plan de base (260 unités) était refusé ; le serveur l'accepte.
	it('raison sociale : compte en caractères, non en unités UTF-16 (200 caractères dont 60 hors BMP)', async () => {
		await ouvrirEn('Entreprise');
		const nom = 'R'.repeat(140) + '𝔸'.repeat(60);
		expect([...nom].length).toBe(200);
		expect(nom.length).toBe(260);
		await saisir('form-name', nom, 20);
		expect(motifNom()).toBeNull();
		expect(boutonCreer().disabled).toBe(false);
		await saisir('form-name', 'R'.repeat(196) + '𝔸'.repeat(60), 20);
		expect(motifNom()).not.toBeNull();
	});

	// Revue P1 (B5, E5) — rouge sans le correctif (mutation M25) : `Number` lisait « 0x1E » comme 30
	// et « 3e2 » comme 300, valeurs acceptées et envoyées sans que l'utilisateur les ait écrites.
	for (const ecrit of ['0x1E', '3e2', 'abc', '30.0', '-1']) {
		it(`délai « ${ecrit} » : refusé en le disant, rien ne part`, async () => {
			await ouvrirEn('Entreprise');
			await saisir('form-name', 'Démo SA', 20);
			await saisir('form-payment-terms-days', ecrit, 20);
			expect(motifDelai()?.textContent).toContain('entre 0 et 365');
			expect(boutonCreer().disabled).toBe(true);
		});
	}

	it('délai vidé : le motif disparaît, le bouton se réactive', async () => {
		await ouvrirEn('Entreprise');
		await saisir('form-name', 'Démo SA', 20);
		await saisir('form-payment-terms-days', '1000', 20);
		expect(motifDelai()).not.toBeNull();
		await saisir('form-payment-terms-days', '', 20);
		expect(motifDelai()).toBeNull();
		expect(boutonCreer().disabled).toBe(false);
	});

	it('ouverture d’une fiche existante valide : aucun motif', async () => {
		repondre(
			{},
			[c({ id: 42, name: 'Démo Existante SA', defaultPaymentTermsDays: 30 } as Partial<ContactResponse> & {
				id: number;
				name: string;
			})]
		);
		render(Page);
		await settle(30);
		await ouvrirEdition();
		expect((champ('form-payment-terms-days') as HTMLInputElement).value).toBe('30');
		expect(motifNom()).toBeNull();
		expect(motifDelai()).toBeNull();
	});
});

// ---------------------------------------------------------------------------
// Story v0200-corrections-mineures — #1342 (un pays seul est refusé, AC7) et #1349 (le motif
// d'un bouton grisé s'affiche près du bouton, AC8). Le i18n est mocké : les textes lus sont
// les replis de `i18nMsg`.
// ---------------------------------------------------------------------------

describe('v0200 — un bouton grisé dit pourquoi, un pays seul est refusé', () => {
	const PAYS_SEUL =
		'Un pays seul ne fait pas une adresse : saisissez le NPA et la localité, ou videz le pays.';

	/** Le bouton de soumission du formulaire ouvert (sonde positive : il existe). */
	function bouton(): HTMLButtonElement {
		const b = document.querySelector<HTMLButtonElement>('[data-testid="contact-form-submit"]');
		if (!b) throw new Error('bouton contact-form-submit introuvable');
		return b;
	}

	/** La ligne du motif, toujours dans le DOM ; son texte rogné. */
	function motif(): string {
		return (zone('contact-form-blocked-reason').textContent ?? '').trim();
	}

	/** Le bouton est trouvé et GRISÉ, puis son motif est lu. */
	function grisePour(texte: string) {
		expect(bouton().disabled).toBe(true);
		expect(motif()).toBe(texte);
	}

	async function ouvrirEn(type: 'Entreprise' | 'Personne') {
		repondre({});
		render(Page);
		await settle(20);
		await ouvrirCreation();
		if (type === 'Personne') {
			await fireEvent.change(champ('form-type'), { target: { value: 'Personne' } });
			await settle(20);
		}
	}

	const FRANCAIS = c({
		id: 60,
		name: 'Démo France SA',
		addressStructured: {
			street: 'Rue de la Paix',
			building: '3',
			postalCode: '75002',
			city: 'Paris',
			country: 'FR'
		}
	} as Partial<ContactResponse> & { id: number; name: string });

	async function ouvrirFrancais() {
		repondre({}, [FRANCAIS]);
		render(Page);
		await settle(30);
		await ouvrirEdition();
	}

	// --- AC7 (#1342) ---------------------------------------------------------

	it('AC7 : pays « FR » seul → bouton grisé, le motif le dit (M8, M9)', async () => {
		await ouvrirEn('Entreprise');
		await saisir('form-name', 'Démo SA', 20);
		await saisir('form-country', 'FR', 20);
		grisePour(PAYS_SEUL);
	});

	it('AC7 : pays « CH » seul, ou vidé → bouton actif (un contact sans adresse s’enregistre)', async () => {
		await ouvrirEn('Entreprise');
		await saisir('form-name', 'Démo SA', 20);
		await saisir('form-country', 'ch', 20);
		expect(bouton().disabled).toBe(false);
		expect(motif()).toBe('');
		await saisir('form-country', '', 20);
		expect(bouton().disabled).toBe(false);
		expect(motif()).toBe('');
	});

	it('AC7 : un contact existant avec une adresse en « FR » se rouvre et se réenregistre', async () => {
		updateContactMock.mockResolvedValueOnce(FRANCAIS);
		await ouvrirFrancais();
		expect((champ('form-country') as HTMLInputElement).value).toBe('FR');
		expect(bouton().disabled).toBe(false);
		expect(motif()).toBe('');
		const form = champ('form-email').closest('form') as HTMLFormElement;
		await fireEvent.submit(form);
		await settle(30);
		expect(updateContactMock).toHaveBeenCalledTimes(1);
	});

	it('AC7 : contact étranger dont on retire l’adresse — grisé tant que le pays reste, puis pays vidé → part en « CH », sans adresse (M8, M9)', async () => {
		updateContactMock.mockResolvedValueOnce(FRANCAIS);
		await ouvrirFrancais();
		for (const id of ['form-street', 'form-building', 'form-npa', 'form-city']) {
			await saisir(id, '', 20);
		}
		grisePour(PAYS_SEUL);
		// Second temps (filet) : le pays vidé, le contact part sans adresse.
		await saisir('form-country', '', 20);
		expect(bouton().disabled).toBe(false);
		expect(motif()).toBe('');
		const form = champ('form-email').closest('form') as HTMLFormElement;
		await fireEvent.submit(form);
		await settle(30);
		const envoye = updateContactMock.mock.calls[0][1] as {
			addressStructured: Record<string, string>;
		};
		expect(envoye.addressStructured).toEqual({
			street: '',
			building: '',
			postalCode: '',
			city: '',
			country: 'CH'
		});
	});

	// --- AC8 (#1349) : un `it` par branche de `formValidation` -----------------

	it('AC8 : personne sans prénom → le motif s’affiche près du bouton (M9)', async () => {
		await ouvrirEn('Personne');
		await saisir('form-lastname', 'Dupont', 20);
		grisePour('Prénom et nom obligatoires pour une personne');
	});

	it('AC8 : personne sans nom → le motif s’affiche près du bouton (M9)', async () => {
		await ouvrirEn('Personne');
		await saisir('form-firstname', 'Jean', 20);
		grisePour('Prénom et nom obligatoires pour une personne');
	});

	it('AC8 : entreprise sans raison sociale → le motif s’affiche dès l’ouverture (M9)', async () => {
		await ouvrirEn('Entreprise');
		grisePour('La raison sociale est obligatoire');
	});

	it('AC8 : adresse partielle → le motif s’affiche près du bouton (M9)', async () => {
		await ouvrirEn('Entreprise');
		await saisir('form-name', 'Démo SA', 20);
		await saisir('form-street', 'Rue du Lac', 20);
		grisePour('NPA et localité obligatoires si une adresse est saisie');
	});

	it('AC8 : pays seul → le motif s’affiche près du bouton (M8, M9)', async () => {
		await ouvrirEn('Personne');
		await saisir('form-firstname', 'Jean', 20);
		await saisir('form-lastname', 'Dupont', 20);
		await saisir('form-country', 'DE', 20);
		grisePour(PAYS_SEUL);
	});

	it('AC8 : IDE de forme invalide → le motif s’affiche près du bouton (M9)', async () => {
		await ouvrirEn('Entreprise');
		await saisir('form-name', 'Démo SA', 20);
		await saisir('form-ide', 'CHE-12', 20);
		grisePour('Numéro IDE suisse invalide');
	});

	it('AC8 : raison sociale de 256 → motif sous le champ seulement, jamais deux fois (M10)', async () => {
		await ouvrirEn('Entreprise');
		await saisir('form-name', 'R'.repeat(256), 20);
		expect(bouton().disabled).toBe(true);
		expect(zone('contact-name-too-long').textContent).toContain('255 caractères');
		expect(motif()).toBe('');
	});

	it('AC8 : délai « 400 » → motif sous le champ seulement, jamais deux fois (M10)', async () => {
		await ouvrirEn('Entreprise');
		await saisir('form-name', 'Démo SA', 20);
		await saisir('form-payment-terms-days', '400', 20);
		expect(bouton().disabled).toBe(true);
		expect(zone('contact-payment-terms-days-error').textContent).toContain('entre 0 et 365');
		expect(motif()).toBe('');
	});

	it('AC8 : formulaire valide → ligne vide, bouton actif', async () => {
		await ouvrirEn('Entreprise');
		await saisir('form-name', 'Démo SA', 20);
		expect(zone('contact-form-blocked-reason')).not.toBeNull();
		expect(motif()).toBe('');
		expect(bouton().disabled).toBe(false);
	});
});
