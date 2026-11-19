// Story 16-2b (#144) — fiche produit : chargement du plan comptable.
//
// ⚠️ CE FICHIER EXISTE POUR UNE SEULE ASSERTION, ET ELLE EST LA RAISON D'ÊTRE
// D'AC-B2 : `expect(fetchAccountsMock).toHaveBeenCalledWith(true)`.
//
// Pourquoi une assertion dédiée, alors qu'un test fonctionnel « le libellé du
// compte archivé reste affiché » semble couvrir la même chose : 16-1b l'a
// **mesuré**. Le mock de `fetchAccounts` rend la liste complète quel que soit
// son argument, donc un test fonctionnel reste **vert** sous la mutation
// `fetchAccounts(true) → fetchAccounts()`. Seule l'assertion sur l'argument
// l'attrape.
//
// L'assertion homologue d'`InvoiceForm.test.ts` ne couvre PAS ce site : c'est
// un appel **distinct**, dans un autre composant, et la story le dit — « déjà
// en place et non touché par cette story ». La passe 1 de `bmad-code-review` a
// constaté qu'aucun fichier de test n'existait pour cette page, alors que T-B2
// et T-B4 étaient cochées ; c'est ce fichier qui les rend vraies.
//
// Pattern projet : mocks hoistés AVANT l'import du composant, render via
// `@testing-library/svelte` (Svelte 5).

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, fireEvent, cleanup } from '@testing-library/svelte';
import Big from 'big.js';
import type { AccountResponse } from '$lib/features/accounts/accounts.types';
import type { ProductResponse } from '$lib/features/products/products.types';

vi.mock('$app/environment', () => ({ browser: true }));
vi.mock('$app/navigation', () => ({ goto: vi.fn() }));
vi.mock('$app/state', () => ({ page: { url: new URL('http://localhost/products') } }));

// Story 48-10 (M10) : le mock interpole `{ $max }`, comme celui de
// `journal-entries-list-page.test.ts` — sans quoi un nombre de décimales faux resterait invisible.
vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_key: string, fallback: string, args?: Record<string, string | number>) =>
		args ? fallback.replace(/\{\s*\$(\w+)\s*\}/g, (_, n) => String(args[n] ?? '')) : fallback,
}));

const notifyErrorMock = vi.fn();
vi.mock('$lib/shared/utils/notify', () => ({
	notifyError: (...args: unknown[]) => notifyErrorMock(...args),
	notifySuccess: vi.fn(),
}));

const listProductsMock = vi.fn();
// Story 36-1 (AC4) : nommés, pour constater qu'aucun envoi ne part.
const createProductMock = vi.fn();
const updateProductMock = vi.fn();
vi.mock('$lib/features/products/products.api', () => ({
	listProducts: (q?: unknown) => listProductsMock(q),
	createProduct: (...a: unknown[]) => createProductMock(...a),
	updateProduct: (...a: unknown[]) => updateProductMock(...a),
	archiveProduct: vi.fn(),
}));

const fetchAccountsMock = vi.fn();
vi.mock('$lib/features/accounts/accounts.api', () => ({
	fetchAccounts: (includeArchived?: boolean) => fetchAccountsMock(includeArchived),
}));

vi.mock('$lib/features/vat-rates', () => ({
	getVatRates: vi.fn(async () => [{ rate: '8.10', label: 'product-vat-normal' }]),
}));

import Page from './+page.svelte';

