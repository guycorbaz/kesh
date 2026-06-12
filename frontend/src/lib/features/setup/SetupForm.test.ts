// Story v011-5 — Tests Vitest pour `SetupForm.svelte`.
//
// Couvre :
// - Validation client : password < 12 chars, password mismatch, username vide.
// - Submit désactivé tant que form invalide.
// - Submit happy path → setupAdmin appelé avec args trimmés.
// - Story 17-4d : champ email optionnel (vide → undefined, sans `@` → bloqué,
//   renseigné → transmis trimé).

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, fireEvent } from '@testing-library/svelte';

vi.mock('./setup.api', () => ({
	setupAdmin: vi.fn().mockResolvedValue(undefined),
}));

vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_key: string, fallback: string) => fallback,
}));

vi.mock('$app/navigation', () => ({
	goto: vi.fn().mockResolvedValue(undefined),
}));

vi.mock('$lib/shared/utils/api-client', () => ({
	isApiError: (e: unknown) =>
		typeof e === 'object' && e !== null && 'code' in e && 'status' in e,
}));

import SetupForm from './SetupForm.svelte';
import { setupAdmin } from './setup.api';

describe('SetupForm validation', () => {
	beforeEach(() => {
		vi.clearAllMocks();
	});

	it('submit button is disabled by default (empty form)', () => {
		const { getByTestId } = render(SetupForm);
		const submit = getByTestId('setup-submit') as HTMLButtonElement;
		expect(submit.disabled).toBe(true);
	});

	it('submit is disabled when password < 12 chars', async () => {
		const { getByTestId } = render(SetupForm);
		const username = getByTestId('setup-username') as HTMLInputElement;
		const pw = getByTestId('setup-password') as HTMLInputElement;
		const pwConfirm = getByTestId('setup-password-confirm') as HTMLInputElement;
		const submit = getByTestId('setup-submit') as HTMLButtonElement;

		await fireEvent.input(username, { target: { value: 'admin' } });
		await fireEvent.input(pw, { target: { value: 'short-10ch' } });
		await fireEvent.input(pwConfirm, { target: { value: 'short-10ch' } });

		expect(submit.disabled).toBe(true);
	});

	it('submit is disabled on password mismatch + shows error', async () => {
		const { getByTestId, queryByTestId } = render(SetupForm);
		const username = getByTestId('setup-username') as HTMLInputElement;
		const pw = getByTestId('setup-password') as HTMLInputElement;
		const pwConfirm = getByTestId('setup-password-confirm') as HTMLInputElement;
		const submit = getByTestId('setup-submit') as HTMLButtonElement;

		await fireEvent.input(username, { target: { value: 'admin' } });
		await fireEvent.input(pw, { target: { value: 'valid-12-chars-pw' } });
		await fireEvent.input(pwConfirm, { target: { value: 'DIFFERENT-12-chars-pw' } });

		expect(queryByTestId('setup-password-mismatch')).toBeTruthy();
		expect(submit.disabled).toBe(true);
	});

	it('submit is enabled when all fields valid', async () => {
		const { getByTestId } = render(SetupForm);
		const username = getByTestId('setup-username') as HTMLInputElement;
		const pw = getByTestId('setup-password') as HTMLInputElement;
		const pwConfirm = getByTestId('setup-password-confirm') as HTMLInputElement;
		const submit = getByTestId('setup-submit') as HTMLButtonElement;

		await fireEvent.input(username, { target: { value: 'admin' } });
		await fireEvent.input(pw, { target: { value: 'valid-12-chars-pw' } });
		await fireEvent.input(pwConfirm, { target: { value: 'valid-12-chars-pw' } });

		expect(submit.disabled).toBe(false);
	});

	it('submit happy path → setupAdmin called with trimmed args', async () => {
		const { getByTestId, container } = render(SetupForm);
		const username = getByTestId('setup-username') as HTMLInputElement;
		const pw = getByTestId('setup-password') as HTMLInputElement;
		const pwConfirm = getByTestId('setup-password-confirm') as HTMLInputElement;

		await fireEvent.input(username, { target: { value: '  admin  ' } });
		await fireEvent.input(pw, { target: { value: 'valid-12-chars-pw' } });
		await fireEvent.input(pwConfirm, { target: { value: 'valid-12-chars-pw' } });

		const form = container.querySelector('form') as HTMLFormElement;
		await fireEvent.submit(form);

		// setupAdmin appelé avec username trim() ; email vide → undefined (omis
		// du JSON, Story 17-4d DD-5).
		expect(setupAdmin).toHaveBeenCalledTimes(1);
		expect(setupAdmin).toHaveBeenCalledWith('admin', 'valid-12-chars-pw', undefined);
	});

	// Story 17-4d (AC21/DD-5) — champ email optionnel.

	it('submit is disabled when email is non-empty without @', async () => {
		const { getByTestId, queryByTestId } = render(SetupForm);
		const username = getByTestId('setup-username') as HTMLInputElement;
		const pw = getByTestId('setup-password') as HTMLInputElement;
		const pwConfirm = getByTestId('setup-password-confirm') as HTMLInputElement;
		const email = getByTestId('setup-email') as HTMLInputElement;
		const submit = getByTestId('setup-submit') as HTMLButtonElement;

		await fireEvent.input(username, { target: { value: 'admin' } });
		await fireEvent.input(pw, { target: { value: 'valid-12-chars-pw' } });
		await fireEvent.input(pwConfirm, { target: { value: 'valid-12-chars-pw' } });
		await fireEvent.input(email, { target: { value: 'pas-un-email' } });

		expect(queryByTestId('setup-email-invalid')).toBeTruthy();
		expect(submit.disabled).toBe(true);
	});

	it('submit with email → setupAdmin receives trimmed email', async () => {
		const { getByTestId, container } = render(SetupForm);
		const username = getByTestId('setup-username') as HTMLInputElement;
		const pw = getByTestId('setup-password') as HTMLInputElement;
		const pwConfirm = getByTestId('setup-password-confirm') as HTMLInputElement;
		const email = getByTestId('setup-email') as HTMLInputElement;

		await fireEvent.input(username, { target: { value: 'admin' } });
		await fireEvent.input(pw, { target: { value: 'valid-12-chars-pw' } });
		await fireEvent.input(pwConfirm, { target: { value: 'valid-12-chars-pw' } });
		await fireEvent.input(email, { target: { value: '  admin@example.ch  ' } });

		const form = container.querySelector('form') as HTMLFormElement;
		await fireEvent.submit(form);

		expect(setupAdmin).toHaveBeenCalledWith('admin', 'valid-12-chars-pw', 'admin@example.ch');
	});
});
