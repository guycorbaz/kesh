// Story 30-2a (AC6, T-a5) — l'aperçu d'un document à traiter.
//
// i18nMsg rend son repli : les textes vérifiés sont ceux du repli français, qui égale
// le catalogue fr-CH.

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, waitFor, cleanup } from '@testing-library/svelte';

vi.mock('$lib/shared/utils/i18n.svelte', () => ({
	i18nMsg: (_key: string, fallback: string) => fallback,
}));

import DocumentPreview from './DocumentPreview.svelte';

let created: string[];
let revoked: string[];

beforeEach(() => {
	created = [];
	revoked = [];
	let n = 0;
	vi.spyOn(URL, 'createObjectURL').mockImplementation(() => {
		const url = `blob:doc-${++n}`;
		created.push(url);
		return url;
	});
	vi.spyOn(URL, 'revokeObjectURL').mockImplementation((url: string) => {
		revoked.push(url);
	});
});

afterEach(() => {
	cleanup();
	vi.restoreAllMocks();
});

function blob(type: string): Blob {
	return new Blob(['x'], { type });
}

describe('DocumentPreview', () => {
	it('affiche un état de chargement tant que le document n’est pas lu', () => {
		const { getByTestId } = render(DocumentPreview, {
			load: () => new Promise<Blob>(() => {}),
			fileName: 'facture.png',
		});
		expect(getByTestId('document-preview-loading').textContent).toMatch(/Chargement/);
	});

	it('un PNG ou un JPEG s’affiche en <img>, URL blob:, texte alternatif = nom du fichier', async () => {
		for (const type of ['image/png', 'image/jpeg']) {
			const { findByTestId, unmount } = render(DocumentPreview, {
				load: () => Promise.resolve(blob(type)),
				fileName: 'ticket.png',
			});
			const img = (await findByTestId('document-preview-image')) as HTMLImageElement;
			expect(img.getAttribute('src')).toMatch(/^blob:/);
			expect(img.getAttribute('alt')).toBe('ticket.png');
			unmount();
		}
	});

	it('un PDF s’affiche en <iframe> dont le titre accessible est le nom du fichier', async () => {
		const { findByTestId, queryByTestId } = render(DocumentPreview, {
			load: () => Promise.resolve(blob('application/pdf')),
			fileName: 'facture-mai.pdf',
		});
		const frame = (await findByTestId('document-preview-frame')) as HTMLIFrameElement;
		expect(frame.getAttribute('src')).toMatch(/^blob:/);
		expect(frame.getAttribute('title')).toBe('facture-mai.pdf');
		expect(frame.hasAttribute('sandbox')).toBe(false);
		expect(queryByTestId('document-preview-image')).toBeNull();
	});

	it('tout autre type : aucun cadre, un lien « Télécharger » seulement', async () => {
		const { findByTestId, queryByTestId } = render(DocumentPreview, {
			load: () => Promise.resolve(blob('text/html')),
			fileName: 'page.html',
		});
		const link = (await findByTestId('document-preview-download')) as HTMLAnchorElement;
		expect(link.textContent).toMatch(/Télécharger/);
		expect(link.getAttribute('download')).toBe('page.html');
		expect(queryByTestId('document-preview-frame')).toBeNull();
		expect(queryByTestId('document-preview-image')).toBeNull();
	});

	it('410 SOURCE_DOCUMENT_GONE : « Le document n’a pas été restauré »', async () => {
		const { findByTestId } = render(DocumentPreview, {
			load: () =>
				Promise.reject({ code: 'SOURCE_DOCUMENT_GONE', status: 410, message: 'gone' }),
			fileName: 'perdu.pdf',
		});
		expect((await findByTestId('document-preview-gone')).textContent).toMatch(
			/n’a pas été restauré/,
		);
	});

	it('toute autre erreur : un état d’erreur, distinct du 410', async () => {
		const { findByTestId, queryByTestId } = render(DocumentPreview, {
			load: () => Promise.reject(new Error('réseau')),
			fileName: 'x.pdf',
		});
		expect((await findByTestId('document-preview-error')).textContent).toMatch(
			/pas pu être affiché/,
		);
		expect(queryByTestId('document-preview-gone')).toBeNull();
	});

	it('révoque l’URL blob: à la destruction (M-a9)', async () => {
		const { findByTestId, unmount } = render(DocumentPreview, {
			load: () => Promise.resolve(blob('image/png')),
			fileName: 'a.png',
		});
		await findByTestId('document-preview-image');
		expect(created).toEqual(['blob:doc-1']);
		expect(revoked).toEqual([]);
		unmount();
		expect(revoked).toEqual(['blob:doc-1']);
	});

	it('révoque l’URL précédente au changement de document (M-a9)', async () => {
		const { findByTestId, rerender } = render(DocumentPreview, {
			load: () => Promise.resolve(blob('image/png')),
			fileName: 'a.png',
		});
		await findByTestId('document-preview-image');
		await rerender({ load: () => Promise.resolve(blob('application/pdf')), fileName: 'b.pdf' });
		await findByTestId('document-preview-frame');
		expect(created).toEqual(['blob:doc-1', 'blob:doc-2']);
		await waitFor(() => expect(revoked).toEqual(['blob:doc-1']));
	});
});
