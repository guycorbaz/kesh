// Story 17-2b — Tests Vitest pour le util clipboard HTTP-LAN-safe.

import { describe, it, expect, vi, afterEach } from 'vitest';
import { copyToClipboard } from './clipboard';

describe('copyToClipboard', () => {
	// `document.execCommand` est assigné directement (pas via `vi.spyOn`) → non
	// restauré par `restoreAllMocks`. On sauve/restaure manuellement (code-review P1).
	const originalExecCommand = (document as unknown as { execCommand?: unknown }).execCommand;

	afterEach(() => {
		vi.unstubAllGlobals();
		vi.restoreAllMocks();
		(document as unknown as { execCommand: unknown }).execCommand = originalExecCommand;
	});

	it('utilise navigator.clipboard.writeText quand disponible (secure-context)', async () => {
		const writeText = vi.fn().mockResolvedValue(undefined);
		vi.stubGlobal('navigator', { clipboard: { writeText } });
		const ok = await copyToClipboard('kesh_pat_secret');
		expect(ok).toBe(true);
		expect(writeText).toHaveBeenCalledWith('kesh_pat_secret');
	});

	it('fallback execCommand quand navigator.clipboard est absent (HTTP LAN)', async () => {
		// Simule un contexte non-sécurisé : pas de clipboard API.
		vi.stubGlobal('navigator', {});
		const execCommand = vi.fn().mockReturnValue(true);
		// jsdom n'implémente pas execCommand → on le stube.
		(document as unknown as { execCommand: typeof execCommand }).execCommand = execCommand;
		const ok = await copyToClipboard('kesh_pat_secret');
		expect(ok).toBe(true);
		expect(execCommand).toHaveBeenCalledWith('copy');
	});

	it('fallback execCommand quand writeText échoue (permission refusée)', async () => {
		const writeText = vi.fn().mockRejectedValue(new Error('denied'));
		vi.stubGlobal('navigator', { clipboard: { writeText } });
		const execCommand = vi.fn().mockReturnValue(true);
		(document as unknown as { execCommand: typeof execCommand }).execCommand = execCommand;
		const ok = await copyToClipboard('x');
		expect(ok).toBe(true);
		expect(execCommand).toHaveBeenCalledWith('copy');
	});

	it('retourne false si execCommand échoue', async () => {
		vi.stubGlobal('navigator', {});
		const execCommand = vi.fn().mockReturnValue(false);
		(document as unknown as { execCommand: typeof execCommand }).execCommand = execCommand;
		const ok = await copyToClipboard('x');
		expect(ok).toBe(false);
	});

	it('retourne false ET ne laisse pas de textarea orpheline si execCommand throw', async () => {
		vi.stubGlobal('navigator', {});
		const execCommand = vi.fn().mockImplementation(() => {
			throw new Error('SecurityError');
		});
		(document as unknown as { execCommand: typeof execCommand }).execCommand = execCommand;
		const before = document.querySelectorAll('textarea').length;
		const ok = await copyToClipboard('x');
		expect(ok).toBe(false);
		// Le `finally` retire la textarea malgré le throw (pas de fuite DOM).
		expect(document.querySelectorAll('textarea').length).toBe(before);
	});
});
