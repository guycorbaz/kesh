/**
 * Fabriques des tests Vitest de l'écran des postes ouverts (Story 15-1c-i).
 *
 * ⚠️ Le nom porte `.test.` : le fichier est hors collecte des gardes i18n (aucun
 * texte d'écran ici) et hors du motif `*.test.ts` de Vitest (aucun test ici).
 */
import type {
	LetteringDetail,
	LetteringDetailLine,
	OpenItem,
	OpenItemsResponse,
	Proposal,
	ProposalLine,
	ProposalsResponse,
} from './open-items.types';
import type { AccountResponse } from '$lib/features/accounts/accounts.types';

export function item(over: Partial<OpenItem> = {}): OpenItem {
	return {
		lineId: 1,
		entryId: 10,
		entryNumber: 12,
		fiscalYearName: 'Exercice 2026',
		date: '2026-03-04',
		journal: 'OD',
		description: 'Avance au fournisseur',
		debit: '100.0000',
		credit: '0.0000',
		document: null,
		letteringCode: null,
		letteringOrigin: null,
		letteredOn: null,
		reason: 'unlettered',
		documentState: null,
		amountDue: null,
		manuallyLetterable: true,
		inOpenPeriod: true,
		...over,
	};
}

export function view(over: Partial<OpenItemsResponse> = {}): OpenItemsResponse {
	const items = over.items ?? [item()];
	return {
		accountId: 7,
		accountNumber: '1091',
		asOf: '2026-03-31',
		balance: '100.0000',
		openTotal: '100.0000',
		total: items.length,
		offset: 0,
		limit: 50,
		items,
		...over,
	};
}

export function proposalLine(over: Partial<ProposalLine> = {}): ProposalLine {
	return {
		lineId: 1,
		entryId: 10,
		entryNumber: 12,
		fiscalYearName: 'Exercice 2026',
		date: '2026-03-04',
		journal: 'OD',
		description: 'Avance',
		document: null,
		inOpenPeriod: true,
		...over,
	};
}

export function proposal(over: Partial<Proposal> = {}): Proposal {
	return {
		amount: '100.0000',
		daysApart: 3,
		reversalPair: false,
		debit: proposalLine({ lineId: 1, entryId: 10 }),
		credit: proposalLine({ lineId: 2, entryId: 11, entryNumber: 13, description: 'Remboursement' }),
		...over,
	};
}

export function proposals(items: Proposal[] = [proposal()], over: Partial<ProposalsResponse> = {}) {
	return { accountId: 7, candidateCount: 2, total: items.length, limit: 100, items, ...over };
}

export function groupLine(over: Partial<LetteringDetailLine> = {}): LetteringDetailLine {
	return {
		id: 1,
		entryId: 10,
		entryNumber: 12,
		fiscalYearId: 1,
		fiscalYearName: 'Exercice 2026',
		date: '2026-03-04',
		debit: '100.0000',
		credit: '0.0000',
		journal: 'OD',
		description: 'Avance',
		document: null,
		ownedByDocument: false,
		inOpenPeriod: true,
		...over,
	};
}

export function group(over: Partial<LetteringDetail> = {}): LetteringDetail {
	return {
		key: 27,
		code: 'AA',
		origin: 'manual',
		accountId: 7,
		accountNumber: '1091',
		accountName: 'Compte de passage',
		manualDissolutionBlockedBy: null,
		lines: [
			groupLine({ id: 1 }),
			groupLine({ id: 2, entryId: 11, entryNumber: 13, debit: '0.0000', credit: '100.0000' }),
		],
		...over,
	};
}

export function account(over: Partial<AccountResponse> = {}): AccountResponse {
	return {
		id: 7,
		companyId: 1,
		number: '1091',
		name: 'Compte de passage',
		accountType: 'Asset',
		parentId: null,
		active: true,
		role: null,
		postable: true,
		letterable: true,
		version: 1,
		createdAt: '2026-01-01T00:00:00',
		updatedAt: '2026-01-01T00:00:00',
		...over,
	};
}

/** Un refus du serveur, sous la forme que lève `apiClient` (`ApiError`). */
export function refusal(status: number, code: string, message = `message de ${code}`) {
	return { status, code, message };
}
