// Story v0190-correction (#1162), AC1 — `ContactPicker` : le client que le
// champ montre est celui que le parent enregistrera.
//
// Montage : `listContacts` mocké (filtre par sous-chaîne, ou promesse tenue à
// la main pour les courses) ; temps simulé pour l'anti-rebond de 300 ms.
// L'événement d'*Entrée* se constate par la valeur rendue de `fireEvent.keyDown`
// (`false` quand `preventDefault` a été appelé). Les cas qui constatent
// l'affichage après un choix réinjectent `selected` comme le ferait un parent :
// l'espion `onSelect` appelle `rerender`.
//
// Focus et `blur` passent par `el.focus()` / `el.blur()` (et non
// `fireEvent.focus`) : *Échap* rend le focus au champ par `inputEl.focus()`,
// qui, sous jsdom, déclencherait un second `focus` si le champ n'était pas
// réellement actif.

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, fireEvent, cleanup } from '@testing-library/svelte';
import { tick } from 'svelte';
import { readFileSync } from 'node:fs';
import type { ContactResponse } from '$lib/features/contacts/contacts.types';
import {
	CLASSE_DESIGNEE,
	CLASSE_SURVOL
} from '$lib/features/journal-entries/account-autocomplete-classes';
import { contrastRatio } from '../../../../tests/e2e/helpers/contrast';
import { blocTheme, jetonDans, jetonsDesClasses } from '../../../../tests/e2e/helpers/theme';

const listContactsMock = vi.fn();
vi.mock('$lib/features/contacts/contacts.api', () => ({
	listContacts: (q: unknown) => listContactsMock(q)
}));

import ContactPicker from './ContactPicker.svelte';

// ── Données (fictives) ──────────────────────────────────────────────────────

function contact(id: number, name: string, ideNumber: string | null = null): ContactResponse {
	return { id, name, ideNumber } as unknown as ContactResponse;
}

const DUPONT = contact(1, 'Dupont SA');
const MARTIN = contact(2, 'Martin Sàrl', 'CHE109322551');
const MARCEL = contact(3, 'Marcel et Cie');
const TOUS = [DUPONT, MARTIN, MARCEL];

function reponse(items: ContactResponse[]) {
	return { items, total: items.length, offset: 0, limit: 50 };
}

/** Recherche simulée : sous-chaîne, sans casse. */
function parFiltre(q: { search?: string }) {
	const s = (q?.search ?? '').toLowerCase();
	return reponse(TOUS.filter((c) => c.name.toLowerCase().includes(s)));
}

/** Promesse tenue à la main : une recherche « en vol ». */
function enVol() {
	let tenir!: (items: ContactResponse[]) => void;
	let rompre!: () => void;
	const promesse = new Promise<ReturnType<typeof reponse>>((res, rej) => {
		tenir = (items) => res(reponse(items));
		rompre = () => rej(new Error('réseau'));
	});
	listContactsMock.mockImplementationOnce(() => promesse);
	return { tenir, rompre };
}

// ── Montage et gestes ───────────────────────────────────────────────────────

const ANTI_REBOND = 300;

/** Laisse passer `ms` de temps simulé et les microtâches qui suivent. */
async function laisser(ms = 0) {
	await vi.advanceTimersByTimeAsync(ms);
	await tick();
	await tick();
}

function monter(
	selected: ContactResponse | null = DUPONT,
	autres: { keepDraftOnBlur?: boolean; onDraftChange?: (u: boolean) => void } = {}
) {
	const onSelect = vi.fn();
	const r = render(ContactPicker, { selected, onSelect, ...autres });
	onSelect.mockImplementation((c: ContactResponse) => {
		void r.rerender({ selected: c, onSelect });
	});
	const input = r.getByRole('combobox') as HTMLInputElement;
	return { ...r, input, onSelect };
}

async function focus(el: HTMLInputElement) {
	el.focus();
	await tick();
}

async function quitter(el: HTMLInputElement) {
	el.blur();
	await tick();
}

async function taper(el: HTMLInputElement, v: string) {
	await fireEvent.input(el, { target: { value: v } });
}

async function touche(el: HTMLInputElement, key: string, init: KeyboardEventInit = {}) {
	const nonEmpechee = await fireEvent.keyDown(el, { key, ...init });
	await tick();
	return nonEmpechee;
}

