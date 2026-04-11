import { beforeEach, describe, expect, it, vi } from 'vitest';

vi.mock('svelte-sonner', () => ({
	toast: {
		success: vi.fn(),
		info: vi.fn(),
		warning: vi.fn(),
		error: vi.fn()
	}
}));

import { toast } from 'svelte-sonner';
import { notifyError, notifyInfo, notifySuccess, notifyWarning } from './notify';

describe('notify helpers', () => {
	beforeEach(() => {
		vi.clearAllMocks();
	});

	describe('notifySuccess', () => {
		it('appelle toast.success avec durée 4000ms', () => {
			notifySuccess('Opération réussie');
			expect(toast.success).toHaveBeenCalledWith('Opération réussie', {
				description: undefined,
				duration: 4000
			});
		});

		it('passe description optionnelle', () => {
			notifySuccess('Titre', 'Détails');
			expect(toast.success).toHaveBeenCalledWith('Titre', {
				description: 'Détails',
				duration: 4000
			});
		});
	});

	describe('notifyInfo', () => {
		it('appelle toast.info avec durée 4000ms', () => {
			notifyInfo('Information');
			expect(toast.info).toHaveBeenCalledWith('Information', {
				description: undefined,
				duration: 4000
			});
		});
	});

	describe('notifyWarning', () => {
		it('appelle toast.warning avec durée 6000ms (plus visible)', () => {
			notifyWarning('Avertissement');
			expect(toast.warning).toHaveBeenCalledWith('Avertissement', {
				description: undefined,
				duration: 6000
			});
		});

		it('passe description optionnelle', () => {
			notifyWarning('Import partiel', '3 lignes ignorées');
			expect(toast.warning).toHaveBeenCalledWith('Import partiel', {
				description: '3 lignes ignorées',
				duration: 6000
			});
		});
	});

	describe('notifyError', () => {
		it('appelle toast.error avec durée 6000ms', () => {
			notifyError('Erreur');
			expect(toast.error).toHaveBeenCalledWith('Erreur', {
				description: undefined,
				duration: 6000
			});
		});

		it('passe description optionnelle', () => {
			notifyError('Erreur API', 'Connexion refusée');
			expect(toast.error).toHaveBeenCalledWith('Erreur API', {
				description: 'Connexion refusée',
				duration: 6000
			});
		});
	});
});
