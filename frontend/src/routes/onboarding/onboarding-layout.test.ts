// Story 15-7b2 (AC 4, test 12) — après une remise à zéro, la réponse porte
// `isStub: true` (la société est remise à l'état provisoire EN PLACE) : le
// bandeau « nom provisoire » du layout d'onboarding s'affiche. `isStub` est un
// état de module en lecture seule : on le pose en appelant `resetDemo()` sur
// l'API mockée, avant le rendu.
import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/svelte';
import { createRawSnippet } from 'svelte';

vi.mock('$lib/features/onboarding/onboarding.api', () => ({
	fetchState: vi.fn(),
	setLanguage: vi.fn(),
	setMode: vi.fn(),
	seedDemo: vi.fn(),
	resetDemo: vi.fn(),
}));
vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_key: string, fallback: string) => fallback,
	loadI18nMessages: vi.fn(),
}));
vi.mock('$lib/shared/utils/app-version.svelte', () => ({
	appVersion: { value: '0.13.0' },
}));

import * as api from '$lib/features/onboarding/onboarding.api';
import { onboardingState } from '$lib/features/onboarding/onboarding.svelte';
import Layout from './+layout.svelte';

const children = createRawSnippet(() => ({ render: () => '<p>contenu</p>' }));

async function renderAfterReset(isStub: boolean) {
	vi.mocked(api.resetDemo).mockResolvedValue({
		stepCompleted: 0,
		isDemo: false,
		uiMode: null,
		isStub,
	});
	await onboardingState.resetDemo();
	return render(Layout, { props: { children } });
}

describe('layout d’onboarding — bandeau de la société provisoire', () => {
	it('après une remise à zéro (isStub vrai), le bandeau s’affiche', async () => {
		const { unmount } = await renderAfterReset(true);
		expect(onboardingState.isStub).toBe(true);
		expect(screen.queryByTestId('onboarding-stub-notice')).not.toBeNull();
		expect(screen.getByText('contenu')).toBeTruthy();
		unmount();
	});

	it('isStub faux : pas de bandeau', async () => {
		const { unmount } = await renderAfterReset(false);
		expect(screen.queryByTestId('onboarding-stub-notice')).toBeNull();
		unmount();
	});
});
