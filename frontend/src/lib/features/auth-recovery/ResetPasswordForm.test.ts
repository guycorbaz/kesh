// Story 17-4d Pass 1 (PD6) — Tests Vitest du composant ResetPasswordForm.
//
// Couvre les transitions d'état : token null/vide → lien invalide sans appel
// API ; validation locale (12 chars Unicode-safe + correspondance) ; succès →
// CTA login ; 400 INVALID_OR_EXPIRED_TOKEN → bascule lien invalide.

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, fireEvent, waitFor } from '@testing-library/svelte';

vi.mock('./auth-recovery.api', () => ({
	resetPassword: vi.fn().mockResolvedValue(undefined),
}));

vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_key: string, fallback: string) => fallback,
}));

vi.mock('$lib/shared/utils/api-client', () => ({
	isApiError: (e: unknown) =>
		typeof e === 'object' && e !== null && 'code' in e && 'status' in e,
}));

import ResetPasswordForm from './ResetPasswordForm.svelte';
import { resetPassword } from './auth-recovery.api';

const VALID_PW = 'valid-12-chars-pw';

describe('ResetPasswordForm', () => {
	beforeEach(() => {
		vi.clearAllMocks();
	});

	it('token null → état lien invalide, pas de formulaire ni d’appel API', () => {
		const { queryByTestId } = render(ResetPasswordForm, { props: { token: null } });
		expect(queryByTestId('reset-invalid-link')).toBeTruthy();
		expect(queryByTestId('reset-request-new-link')).toBeTruthy();
		expect(queryByTestId('reset-submit')).toBeNull();
		expect(resetPassword).not.toHaveBeenCalled();
	});

	it('token espaces-seulement → état lien invalide', () => {
		const { queryByTestId } = render(ResetPasswordForm, { props: { token: '   ' } });
		expect(queryByTestId('reset-invalid-link')).toBeTruthy();
		expect(queryByTestId('reset-submit')).toBeNull();
	});

	it('submit désactivé tant que la validation locale échoue (longueur + correspondance)', async () => {
		const { getByTestId, queryByTestId } = render(ResetPasswordForm, {
			props: { token: 'tok27' },
		});
		const pw = getByTestId('reset-password') as HTMLInputElement;
		const confirm = getByTestId('reset-password-confirm') as HTMLInputElement;
		const submit = getByTestId('reset-submit') as HTMLButtonElement;

		expect(submit.disabled).toBe(true);

		await fireEvent.input(pw, { target: { value: 'court' } });
		await fireEvent.input(confirm, { target: { value: 'court' } });
		expect(submit.disabled).toBe(true);

		await fireEvent.input(pw, { target: { value: VALID_PW } });
		await fireEvent.input(confirm, { target: { value: 'DIFFERENT-12-chars' } });
		expect(queryByTestId('reset-password-mismatch')).toBeTruthy();
		expect(submit.disabled).toBe(true);

		await fireEvent.input(confirm, { target: { value: VALID_PW } });
		expect(submit.disabled).toBe(false);
	});

	it('succès → resetPassword(token, pw) appelé + état succès avec CTA login', async () => {
		const { getByTestId, container, queryByTestId } = render(ResetPasswordForm, {
			props: { token: 'tok27' },
		});
		await fireEvent.input(getByTestId('reset-password'), { target: { value: VALID_PW } });
		await fireEvent.input(getByTestId('reset-password-confirm'), {
			target: { value: VALID_PW },
		});
		await fireEvent.submit(container.querySelector('form') as HTMLFormElement);

		await waitFor(() => expect(queryByTestId('reset-success')).toBeTruthy());
		expect(resetPassword).toHaveBeenCalledWith('tok27', VALID_PW);
		expect(queryByTestId('reset-login-cta')).toBeTruthy();
		expect(queryByTestId('reset-submit')).toBeNull();
	});

	it('400 INVALID_OR_EXPIRED_TOKEN → bascule en état lien invalide (générique DC4)', async () => {
		vi.mocked(resetPassword).mockRejectedValueOnce({
			code: 'INVALID_OR_EXPIRED_TOKEN',
			message: 'Lien invalide ou expiré',
			status: 400,
		});
		const { getByTestId, container, queryByTestId } = render(ResetPasswordForm, {
			props: { token: 'tok-perime' },
		});
		await fireEvent.input(getByTestId('reset-password'), { target: { value: VALID_PW } });
		await fireEvent.input(getByTestId('reset-password-confirm'), {
			target: { value: VALID_PW },
		});
		await fireEvent.submit(container.querySelector('form') as HTMLFormElement);

		await waitFor(() => expect(queryByTestId('reset-invalid-link')).toBeTruthy());
		expect(queryByTestId('reset-request-new-link')).toBeTruthy();
		expect(queryByTestId('reset-submit')).toBeNull();
	});

	it('429 RATE_LIMITED → message rate-limit, formulaire toujours visible', async () => {
		vi.mocked(resetPassword).mockRejectedValueOnce({
			code: 'RATE_LIMITED',
			message: 'Trop de tentatives',
			status: 429,
		});
		const { getByTestId, container, queryByTestId } = render(ResetPasswordForm, {
			props: { token: 'tok27' },
		});
		await fireEvent.input(getByTestId('reset-password'), { target: { value: VALID_PW } });
		await fireEvent.input(getByTestId('reset-password-confirm'), {
			target: { value: VALID_PW },
		});
		await fireEvent.submit(container.querySelector('form') as HTMLFormElement);

		await waitFor(() => expect(queryByTestId('reset-error')).toBeTruthy());
		expect(queryByTestId('reset-submit')).toBeTruthy();
		expect(queryByTestId('reset-invalid-link')).toBeNull();
	});
});