const entree = (el: HTMLInputElement, init: KeyboardEventInit = {}) => touche(el, 'Enter', init);
const bas = (el: HTMLInputElement) => touche(el, 'ArrowDown');
const haut = (el: HTMLInputElement) => touche(el, 'ArrowUp');
const echap = (el: HTMLInputElement) => touche(el, 'Escape');

function options(container: HTMLElement): HTMLElement[] {
	return Array.from(container.querySelectorAll('[role="option"]'));
}

function porte(el: Element, classes: string): boolean {
	return classes.split(/\s+/).every((c) => el.classList.contains(c));
}

/** Noms des options qui portent `CLASSE_DESIGNEE`. */
function designees(container: HTMLElement): string[] {
	return options(container)
		.filter((o) => porte(o, CLASSE_DESIGNEE))
		.map((o) => o.querySelector('.font-medium')?.textContent ?? '');
}

function liste(container: HTMLElement): HTMLElement | null {
	return container.querySelector('[role="listbox"]');
}

function option(container: HTMLElement, nom: string): HTMLElement {
	const o = options(container).find((x) => x.querySelector('.font-medium')?.textContent === nom);
	if (!o) throw new Error(`option « ${nom} » absente`);
	return o;
}

/** Monte, prend le focus et laisse arriver la recherche du focus. */
async function focusPose(selected: ContactResponse | null = DUPONT) {
	const m = monter(selected);
	await focus(m.input);
	await laisser();
	return m;
}

/** Tape `v` et laisse arriver la réponse de sa recherche. */
async function saisir(el: HTMLInputElement, v: string) {
	await taper(el, v);
	await laisser(ANTI_REBOND);
}

beforeEach(() => {
	vi.useFakeTimers();
	listContactsMock.mockReset();
	listContactsMock.mockImplementation(async (q: { search?: string }) => parFiltre(q));
});

afterEach(() => {
	cleanup();
	vi.useRealTimers();
});

// ── AC1 ─────────────────────────────────────────────────────────────────────

describe('ContactPicker — le brouillon se jette au départ du champ (D1)', () => {
	it('a. client retenu, « zzz » tapé, champ quitté : le champ montre le client, rien n’est choisi', async () => {
		const { input, onSelect } = await focusPose(DUPONT);
		await taper(input, 'zzz');
		expect(input.value).toBe('zzz');
		await quitter(input);
		expect(input.value).toBe('Dupont SA');
		expect(onSelect).not.toHaveBeenCalled();
	});

	it('b. aucun client, « zzz » tapé, champ quitté : le champ est vide', async () => {
		const { input, onSelect } = await focusPose(null);
		await taper(input, 'zzz');
		await quitter(input);
		expect(input.value).toBe('');
		expect(onSelect).not.toHaveBeenCalled();
	});

	it('f. choix à la souris puis départ du champ : le champ montre le contact choisi', async () => {
		const { input, container, onSelect } = await focusPose(DUPONT);
		await saisir(input, 'Mar');
		await fireEvent.mouseDown(option(container, 'Marcel et Cie'));
		await laisser();
		expect(onSelect).toHaveBeenCalledWith(MARCEL);
		await quitter(input);
		expect(input.value).toBe('Marcel et Cie');
	});
});

