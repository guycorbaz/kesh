/**
 * Aide commune des specs du dossier d'import — Story 30-2a (#685, T-a7).
 *
 * ⚠️ **Ces specs portent la recette : elles ne doivent pas pouvoir se taire.** Aucun
 * `test.skip` : les deux répertoires se lisent dans l'environnement du runner, avec pour
 * défauts **ceux de la recette documentée** (`docs/testing.md`, § *Prérequis Playwright
 * local*, terminal 1) — si bien que la recette suivie à la lettre suffit, sans variable
 * côté runner. Un répertoire absent ou non inscriptible fait **échouer** la spec en le
 * nommant ; un backend qui lit un autre dossier (un worktree a les siens,
 * `docs/agents-orchestration.md` § 4) est détecté par la poignée de main.
 *
 * Fixtures **synthétiques** (§ 8 des consignes : aucune pièce réelle) : les deux PNG à QR
 * versionnés de `fixtures/`, un PNG minimal et un PDF minimal écrits ici en constantes.
 * Chaque parcours dépose **sa propre copie**, rendue unique par un suffixe ajouté APRÈS la
 * fin logique du fichier (après `IEND` d'un PNG, après `%%EOF` d'un PDF) — ce qu'ignorent
 * les lecteurs (test Rust `import_png_with_suffix_after_iend_still_decoded`). Sans cela, le
 * staging, unique par empreinte, refuserait la seconde copie en doublon.
 */
import { expect } from '@playwright/test';
import type { Page } from '@playwright/test';
import { constants, promises as fs } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { authedApiContext, disposeContextSafe } from './test-state';

/** Dossier d'import lu par le backend (défaut : celui de la recette). */
export const INBOX_DIR = process.env.KESH_INBOX_DIR ?? '/tmp/kesh-e2e/inbox';
/** Dossier des documents archivés (défaut : celui de la recette). */
export const DOCUMENTS_DIR = process.env.KESH_DOCUMENTS_DIR ?? '/tmp/kesh-e2e/documents';

const FIXTURES = path.join(path.dirname(fileURLToPath(import.meta.url)), '..', 'fixtures');

/** PNG 1×1 valide, sans QR (constante d'octets, aucun fichier binaire). */
export const PNG_SANS_QR: Buffer = Buffer.from(
	'iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mNkYAAAAAYAAjCB0C8AAAAASUVORK5CYII=',
	'base64',
);

/** PDF minimal d'une page de texte, sans QR, écrit à la main. */
export const PDF_SANS_QR: Buffer = Buffer.from(
	[
		'%PDF-1.4',
		'1 0 obj << /Type /Catalog /Pages 2 0 R >> endobj',
		'2 0 obj << /Type /Pages /Kids [3 0 R] /Count 1 >> endobj',
		'3 0 obj << /Type /Page /Parent 2 0 R /MediaBox [0 0 200 100] /Contents 4 0 R /Resources << /Font << /F1 5 0 R >> >> >> endobj',
		'4 0 obj << /Length 44 >> stream',
		'BT /F1 12 Tf 20 50 Td (Pas une facture) Tj ET',
		'endstream endobj',
		'5 0 obj << /Type /Font /Subtype /Type1 /BaseFont /Helvetica >> endobj',
		'trailer << /Root 1 0 R >>',
		'%%EOF',
		'',
	].join('\n'),
	'latin1',
);

/** Lit une fixture versionnée de `tests/e2e/fixtures/`. */
export async function readFixture(name: string): Promise<Buffer> {
	return fs.readFile(path.join(FIXTURES, name));
}

/** Jeton aléatoire court, pour les noms et les suffixes. */
export function token(): string {
	return `${Date.now()}-${Math.floor(Math.random() * 1e9)}`;
}

/**
 * Copie unique d'un document : un suffixe après sa fin logique, que les lecteurs
 * ignorent (commentaire PDF après `%%EOF`, octets après `IEND` d'un PNG).
 */
export function uniqueCopy(bytes: Buffer, kind: 'png' | 'pdf'): Buffer {
	const suffix = kind === 'pdf' ? `\n% kesh-e2e-${token()}\n` : `kesh-e2e-${token()}`;
	return Buffer.concat([bytes, Buffer.from(suffix, 'latin1')]);
}

/**
 * Échoue — sans jamais sauter — si l'un des deux dossiers manque ou n'est pas
 * inscriptible, en le nommant.
 */
export async function assertInboxDirsUsable(): Promise<void> {
	for (const [name, dir] of [
		['KESH_INBOX_DIR', INBOX_DIR],
		['KESH_DOCUMENTS_DIR', DOCUMENTS_DIR],
	] as const) {
		try {
			await fs.access(dir, constants.W_OK);
		} catch {
			throw new Error(
				`${name} = ${dir} : répertoire absent ou non inscriptible par le runner. ` +
					'Créez-le (recette de docs/testing.md, « Prérequis Playwright local ») ' +
					'ou passez la variable au runner avec la même valeur qu’au backend.',
			);
		}
	}
}

/** Dépose `bytes` sous `fileName` dans le dossier d'import ; rend le chemin. */
export async function deposit(fileName: string, bytes: Buffer): Promise<string> {
	const target = path.join(INBOX_DIR, fileName);
	await fs.writeFile(target, bytes);
	return target;
}

