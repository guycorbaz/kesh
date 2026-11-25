// Story 1582 (#1582) — le logo et, dessous, le nom de la comptabilité ouverte.
// jsdom n'a pas Tailwind : l'espace du nom accessible vient de deux éléments `div`
// (blocs par nature, aussi sous jsdom), et la coupe se vérifie par ses classes.

import { describe, it, expect } from 'vitest';
import { render } from '@testing-library/svelte';
import AppBrand from './AppBrand.svelte';

describe('AppBrand (Story 1582)', () => {
	it('société : lien « Kesh Dupont Sàrl », deux blocs, nom titré', () => {
		const { getByRole, getByTestId } = render(AppBrand, { name: 'Dupont Sàrl' });
		const lien = getByRole('link', { name: 'Kesh Dupont Sàrl' });
		expect(lien.getAttribute('href')).toBe('/');
		const nom = getByTestId('company-name');
		expect(lien.contains(nom)).toBe(true);
		expect(nom.textContent).toBe('Dupont Sàrl');
		expect(nom.getAttribute('title')).toBe('Dupont Sàrl');
		// Deux blocs l'un sous l'autre : des `div` (display:block), non des `span` en ligne.
		const blocs = Array.from(lien.children);
		expect(blocs.map((b) => b.tagName)).toEqual(['DIV', 'DIV']);
		expect(blocs[0].textContent).toBe('Kesh');
		expect(blocs[1]).toBe(nom);
		expect(lien.classList.contains('flex-col')).toBe(true);
	});

	it('sans nom : lien « Kesh » seul, aucun company-name', () => {
		const { getByRole, queryByTestId } = render(AppBrand, { name: null });
		// sonde positive : le lien existe
		expect(getByRole('link', { name: 'Kesh' })).toBeTruthy();
		expect(queryByTestId('company-name')).toBeNull();
	});

	it('nom de 70 caractères : texte complet dans le DOM, coupé par la CSS, title complet', () => {
		const long = 'S'.repeat(70);
		const { getByTestId } = render(AppBrand, { name: long });
		const nom = getByTestId('company-name');
		expect(nom.textContent).toBe(long);
		expect(nom.getAttribute('title')).toBe(long);
		expect(nom.classList.contains('truncate')).toBe(true);
		expect(nom.classList.contains('max-w-56')).toBe(true);
	});

	it('du HTML dans le nom est rendu en texte', () => {
		const { getByTestId } = render(AppBrand, { name: '<b>x</b>' });
		const nom = getByTestId('company-name');
		expect(nom.textContent).toBe('<b>x</b>');
		expect(nom.querySelector('b')).toBeNull();
	});

	it('personne physique : « Marie Dupont » rendu tel quel', () => {
		const { getByTestId, getByRole } = render(AppBrand, { name: 'Marie Dupont' });
		expect(getByTestId('company-name').textContent).toBe('Marie Dupont');
		expect(getByRole('link', { name: 'Kesh Marie Dupont' })).toBeTruthy();
	});
});