describe('ContactPicker — Entrée (D2)', () => {
	it('c. brouillon, liste fermée par Échap : Entrée empêchée, rien choisi, la liste rouverte ; la seconde Entrée choisit le premier de la réponse à « Mar »', async () => {
		const { input, container, onSelect } = await focusPose(DUPONT);
		await saisir(input, 'Mar');
		expect(await echap(input)).toBe(false);
		expect(liste(container)).toBeNull();

		expect(await entree(input)).toBe(false);
		expect(onSelect).not.toHaveBeenCalled();
		expect(input.value).toBe('Mar');
		expect(liste(container)).not.toBeNull();
		expect(designees(container)).toEqual(['Martin Sàrl']);

		expect(await entree(input)).toBe(false);
		expect(onSelect).toHaveBeenCalledWith(MARTIN);
	});

	it('c. brouillon sans résultat : Entrée empêchée, rien choisi, le texte reste, la liste dit « Aucun contact »', async () => {
		const { input, container, onSelect } = await focusPose(DUPONT);
		await saisir(input, 'zzz');
		expect(await entree(input)).toBe(false);
		expect(onSelect).not.toHaveBeenCalled();
		expect(input.value).toBe('zzz');
		expect(liste(container)?.textContent).toContain('Aucun contact');
	});

	it('d. Entrée pendant l’anti-rebond (anciens résultats présents) : empêchée ; la recherche faite, Entrée choisit dans la NOUVELLE liste', async () => {
		const { input, onSelect } = await focusPose(DUPONT);
		await taper(input, 'Marc');
		await laisser(100);
		// Deux Entrées : la première ne désigne rien dans l'ancienne liste, la
		// seconde ne choisit donc rien (gardes `searching` des règles 1 et 2).
		expect(await entree(input)).toBe(false);
		expect(await entree(input)).toBe(false);
		expect(onSelect).not.toHaveBeenCalled();

		await laisser(ANTI_REBOND);
		expect(await entree(input)).toBe(false);
		expect(onSelect).toHaveBeenCalledWith(MARCEL);
	});

	it('d-bis. la réponse d’une recherche partie avant la dernière frappe ne désigne rien', async () => {
		const { input, container, onSelect } = await focusPose(DUPONT);
		const m = enVol();
		await taper(input, 'M');
		await laisser(ANTI_REBOND); // recherche « M » en vol
		await taper(input, 'Ma'); // anti-rebond armé
		m.tenir([MARTIN, MARCEL]);
		await laisser();
		expect(await entree(input)).toBe(false);
		expect(onSelect).not.toHaveBeenCalled();
		expect(designees(container)).toEqual([]);
	});

	it('d-ter. frappe, départ pendant l’anti-rebond, retour, flèche bas, Entrée : le contact désigné est choisi', async () => {
		const { input, onSelect } = await focusPose(DUPONT);
		await taper(input, 'Ma');
		await quitter(input);
		await laisser(ANTI_REBOND);
		await focus(input);
		await laisser(); // réponse de la recherche du focus
		await bas(input);
		expect(await entree(input)).toBe(false);
		expect(onSelect).toHaveBeenCalledWith(DUPONT);
	});

	it('d-quater. la frappe efface la désignation : « Chargement… », aucune option désignée, pas d’option active ; flèche et Entrée sans effet', async () => {
		const { input, container, onSelect } = await focusPose(DUPONT);
		await saisir(input, 'Ma');
		expect(designees(container)).toEqual(['Martin Sàrl']);

		await taper(input, 'Mar');
		expect(designees(container)).toEqual([]);
		expect(liste(container)?.textContent).toContain('Chargement…');
		expect(input.getAttribute('aria-activedescendant')).toBeNull();

		await bas(input);
		expect(designees(container)).toEqual([]);
		expect(await entree(input)).toBe(false);
		expect(onSelect).not.toHaveBeenCalled();
	});

	it('d-quinquies (i). départ pendant une recherche en vol, réponse après, retour, Entrée : rien choisi, rien désigné', async () => {
		const { input, container, onSelect } = await focusPose(DUPONT);
		const m = enVol();
		await saisir(input, 'Ma'); // en vol
		await quitter(input);
		m.tenir([MARTIN]);
		await laisser();
		await focus(input);
		await laisser();
		expect(designees(container)).toEqual([]);
		await entree(input);
		expect(onSelect).not.toHaveBeenCalled();
	});

	it('d-quinquies (ii). échec de la recherche : « Aucun contact », Entrée empêchée ; une frappe réussie, une flèche et Entrée choisissent', async () => {
		const { input, container, onSelect } = await focusPose(DUPONT);
		listContactsMock.mockImplementationOnce(async () => {
			throw new Error('réseau');
		});
		await saisir(input, 'Ma');
		expect(liste(container)?.textContent).toContain('Aucun contact');
		expect(liste(container)?.textContent).not.toContain('Chargement…');
		expect(await entree(input)).toBe(false);
		expect(onSelect).not.toHaveBeenCalled();

		await saisir(input, 'Mar');
		await bas(input);
		expect(await entree(input)).toBe(false);
		expect(onSelect).toHaveBeenCalledWith(MARCEL);
	});

	it('d-sexies. frappe pendant une recherche en vol, départ avant 300 ms, retour : la recherche du champ non touché repart, sans « Chargement… » durable', async () => {
		const { input, container } = await focusPose(DUPONT);
		const m = enVol();
		await saisir(input, 'Ma'); // en vol
		await taper(input, 'Mar');
		await quitter(input);
		await focus(input);
		expect(listContactsMock).toHaveBeenLastCalledWith(expect.objectContaining({ search: '' }));
		await laisser();
		expect(liste(container)?.textContent).not.toContain('Chargement…');
		m.tenir([MARTIN]);
		await laisser();
		expect(liste(container)?.textContent).not.toContain('Chargement…');
		expect(options(container)).toHaveLength(3);
	});

	it('e. champ non touché : Entrée laisse partir (non empêchée), rien choisi ; flèche bas désigne le PREMIER, Entrée le choisit', async () => {
		const { input, container, onSelect } = await focusPose(DUPONT);
		expect(options(container)).toHaveLength(3);
		expect(designees(container)).toEqual([]);
		expect(await entree(input)).toBe(true);
		expect(onSelect).not.toHaveBeenCalled();

		await bas(input);
		expect(designees(container)).toEqual(['Dupont SA']);
		expect(await entree(input)).toBe(false);
		expect(onSelect).toHaveBeenCalledWith(DUPONT);
	});

	it('e-quinquies. focus seul sur un champ non touché : aucune désignation à l’arrivée, Entrée laisse partir', async () => {
		const { input, container, onSelect } = await focusPose(MARTIN);
		expect(designees(container)).toEqual([]);
		expect(input.getAttribute('aria-activedescendant')).toBeNull();
		expect(await entree(input)).toBe(true);
		expect(onSelect).not.toHaveBeenCalled();
	});

	it('e-ter. brouillon blanc + Entrée : empêchée, brouillon jeté, liste fermée, client réaffiché ; l’Entrée suivante part', async () => {
		const { input, container, onSelect } = await focusPose(DUPONT);
		await taper(input, '');
		expect(await entree(input)).toBe(false);
		expect(input.value).toBe('Dupont SA');
		expect(liste(container)).toBeNull();
		expect(await entree(input)).toBe(true);
		expect(onSelect).not.toHaveBeenCalled();
	});

	it('h. composition IME (isComposing, keyCode 229) : Entrée non interceptée, rien choisi', async () => {
		const { input, onSelect } = await focusPose(DUPONT);
		await saisir(input, 'Ma');
		expect(await entree(input, { isComposing: true })).toBe(true);
		expect(await entree(input, { keyCode: 229 } as KeyboardEventInit)).toBe(true);
		expect(onSelect).not.toHaveBeenCalled();
	});

	it('h. touche maintenue (repeat) : Entrée empêchée, rien choisi', async () => {
		const { input, onSelect } = await focusPose(DUPONT);
		await saisir(input, 'Ma');
		expect(await entree(input, { repeat: true })).toBe(false);
		expect(onSelect).not.toHaveBeenCalled();
	});
});