/** `true` si le fichier est encore dans le dossier d'import. */
export async function stillInInbox(fileName: string): Promise<boolean> {
	try {
		await fs.access(path.join(INBOX_DIR, fileName));
		return true;
	} catch {
		return false;
	}
}

/** Entrée `accepted[]` du rapport d'import. */
export interface AcceptedEntry {
	importedSupplierInvoiceId: number;
	fileName: string;
	qr: string;
	qrError?: string;
}

/** Lance un run d'import par l'API et rend son rapport. */
export async function importViaApi(
	page: Page,
): Promise<{ accepted: AcceptedEntry[]; failed: Array<{ fileName: string; errorCode: string }> }> {
	const ctx = await authedApiContext(page);
	try {
		const res = await ctx.post('/api/v1/inbox-import', { data: {} });
		expect(res.ok(), `inbox-import: ${res.status()}`).toBeTruthy();
		return await res.json();
	} finally {
		await disposeContextSafe(ctx);
	}
}

/** Retire un document de la liste par l'API (`discard`). */
export async function discardViaApi(page: Page, id: number): Promise<void> {
	const ctx = await authedApiContext(page);
	try {
		const res = await ctx.post(`/api/v1/imported-supplier-invoices/${id}/discard`, { data: {} });
		expect(res.status(), `discard ${id}`).toBe(204);
	} finally {
		await disposeContextSafe(ctx);
	}
}

/**
 * Poignée de main runner ↔ backend : un fichier sonde déposé ici doit revenir dans
 * `accepted[]` d'un import lancé là-bas. Sinon, les deux ne lisent pas le même dossier,
 * et la spec échoue en le disant au lieu de chercher la panne dans l'écran. La sonde
 * est ensuite retirée de la liste. Exige une session (`login(page)`).
 */
export async function inboxHandshake(page: Page): Promise<void> {
	const probe = `e2e-sonde-${token()}.png`;
	await deposit(probe, uniqueCopy(PNG_SANS_QR, 'png'));
	const report = await importViaApi(page);
	const entry = report.accepted.find((a) => a.fileName === probe);
	if (!entry) {
		throw new Error(
			`le runner et le backend n'utilisent pas le même KESH_INBOX_DIR (runner : ${INBOX_DIR}) — ` +
				`sonde ${probe} absente du rapport : ${JSON.stringify(report)}`,
		);
	}
	await discardViaApi(page, entry.importedSupplierInvoiceId);
}

/**
 * Story 30-2b — crée un fournisseur par l'API et rend son id. Partagé par
 * `inbox-import.spec.ts` et `supplier-documents.spec.ts` (DRY).
 */
export async function createSupplierViaApi(page: Page, name: string): Promise<number> {
	const ctx = await authedApiContext(page);
	try {
		const res = await ctx.post('/api/v1/contacts', {
			data: {
				contactType: 'Entreprise',
				name,
				isClient: false,
				isSupplier: true,
				address: 'Rue 2\n1000 Lausanne',
				defaultPaymentTerms: '30 jours net',
			},
		});
		expect(res.ok(), `createSupplier failed: ${res.status()}`).toBeTruthy();
		return (await res.json()).id as number;
	} finally {
		await disposeContextSafe(ctx);
	}
}

/**
 * Désigne le compte créanciers (2000) s'il manque et rend un compte de charge — sans quoi
 * aucune facture fournisseur ne se crée. Le seed `with-company` ne le désigne pas (Story
 * 15-5d, AC6 — contournement gardé ; il ne dépend pas de #521). Déplacé ici depuis
 * `inbox-import.spec.ts` par la Story 30-2b (DRY).
 */
export async function ensurePayableAndExpense(page: Page): Promise<number> {
	const ctx = await authedApiContext(page);
	try {
		const accounts = (await (await ctx.get('/api/v1/accounts')).json()) as Array<{
			id: number;
			number: string;
			accountType: string;
			active: boolean;
			postable: boolean;
		}>;
		const payable = accounts.find((a) => a.number === '2000');
		const expense = accounts.find((a) => a.accountType === 'Expense' && a.active && a.postable);
		expect(payable, 'compte 2000 attendu').toBeTruthy();
		expect(expense, 'un compte de charge attendu').toBeTruthy();
		const s = await (await ctx.get('/api/v1/company/invoice-settings')).json();
		if (s.defaultPayableAccountId == null) {
			const putRes = await ctx.put('/api/v1/company/invoice-settings', {
				data: { ...s, defaultPayableAccountId: payable!.id },
			});
			expect(putRes.ok(), `set payable failed: ${putRes.status()}`).toBeTruthy();
		}
		return expense!.id;
	} finally {
		await disposeContextSafe(ctx);
	}
}

/**
 * Story 30-2b — remplit la première ligne du formulaire de facture fournisseur
 * (`SupplierInvoiceForm`) : description, prix unitaire, compte de charge (quantité 1, TVA 0).
 */
export async function fillFirstInvoiceLine(
	page: Page,
	description: string,
	unitPrice: string,
	expenseAccountId: number,
): Promise<void> {
	const line = page.getByTestId('supplier-invoice-line').first();
	await line.getByTestId('supplier-invoice-line-desc').fill(description);
	await line.getByTestId('supplier-invoice-line-unit-price').fill(unitPrice);
	await line.getByTestId('supplier-invoice-line-account').selectOption(String(expenseAccountId));
}
