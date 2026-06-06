// Story 17-2b — Tests Vitest pour le util clipboard HTTP-LAN-safe.

import { describe, it, expect, vi, afterEach } from 'vitest';
import { copyToClipboard } from './clipboard';

describe('copyToClipboard', () => {
	afterEach(() => {
		vi.unstubAllGlobals();
		vi.restoreAllMocks();
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
});