function acc(partial: Partial<AccountResponse> & { id: number; number: string }): AccountResponse {
	return {
		companyId: 1,
		name: 'Ventes',
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

const SALES = acc({ id: 3000, number: '3000', name: 'Ventes' });
const ARCHIVED = acc({ id: 3900, number: '3900', name: 'Ventes closes', active: false });

/** Laisse passer `onMount` et les `$effect` asynchrones (comptes, TVA, liste). */
async function settle() {
	await new Promise((r) => setTimeout(r, 50));
}

beforeEach(() => {
	vi.clearAllMocks();
	fetchAccountsMock.mockResolvedValue([SALES, ARCHIVED]);
	listProductsMock.mockResolvedValue({ items: [], total: 0, limit: 20, offset: 0 });
});

describe('fiche produit — chargement du plan comptable (Story 16-2b, AC-B2)', () => {
	it('AC-B2 : `fetchAccounts` est appelé AVEC `true` — le tueur de la mutation 2', async () => {
		render(Page);
		await settle();

		// ⚠️ NE PAS remplacer par `toHaveBeenCalled()` : c'est l'ARGUMENT qui est
		// l'objet du test. Sans le flag, un article dont le compte a été archivé
		// depuis verrait son champ paraître VIDE — et D-B2 garantit qu'aucun
		// marqueur ne viendrait le nuancer sur cette fiche.
		expect(fetchAccountsMock).toHaveBeenCalledWith(true);
	});

	it("un échec de chargement du plan comptable est SIGNALÉ, pas avalé", async () => {
		fetchAccountsMock.mockRejectedValue(new Error('réseau'));
		render(Page);
		await settle();

		// Sans cette notification, l'utilisateur trouve, à la place de
		// l'autocomplétion, un champ de saisie d'identifiant TECHNIQUE brut,
		// sans aucune explication — et peut y taper un numéro de compte en
		// croyant bien faire. *(Passe 1 de revue.)*
		expect(notifyErrorMock).toHaveBeenCalled();
	});

	it("le chargement nominal ne notifie AUCUNE erreur", async () => {
		render(Page);
		await settle();

		// Contre-épreuve : sans elle, une notification posée inconditionnellement
		// ferait passer le test précédent sans rien mesurer.
		expect(notifyErrorMock).not.toHaveBeenCalled();
	});
});

// Story 36-1 (#1095, AC4) — Entrée dans le compte de produit du dialogue
// d'article. jsdom n'implémente pas la soumission implicite : elle est rejouée
// comme le navigateur (soumission si l'événement n'a pas été empêché).
describe('fiche produit — Entrée dans le compte de produit (Story 36-1)', () => {
	const ARTICLE: ProductResponse = {
		id: 41,
		companyId: 1,
		name: 'Conseil',
		description: null,
		unitPrice: '150.0000',
		vatRate: '8.10',
		defaultRevenueAccountId: SALES.id,
		active: true,
		version: 1,
		createdAt: '',
		updatedAt: '',
	};

	it('texte sans correspondance, Entrée : rien ne part, le dialogue reste ouvert sur la saisie', async () => {
		listProductsMock.mockResolvedValue({ items: [ARTICLE], total: 1, limit: 20, offset: 0 });
		updateProductMock.mockResolvedValue(ARTICLE);
		const { getByLabelText } = render(Page);
		await settle();

		await fireEvent.click(getByLabelText('Modifier'));
		await settle();
		const champ = document.querySelector<HTMLInputElement>('#form-revenue-account');
		if (!champ) throw new Error('champ #form-revenue-account introuvable : dialogue non monté');
		expect(champ.value).toBe('3000 — Ventes');

		await fireEvent.input(champ, { target: { value: 'zzz' } });
		const nonEmpechee = await fireEvent.keyDown(champ, { key: 'Enter' });
		if (nonEmpechee) champ.form!.requestSubmit();
		await settle();

		// L'envoi d'abord : c'est lui, le défaut (#1095) — l'ancien compte partait.
		expect(updateProductMock).not.toHaveBeenCalled();
		expect(createProductMock).not.toHaveBeenCalled();
		expect(nonEmpechee).toBe(false);
		expect(document.querySelector('#form-revenue-account')).not.toBeNull();
		expect(champ.value).toBe('zzz');
	});
});

// Story 48-15 (AC7) — le nom de l'article finit sur le PDF de facture (il devient
// la désignation de la ligne) : un nom collé trop long est refusé en le disant,
// non coupé en silence à 255 par un `maxlength`.
describe('fiche produit — le nom ne coupe pas un texte collé (Story 48-15)', () => {
	async function ouvrirCreation(): Promise<HTMLInputElement> {
		render(Page);
		await settle();
		const bouton = [...document.querySelectorAll('button')].find((b) =>
			/Nouveau produit/.test(b.textContent ?? '')
		);
		if (!bouton) throw new Error('bouton « Nouveau produit » introuvable');
		await fireEvent.click(bouton);
		await settle();
		// Sonde positive : le champ est TROUVÉ avant l'assertion d'absence.
		const champ = document.querySelector<HTMLInputElement>('#form-name');
		if (!champ) throw new Error('champ #form-name introuvable : dialogue non monté');
		return champ;
	}

	// Rouge par M12 (`maxlength={255}` remis).
	it('nom de l’article : aucun maxlength', async () => {
		const champ = await ouvrirCreation();
		expect(champ.tagName).toBe('INPUT');
		expect(champ.hasAttribute('maxlength')).toBe(false);
	});

	// L'écran juge lui-même la longueur du nom : le bouton se désactive, et le motif
	// s'affiche au pied du formulaire dès la frappe (`formTouched && formValidation`).
	// FILET (revue P1, A5) : non-régression du motif existant de la page ; il ne rougit sous
	// aucune mutation de la story (M24 est tuée par le test suivant).
	it('un nom de 256 caractères collé est refusé en le disant, la saisie reste, rien ne part', async () => {
		const champ = await ouvrirCreation();
		await fireEvent.input(champ, { target: { value: 'A'.repeat(256) } });
		await settle();
		expect(champ.form!.textContent).toContain('Le nom doit faire au plus 255 caractères');
		const soumettre = [...champ.form!.querySelectorAll('button')].find(
			(b) => b.getAttribute('type') === 'submit'
		);
		expect(soumettre?.disabled).toBe(true);
		expect(createProductMock).not.toHaveBeenCalled();
		expect(champ.value).toBe('A'.repeat(256));
	});

	// Revue P1 (B2, E2) — rouge sans le correctif (mutation M24) : compté en unités UTF-16, un nom
	// de 200 caractères dont 60 hors du plan de base (260 unités) était refusé ; le serveur l'accepte.
	it('le nom se compte en caractères, non en unités UTF-16 (200 caractères dont 60 hors BMP)', async () => {
		const champ = await ouvrirCreation();
		const nom = 'A'.repeat(140) + '𝔸'.repeat(60);
		expect([...nom].length).toBe(200);
		expect(nom.length).toBe(260);
		await fireEvent.input(champ, { target: { value: nom } });
		await settle();
		expect(champ.form!.textContent).not.toContain('Le nom doit faire au plus 255 caractères');
		await fireEvent.input(champ, { target: { value: 'A'.repeat(196) + '𝔸'.repeat(60) } });
		await settle();
		expect(champ.form!.textContent).toContain('Le nom doit faire au plus 255 caractères');
	});
});

// Story 48-10 (#1173 ; AC1, AC5, AC7, AC12) — le prix d'un article se lit comme le montant
// d'une écriture : « 1.250 » refusé comme ambigu, sous le champ, sans envoi ni perte de
// saisie ; « 1'250 » part `1250`. Avant la story, « 1.250 » partait et le serveur le lisait 1,25.
describe('fiche produit — prix ambigu (Story 48-10)', () => {
	const AMBIGU = 'Montant ambigu';
	// Les rendus des blocs précédents ne sont pas démontés (aucun nettoyage automatique sans
	// `globals`) : leurs dialogues ouverts porteraient eux aussi un `#form-price`.
	beforeEach(() => cleanup());

	function article(unitPrice: string): ProductResponse {
		return {
			id: 77,
			companyId: 1,
			name: 'Abonnement',
			description: null,
			unitPrice,
			vatRate: '8.10',
			defaultRevenueAccountId: null,
			active: true,
			version: 4,
			createdAt: '',
			updatedAt: '',
		};
	}

	function prix(): HTMLInputElement {
		const champ = document.querySelector<HTMLInputElement>('#form-price');
		if (!champ) throw new Error('champ #form-price introuvable : dialogue non monté');
		return champ;
	}
	const messageSousLeChamp = () => document.querySelector('[data-testid="product-form-price-error"]');
	const piedRouge = () => document.querySelector('[data-testid="product-form-error"]');
	const soumettre = () =>
		[...prix().form!.querySelectorAll('button')].find((b) => b.getAttribute('type') === 'submit')!;

	async function ouvrirCreation(): Promise<void> {
		render(Page);
		await settle();
		const bouton = [...document.querySelectorAll('button')].find((b) =>
			/Nouveau produit/.test(b.textContent ?? '')
		);
		if (!bouton) throw new Error('bouton « Nouveau produit » introuvable');
		await fireEvent.click(bouton);
		await settle();
		const nom = document.querySelector<HTMLInputElement>('#form-name')!;
		await fireEvent.input(nom, { target: { value: 'Conseil' } });
	}

	async function ouvrirEdition(unitPrice: string): Promise<void> {
		listProductsMock.mockResolvedValue({ items: [article(unitPrice)], total: 1, limit: 20, offset: 0 });
		const { getByLabelText } = render(Page);
		await settle();
		await fireEvent.click(getByLabelText('Modifier'));
		await settle();
	}

	async function taper(valeur: string): Promise<void> {
		await fireEvent.input(prix(), { target: { value: valeur } });
		await settle();
	}

	/**
	 * Soumission du formulaire, **même bouton désactivé** (`fireEvent.submit`) : scénario défensif
	 * de la fiche (D1). Revue P1 : `requestSubmit()` n'émet rien dans jsdom quand le bouton par
	 * défaut est désactivé — les tests « Entrée » sur une saisie refusée ne soumettaient donc rien.
	 */
	async function entree(): Promise<void> {
		await fireEvent.submit(prix().form!);
		await settle();
	}

	// AC1 — rouge avant le correctif : « 1.250 » partait (`unitPrice: '1.250'`).
	it('création : « 1.250 » → message ambigu sous le champ, rien ne part, saisie conservée (M1)', async () => {
		await ouvrirCreation();
		await taper('1.250');
		expect(messageSousLeChamp()?.textContent).toContain(AMBIGU);
		expect(soumettre().disabled).toBe(true);
		await entree();
		expect(createProductMock).not.toHaveBeenCalled();
		expect(updateProductMock).not.toHaveBeenCalled();
		expect(prix().value).toBe('1.250');
	});

	it('modification : « 1,250 » → message ambigu, `updateProduct` non appelé', async () => {
		await ouvrirEdition('150.0000');
		await taper('1,250');
		expect(messageSousLeChamp()?.textContent).toContain(AMBIGU);
		await entree();
		expect(updateProductMock).not.toHaveBeenCalled();
		expect(prix().value).toBe('1,250');
	});

	// AC1 — rouge avant le correctif : « 1'250 » était refusé (« Format de prix invalide »).
	it("création : « 1'250 » part `1250` (M2)", async () => {
		createProductMock.mockResolvedValue(article('1250.0000'));
		await ouvrirCreation();
		await taper("1'250");
		expect(messageSousLeChamp()).toBeNull();
		await entree();
		expect(createProductMock).toHaveBeenCalledTimes(1);
		expect(createProductMock.mock.calls[0][0]).toMatchObject({ unitPrice: '1250' });
	});

	// AC5 — rien de juste n'est refusé ; la charge part normalisée (M14 : « 1250 » bordé de blancs).
	for (const [saisie, charge] of [
		['1.25', '1.25'],
		['1250', '1250'],
		["1'250.50", '1250.50'],
		['1’250', '1250'],
		['1,25', '1.25'],
		['0.50', '0.50'],
		[' 1250 ', '1250'],
		['1250.0000', '1250.0000'],
		['12.1250', '12.1250'],
		['0.125', '0.125'],
		['1234.125', '1234.125'],
		["1'234.125", '1234.125'],
	]) {
		it(`création : « ${saisie} » accepté sans message, part « ${charge} »`, async () => {
			createProductMock.mockResolvedValue(article('1.0000'));
			await ouvrirCreation();
			await taper(saisie);
			expect(messageSousLeChamp()).toBeNull();
			expect(prix().getAttribute('aria-invalid')).not.toBe('true');
			await entree();
			expect(createProductMock).toHaveBeenCalledTimes(1);
			expect(createProductMock.mock.calls[0][0]).toMatchObject({ unitPrice: charge });
		});
	}

	it('« abc » et « -1 » : refus sous le champ, avec leurs messages actuels, rien ne part', async () => {
		await ouvrirCreation();
		await taper('abc');
		expect(messageSousLeChamp()?.textContent).toContain('Format de prix invalide');
		await taper('-1');
		expect(messageSousLeChamp()?.textContent).toContain('Le prix doit être positif ou nul');
		await taper('1e3');
		expect(messageSousLeChamp()?.textContent).toContain('Format de prix invalide');
		await entree();
		expect(createProductMock).not.toHaveBeenCalled();
	});

	// AC1, AC7 — un article rouvert sans toucher au prix : aucun message, la valeur d'origine
	// repart (jamais un montant reformaté qui change de valeur). FILETS (M3 les tue).
	for (const [serveur, champ] of [
		['1.2340', '1.2340'],
		['1.2500', '1.25'],
		['1500.0000', '1500.00'],
		['0.1230', '0.1230'],
	]) {
		it(`rouvert au prix ${serveur} → champ « ${champ} », sans message, enregistrable à la même valeur (M3)`, async () => {
			updateProductMock.mockResolvedValue(article(serveur));
			await ouvrirEdition(serveur);
			expect(prix().value).toBe(champ);
			expect(messageSousLeChamp()).toBeNull();
			expect(soumettre().disabled).toBe(false);
			await entree();
			expect(updateProductMock).toHaveBeenCalledTimes(1);
			const envoye = (updateProductMock.mock.calls[0][1] as { unitPrice: string }).unitPrice;
			expect(new Big(envoye).eq(new Big(serveur))).toBe(true);
		});
	}

	// AC1 — prix effacé : le motif reste visible au pied (M20), bouton désactivé.
	it('prix effacé à l’édition → « Le prix est obligatoire » visible au pied, bouton désactivé (M20)', async () => {
		await ouvrirEdition('150.0000');
		await taper('');
		expect(prix().form!.textContent).toContain('Le prix est obligatoire');
		expect(messageSousLeChamp()).toBeNull();
		expect(soumettre().disabled).toBe(true);
	});

	// AC12 — accessibilité du refus, et retour à l'état neutre après correction (M16).
	it('« 1.250 » : aria-invalid et aria-describedby nomment le message ; corrigé, ils le quittent', async () => {
		await ouvrirCreation();
		await taper('1.250');
		const msg = messageSousLeChamp()!;
		expect(msg.id).toBe('form-price-error');
		expect(prix().getAttribute('aria-invalid')).toBe('true');
		expect((prix().getAttribute('aria-describedby') ?? '').split(' ')).toContain(msg.id);
		await taper("1'250");
		expect(['false', null]).toContain(prix().getAttribute('aria-invalid'));
		expect((prix().getAttribute('aria-describedby') ?? '').split(' ')).not.toContain('form-price-error');
	});

	// AC12 (V2-3) — Entrée sur « 1.250 », puis correction : aucun message rouge périmé au pied (M19).
	it('Entrée sur « 1.250 », puis correction en « 1’250 » → aucun message rouge au pied', async () => {
		await ouvrirCreation();
		await taper('1.250');
		await entree();
		// Le message du prix n'est pas répété au pied : il est déjà sous le champ.
		expect(piedRouge()).toBeNull();
		expect(createProductMock).not.toHaveBeenCalled();
		await taper("1'250");
		expect(piedRouge()).toBeNull();
		expect(prix().form!.textContent).not.toContain(AMBIGU);
	});

	// Revue P1 (B1) — prix vide + Entrée, puis prix saisi : « Le prix est obligatoire » ne reste pas
	// en rouge au pied sous un prix rempli.
	it('prix vide + Entrée, puis « 12.50 » → plus de message rouge au pied (M24)', async () => {
		await ouvrirCreation();
		await entree();
		expect(piedRouge()?.textContent).toContain('Le prix est obligatoire');
		await taper('12.50');
		expect(piedRouge()).toBeNull();
	});

	// Revue P1 (E1) — un refus du serveur sur le prix ne reste pas au pied après correction.
	it('refus du serveur sur le prix, puis correction du prix → plus de message rouge au pied (M24)', async () => {
		const plafond = 'Le prix unitaire doit être inférieur ou égal à 1 000 000 000';
		createProductMock.mockRejectedValue({ code: 'VALIDATION_ERROR', status: 400, message: plafond });
		await ouvrirCreation();
		await taper('2000000000');
		await entree();
		expect(piedRouge()?.textContent).toContain(plafond);
		await taper('200');
		expect(piedRouge()).toBeNull();
	});

	// Ce qui ne vient pas du prix reste : le doublon de nom refusé par le serveur.
	it('doublon de nom refusé par le serveur : une saisie du prix ne l’efface pas', async () => {
		createProductMock.mockRejectedValue({ code: 'RESOURCE_CONFLICT', status: 409, message: 'conflit' });
		await ouvrirCreation();
		await taper('12.50');
		await entree();
		expect(piedRouge()?.textContent).toContain('Un produit avec ce nom existe déjà');
		await taper('13.50');
		expect(piedRouge()?.textContent).toContain('Un produit avec ce nom existe déjà');
	});

	// Revue P1 (E2) — nom vide ET prix ambigu : le motif du nom reste au pied, le refus du prix sous
	// le champ (mutation du pied `formValidation !== priceMessage` → `!priceMessage`, M25).
	it('nom vide et « 1.250 » : motif du nom au pied, refus du prix sous le champ, bouton désactivé (M25)', async () => {
		await ouvrirCreation();
		await fireEvent.input(document.querySelector<HTMLInputElement>('#form-name')!, {
			target: { value: '' },
		});
		await taper('1.250');
		expect(prix().form!.textContent).toContain('Le nom est obligatoire');
		expect(messageSousLeChamp()?.textContent).toContain(AMBIGU);
		expect(soumettre().disabled).toBe(true);
	});
});