describe('ContactPicker — la désignation (D3)', () => {
	it('e-quater. liste vide au focus, flèche bas relance et désigne le premier ; flèche haut, le dernier', async () => {
		listContactsMock.mockImplementationOnce(async () => reponse([]));
		const { input, container, onSelect } = await focusPose(DUPONT);
		expect(liste(container)?.textContent).toContain('Aucun contact');

		listContactsMock.mockImplementation(async () => reponse([DUPONT, MARTIN]));
		await bas(input);
		await laisser();
		expect(designees(container)).toEqual(['Dupont SA']);
		expect(await entree(input)).toBe(false);
		expect(onSelect).toHaveBeenCalledWith(DUPONT);
		await laisser();

		await haut(input);
		await laisser();
		expect(designees(container)).toEqual(['Martin Sàrl']);
	});

	it('e-quater-bis (i). une relance abandonnée par le départ du champ ne désigne rien plus tard', async () => {
		listContactsMock.mockImplementationOnce(async () => {
			throw new Error('réseau');
		});
		const { input, container, onSelect } = await focusPose(DUPONT);
		const m = enVol();
		await bas(input); // relance, en vol
		await quitter(input);
		m.tenir([DUPONT, MARTIN]);
		await laisser();
		await focus(input);
		await laisser();
		expect(options(container).length).toBeGreaterThan(0);
		expect(designees(container)).toEqual([]);
		expect(await entree(input)).toBe(true);
		expect(onSelect).not.toHaveBeenCalled();
	});

	it('e-quater-bis (ii). relance sous « Zzz » abandonnée par la frappe : la réponse de « » ne désigne rien ; Entrée jette le brouillon', async () => {
		const { input, container, onSelect } = await focusPose(DUPONT);
		await saisir(input, 'Zzz');
		const m = enVol();
		await bas(input); // relance sous « Zzz », en vol
		await saisir(input, ''); // champ vidé : réponse de « » (trois contacts)
		m.tenir([MARTIN]);
		await laisser();
		expect(options(container).length).toBeGreaterThan(0);
		expect(designees(container)).toEqual([]);
		expect(await entree(input)).toBe(false);
		expect(input.value).toBe('Dupont SA');
		expect(onSelect).not.toHaveBeenCalled();
	});

	it('e-sexies. « Zzz » sans résultat, flèche : la relance cherche « Zzz », non « »', async () => {
		const { input } = await focusPose(DUPONT);
		await saisir(input, 'Zzz');
		// La flèche RELANCE (un appel de plus) : sans ce compte, le dernier appel
		// serait encore celui de la frappe et le cas passerait à vide.
		const avant = listContactsMock.mock.calls.length;
		await bas(input);
		expect(listContactsMock.mock.calls.length).toBe(avant + 1);
		expect(listContactsMock).toHaveBeenLastCalledWith(expect.objectContaining({ search: 'Zzz' }));
	});

	it('e-bis. liste fermée par Échap : flèche bas rouvre et désigne le premier ; flèche haut, le dernier', async () => {
		const { input, container } = await focusPose(DUPONT);
		await saisir(input, 'Ma');
		await echap(input);
		expect(liste(container)).toBeNull();
		await bas(input);
		expect(designees(container)).toEqual(['Martin Sàrl']);
		await echap(input);
		await haut(input);
		expect(designees(container)).toEqual(['Marcel et Cie']);
	});

	it('e-bis, cas voisin. « Ma », Échap avant la réponse : la réponse arrive liste fermée sans rien désigner ; flèche bas → le premier', async () => {
		const { input, container } = await focusPose(DUPONT);
		const m = enVol();
		await saisir(input, 'Ma'); // en vol
		await echap(input);
		m.tenir([MARTIN, MARCEL]);
		await laisser();
		expect(liste(container)).toBeNull();
		await bas(input);
		expect(designees(container)).toEqual(['Martin Sàrl']);
	});

	it('e-bis-bis. relance par la flèche, Échap avant la réponse : rien désigné liste fermée, plus de « Chargement… » ; flèche bas → le premier', async () => {
		listContactsMock.mockImplementationOnce(async () => reponse([]));
		const { input, container } = await focusPose(DUPONT);
		const m = enVol();
		await bas(input); // relance, en vol
		expect(liste(container)?.textContent).toContain('Chargement…');
		await echap(input);
		m.tenir([DUPONT, MARTIN]);
		await laisser();
		expect(liste(container)).toBeNull();

		await bas(input);
		expect(liste(container)?.textContent).not.toContain('Chargement…');
		expect(designees(container)).toEqual(['Dupont SA']);
	});

	it('e-bis-bis, second volet. « Mar », Échap pendant l’anti-rebond ; Entrée rouvre sur la réponse à « Mar »', async () => {
		const { input, container, onSelect } = await focusPose(DUPONT);
		await taper(input, 'Mar');
		await echap(input);
		await laisser(ANTI_REBOND);
		expect(liste(container)).toBeNull();
		expect(await entree(input)).toBe(false);
		expect(designees(container)).toEqual(['Martin Sàrl']);
		expect(options(container)).toHaveLength(2);
		expect(await entree(input)).toBe(false);
		expect(onSelect).toHaveBeenCalledWith(MARTIN);
	});

	it('f-bis. rendu : l’option désignée porte CLASSE_DESIGNEE, les autres CLASSE_SURVOL, aucune `bg-muted`', async () => {
		const { input, container } = await focusPose(DUPONT);
		await saisir(input, 'Ma');
		const [martin, marcel] = options(container);
		expect(porte(martin, CLASSE_DESIGNEE)).toBe(true);
		expect(porte(martin, CLASSE_SURVOL)).toBe(false);
		expect(porte(marcel, CLASSE_SURVOL)).toBe(true);
		expect(porte(marcel, CLASSE_DESIGNEE)).toBe(false);
		for (const o of options(container)) {
			expect(o.className).not.toMatch(/\bbg-muted\b/);
		}
	});

	it('f-ter. sous-ligne du numéro IDE lisible dans les deux états, sans variante de survol sur l’option désignée', async () => {
		const THEME = blocTheme(readFileSync('src/app.css', 'utf8'));
		const lire = (nom: string) => {
			const v = jetonDans(nom, THEME);
			if (!v) throw new Error(`--color-${nom} absent du @theme`);
			return v;
		};
		const fond = (classes: string) => {
			const j = jetonsDesClasses(classes).filter((x) => x.utilite === 'bg');
			expect(j).toHaveLength(1);
			return lire(j[0].jeton);
		};
		const sousLigne = (o: HTMLElement) => o.querySelector('.text-xs') as HTMLElement;

		const { input, container } = await focusPose(DUPONT);
		await saisir(input, 'Mar');

		// Martin (numéro IDE) désigné.
		const designee = sousLigne(option(container, 'Martin Sàrl'));
		expect(designee.classList.contains('text-primary-foreground')).toBe(true);
		expect(designee.classList.contains('text-text-muted')).toBe(false);
		expect(Array.from(designee.classList).filter((c) => c.startsWith('group-hover:'))).toEqual([]);
		const texteDesigne = jetonsDesClasses(designee.className).filter(
			(x) => x.utilite === 'text' && x.jeton !== 'xs'
		);
		expect(texteDesigne).toHaveLength(1);
		expect(contrastRatio(lire(texteDesigne[0].jeton), fond(CLASSE_DESIGNEE))).toBeGreaterThanOrEqual(4.5);

		// Marcel désigné : Martin, non désigné, survolé.
		await bas(input);
		expect(designees(container)).toEqual(['Marcel et Cie']);
		const survolee = sousLigne(option(container, 'Martin Sàrl'));
		expect(survolee.classList.contains('text-text-muted')).toBe(true);
		const auSurvol = Array.from(survolee.classList).filter((c) => c.startsWith('group-hover:text-'));
		expect(auSurvol).toHaveLength(1);
		const jetonSurvol = auSurvol[0].slice('group-hover:text-'.length);
		expect(contrastRatio(lire(jetonSurvol), fond(CLASSE_SURVOL))).toBeGreaterThanOrEqual(4.5);
	});

	it('f-quater. `mousedown` sur la liste et sur une option : empêché ; sur la liste, le brouillon reste et la liste aussi', async () => {
		const { input, container, onSelect } = await focusPose(DUPONT);
		await saisir(input, 'Ma');
		expect(await fireEvent.mouseDown(liste(container)!)).toBe(false);
		expect(input.value).toBe('Ma');
		expect(liste(container)).not.toBeNull();
		expect(await fireEvent.mouseDown(option(container, 'Marcel et Cie'))).toBe(false);
		expect(onSelect).toHaveBeenCalledWith(MARCEL);
	});
});

