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
vi.mock('$lib/features/contacts/contacts.api', () => ({
	listContacts: (q?: unknown) => listContactsMock(q),
	createContact: vi.fn(),
	updateContact: vi.fn(),
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
		// ⚠️ `formValidation` n'est PAS rendu tant qu'on n'a pas soumis : il
		// n'alimente que le `disabled`, et `submitForm` en fait un `formError`.
		// Il faut donc SOUMETTRE pour le lire — et c'est bien ce qu'on veut
		// éprouver : ce que l'utilisateur se verra reprocher.
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
