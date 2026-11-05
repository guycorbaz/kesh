// Story v0174-correctifs-3 (#889, AC-3.6, site 6) — le rappel manuel propose la date du
// poste, et l'y borne : à 00 h 30 le 5 novembre à Zurich, le 5 (non la veille, l'UTC).
import { describe, it, expect, vi, afterEach } from 'vitest';
import { render, cleanup } from '@testing-library/svelte';

vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_k: string, fallback: string) => fallback,
}));

import ManualReminderDialog from './ManualReminderDialog.svelte';
import { pinClock } from '$lib/shared/utils/pin-clock.test-helper';

afterEach(() => cleanup());

describe('ManualReminderDialog — date du jour locale (#889)', () => {
	pinClock();
	it('la date par défaut et son maximum sont le 5 novembre', () => {
		render(ManualReminderDialog, {
			open: true,
			onOpenChange: vi.fn(),
			invoiceLabel: 'F-2026-001',
			maxLevel: 3,
			defaultLevel: 1,
			onConfirm: vi.fn(),
		});
		const input = document.querySelector('input[type="date"]') as HTMLInputElement;
		expect(input.value).toBe('2026-11-05');
		expect(input.max).toBe('2026-11-05');
	});
});