describe('ContactPicker — le client change hors du champ (D1)', () => {
	it('g. brouillon affiché, le client change : le champ montre le nouveau ; puis `null` : champ vide', async () => {
		const { input, rerender, onSelect } = await focusPose(DUPONT);
		await taper(input, 'zz');
		await rerender({ selected: MARTIN, onSelect });
		expect(input.value).toBe('Martin Sàrl');
		await rerender({ selected: null, onSelect });
		expect(input.value).toBe('');
	});

	it('g. (V2-5) liste ouverte, option désignée, le client change : liste fermée, rien désigné, Entrée n’est pas un choix', async () => {
		const { input, container, rerender, onSelect } = await focusPose(DUPONT);
		await saisir(input, 'Ma');
		expect(designees(container)).toEqual(['Martin Sàrl']);
		await rerender({ selected: MARCEL, onSelect });
		expect(liste(container)).toBeNull();
		expect(designees(container)).toEqual([]);
		expect(await entree(input)).toBe(true);
		expect(onSelect).not.toHaveBeenCalled();
	});

	it('g-bis. recherche « Mar » en vol, le client change, la réponse arrive : aucune option de « Mar » ; la flèche relance la recherche du champ non touché', async () => {
		const { input, container, rerender, onSelect } = await focusPose(DUPONT);
		const m = enVol();
		await saisir(input, 'Mar'); // en vol
		await rerender({ selected: MARTIN, onSelect });
		m.tenir([MARTIN, MARCEL]);
		await laisser();
		expect(options(container)).toEqual([]);

		const avant = listContactsMock.mock.calls.length;
		await bas(input);
		expect(listContactsMock.mock.calls.length).toBe(avant + 1);
		expect(listContactsMock).toHaveBeenLastCalledWith(expect.objectContaining({ search: '' }));
	});
});

