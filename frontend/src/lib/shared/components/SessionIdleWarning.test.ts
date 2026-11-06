// Story v0174-session-2 (0.17.4, #895, AC-3) — l'avertissement avant la
// fermeture pour inactivité, piloté par ses propriétés (C-v0174s-20).

import { describe, it, expect, vi, afterEach } from 'vitest';
import { render, cleanup } from '@testing-library/svelte';

vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_k: string, f: string, args?: Record<string, string | number>) =>
		args ? f.replace(/\{\s*\$(\w+)\s*\}/g, (_, k) => String(args[k] ?? '')) : f,
	loadI18nMessages: vi.fn(),
}));

import SessionIdleWarning from './SessionIdleWarning.svelte';

afterEach(() => {
	cleanup();
	document.body.innerHTML = '';
});

describe('SessionIdleWarning (#895, AC-3)', () => {
	it('compte à rebours m:ss arrondi au supérieur ; la phrase annoncée ne bouge pas', async () => {
		const onStay = vi.fn();
		const { getByTestId, rerender } = render(SessionIdleWarning, { remainingMs: 59_400, onStay });
		expect(getByTestId('session-idle-warning-countdown').textContent?.trim()).toBe('Fermeture dans 1:00.');
		const annonce = getByTestId('session-idle-warning').querySelector('[role="alert"]');
		expect(annonce).not.toBeNull();
		const texte = annonce?.textContent;
		await rerender({ remainingMs: 58_400 });
		expect(getByTestId('session-idle-warning-countdown').textContent?.trim()).toBe('Fermeture dans 0:59.');
		// Le compte à rebours n'est pas dans l'élément annoncé (W6).
		expect(annonce?.textContent).toBe(texte);
		expect(annonce?.contains(getByTestId('session-idle-warning-countdown'))).toBe(false);
		expect(texte).toContain('Rester connecté');
	});

	it('le compte à rebours est muet pour un lecteur d’écran', () => {
		const { getByTestId } = render(SessionIdleWarning, { remainingMs: 30_000, onStay: vi.fn() });
		expect(getByTestId('session-idle-warning-countdown').getAttribute('aria-hidden')).toBe('true');
	});

	it('un clic sur « Rester connecté » appelle onStay une fois', () => {
		const onStay = vi.fn();
		const { getByTestId } = render(SessionIdleWarning, { remainingMs: 30_000, onStay });
		const bouton = getByTestId('session-idle-warning-stay');
		expect(bouton.tagName).toBe('BUTTON');
		expect(bouton.getAttribute('type')).toBe('button');
		bouton.click();
		expect(onStay).toHaveBeenCalledTimes(1);
	});

	it('le rendu ne déplace pas le focus (C-v0174s-11)', () => {
		const input = document.createElement('input');
		document.body.appendChild(input);
		input.focus();
		expect(document.activeElement).toBe(input);
		render(SessionIdleWarning, { remainingMs: 30_000, onStay: vi.fn() });
		expect(document.activeElement).toBe(input);
	});

	it('un pointerdown sur l’avis n’atteint pas document en bouillonnement ; window en capture le voit (C-v0174s-13)', () => {
		const { getByTestId } = render(SessionIdleWarning, { remainingMs: 30_000, onStay: vi.fn() });
		const surDocument = vi.fn();
		const surFenetreCapture = vi.fn();
		document.addEventListener('pointerdown', surDocument);
		window.addEventListener('pointerdown', surFenetreCapture, { capture: true });
		try {
			getByTestId('session-idle-warning-stay').dispatchEvent(new Event('pointerdown', { bubbles: true }));
			expect(surFenetreCapture).toHaveBeenCalledTimes(1);
			expect(surDocument).not.toHaveBeenCalled();
			// Témoin : hors de l'avis, le même événement atteint document.
			document.body.dispatchEvent(new Event('pointerdown', { bubbles: true }));
			expect(surDocument).toHaveBeenCalledTimes(1);
		} finally {
			document.removeEventListener('pointerdown', surDocument);
			window.removeEventListener('pointerdown', surFenetreCapture, { capture: true });
		}
	});

	it('au-dessus d’un dialogue : pointer-events auto et z-index > 50 (classes)', () => {
		const { getByTestId } = render(SessionIdleWarning, { remainingMs: 30_000, onStay: vi.fn() });
		const racine = getByTestId('session-idle-warning');
		expect(racine.classList.contains('pointer-events-auto')).toBe(true);
		expect(racine.classList.contains('z-[60]')).toBe(true);
		expect(racine.classList.contains('fixed')).toBe(true);
	});
});