describe('ContactPicker — le brouillon gardé au départ du champ (D4, décision de Guy du 2026-11-17)', () => {
	it('D4-a. `keepDraftOnBlur` : « Mar » tapé, champ quitté → le texte reste, le parent est prévenu ; au retour, la recherche reprend « Mar » et désigne le premier', async () => {
		const onDraftChange = vi.fn();
		const m = monter(DUPONT, { keepDraftOnBlur: true, onDraftChange });
		await focus(m.input);
		await laisser();
		expect(onDraftChange).toHaveBeenLastCalledWith(false);

		await saisir(m.input, 'Mar');
		expect(onDraftChange).toHaveBeenLastCalledWith(true);
		await quitter(m.input);
		await laisser(ANTI_REBOND);
		expect(m.input.value).toBe('Mar');
		expect(liste(m.container)).toBeNull();
		expect(onDraftChange).toHaveBeenLastCalledWith(true);
		expect(m.onSelect).not.toHaveBeenCalled();

		await focus(m.input);
		expect(listContactsMock).toHaveBeenLastCalledWith(expect.objectContaining({ search: 'Mar' }));
		await laisser();
		expect(designees(m.container)).toEqual(['Martin Sàrl']);
		expect(await entree(m.input)).toBe(false);
		expect(m.onSelect).toHaveBeenCalledWith(MARTIN);
		await laisser();
		expect(m.input.value).toBe('Martin Sàrl');
		expect(onDraftChange).toHaveBeenLastCalledWith(false);
	});

	it('D4-b. `keepDraftOnBlur` : brouillon blanc, champ quitté → jeté (D1), le client reparaît, le parent est prévenu', async () => {
		const onDraftChange = vi.fn();
		const m = monter(DUPONT, { keepDraftOnBlur: true, onDraftChange });
		await focus(m.input);
		await laisser();
		await taper(m.input, '   ');
		await laisser();
		await quitter(m.input);
		await laisser();
		expect(m.input.value).toBe('Dupont SA');
		expect(onDraftChange).toHaveBeenLastCalledWith(false);
	});

	it('D4-c. sans `keepDraftOnBlur` (filtres) : le texte est jeté au départ, le parent reçoit `false`', async () => {
		const onDraftChange = vi.fn();
		const m = monter(DUPONT, { onDraftChange });
		await focus(m.input);
		await laisser();
		await taper(m.input, 'zzz');
		await laisser();
		expect(onDraftChange).toHaveBeenLastCalledWith(true);
		await quitter(m.input);
		await laisser();
		expect(m.input.value).toBe('Dupont SA');
		expect(onDraftChange).toHaveBeenLastCalledWith(false);
	});
});

describe('ContactPicker — seul le bouton principal choisit (revue P1, E3)', () => {
	it('clic droit ou du milieu sur une option : rien n’est choisi, la liste reste ; clic principal : choisi', async () => {
		const { input, container, onSelect } = await focusPose(DUPONT);
		await saisir(input, 'Ma');
		expect(await fireEvent.mouseDown(option(container, 'Marcel et Cie'), { button: 2 })).toBe(false);
		expect(await fireEvent.mouseDown(option(container, 'Marcel et Cie'), { button: 1 })).toBe(false);
		expect(onSelect).not.toHaveBeenCalled();
		expect(liste(container)).not.toBeNull();
		await fireEvent.mouseDown(option(container, 'Marcel et Cie'), { button: 0 });
		expect(onSelect).toHaveBeenCalledWith(MARCEL);
	});
});
