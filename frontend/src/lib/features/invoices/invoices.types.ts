/**
 * Types TS miroir des DTOs API pour les factures brouillon (Story 5.1).
 *
 * Shape identique au backend `crates/kesh-api/src/routes/invoices.rs`
 * (serde `rename_all = "camelCase"`). Montants (`quantity`, `unitPrice`,
 * `vatRate`, `lineTotal`, `totalAmount`) transportés en **string décimale**
 * via `rust_decimal::serde-str`. Ne JAMAIS convertir en `number`.
 */

export type InvoiceSortBy = 'Date' | 'DueDate' | 'TotalAmount' | 'ContactName' | 'CreatedAt';
export type SortDirection = 'Asc' | 'Desc';
export type InvoiceStatus = 'draft' | 'validated' | 'cancelled';

export interface InvoiceLineResponse {
	id: number;
	invoiceId: number;
	position: number;
	description: string;
	quantity: string;
	unitPrice: string;
	vatRate: string;
	lineTotal: string;
	createdAt: string;
}

export interface InvoiceResponse {
	id: number;
	companyId: number;
	contactId: number;
	invoiceNumber: string | null;
	status: InvoiceStatus;
	date: string;
	dueDate: string | null;
	paymentTerms: string | null;
	totalAmount: string;
	/**
	 * TTC canonique (#246, Story 21-2a) — le montant réellement dû (QR, PDF,
	 * e-mail). `totalAmount` reste le HT comptable. String décimale, jamais
	 * Number (convention en tête de fichier).
	 */
	totalTtc: string;
	journalEntryId: number | null;
	paidAt: string | null;
	/** Dernier envoi par e-mail (Story 20-3b2). `null` = jamais envoyée. */
	emailedAt: string | null;
	/** Destinataire du dernier envoi (snapshot `contacts.email` à l'envoi). */
	emailedTo: string | null;
	/** Projet analytique document-level (Epic 19). `null` = non taguée. */
	projectId: number | null;
	/** Story 21-6a (D10) — rappels suspendus. `null` = rappels actifs. */
	dunningPausedAt: string | null;
	dunningPausedNote: string | null;
	/** Calculé backend (P6 review pass 2). Source unique de vérité pour le badge « en retard ». */
	isOverdue: boolean;
	version: number;
	createdAt: string;
	updatedAt: string;
	lines: InvoiceLineResponse[];
}

// Story 5.2 — Configuration facturation
export type JournalCode = 'Achats' | 'Ventes' | 'Banque' | 'Caisse' | 'OD';

export interface InvoiceSettingsResponse {
	companyId: number;
	invoiceNumberFormat: string;
	defaultReceivableAccountId: number | null;
	defaultRevenueAccountId: number | null;
	defaultVatPayableAccountId: number | null;
	defaultVatRecoverableAccountId: number | null;
	defaultVatDecompteAccountId: number | null;
	defaultSalesJournal: JournalCode;
	journalEntryDescriptionTemplate: string;
	version: number;
}

export interface UpdateInvoiceSettingsRequest {
	invoiceNumberFormat: string;
	defaultReceivableAccountId: number | null;
	defaultRevenueAccountId: number | null;
	defaultVatPayableAccountId: number | null;
	defaultVatRecoverableAccountId: number | null;
	defaultVatDecompteAccountId: number | null;
	defaultSalesJournal: JournalCode;
	journalEntryDescriptionTemplate: string;
	version: number;
}

export interface InvoiceListItemResponse {
	id: number;
	companyId: number;
	contactId: number;
	contactName: string;
	invoiceNumber: string | null;
	status: InvoiceStatus;
	date: string;
	dueDate: string | null;
	paymentTerms: string | null;
	totalAmount: string;
	/** TTC canonique (#246). String décimale, jamais Number. */
	totalTtc: string;
	paidAt: string | null;
	/** Story 21-6a (D10) — rappels suspendus. `null` = rappels actifs. */
	dunningPausedAt: string | null;
	dunningPausedNote: string | null;
	version: number;
	createdAt: string;
	updatedAt: string;
}

// Story 5.4 — Échéancier
export type PaymentStatusFilter = 'all' | 'paid' | 'unpaid' | 'overdue';

/**
 * Story 21-6a (D10) — filtre « rappels suspendus » de la liste des factures.
 * `all` est le défaut et ne filtre rien.
 */
export type PausedFilter = 'all' | 'paused' | 'not-paused';

export interface DueDateItem extends InvoiceListItemResponse {
	isOverdue: boolean;
}

export interface DueDatesSummary {
	unpaidCount: number;
	unpaidTotal: string;
	overdueCount: number;
	overdueTotal: string;
}

export interface DueDatesResponse {
	items: DueDateItem[];
	total: number;
	offset: number;
	limit: number;
	summary: DueDatesSummary;
}

export interface DueDatesQuery {
	search?: string;
	contactId?: number;
	dateFrom?: string;
	dateTo?: string;
	dueBefore?: string;
	paymentStatus?: PaymentStatusFilter;
	sortBy?: InvoiceSortBy;
	sortDirection?: SortDirection;
	limit?: number;
	offset?: number;
}

export interface MarkPaidRequest {
	paidAt?: string;
	version: number;
}

export interface UnmarkPaidRequest {
	version: number;
}

export interface CreateInvoiceLineRequest {
	description: string;
	quantity: string;
	unitPrice: string;
	vatRate: string;
}

export interface CreateInvoiceRequest {
	contactId: number;
	date: string;
	dueDate?: string | null;
	paymentTerms?: string | null;
	/** Projet analytique optionnel (Epic 19, Story 19-4). */
	projectId?: number | null;
	lines: CreateInvoiceLineRequest[];
}

export interface UpdateInvoiceRequest extends CreateInvoiceRequest {
	version: number;
}

export interface ListInvoicesQuery {
	search?: string;
	status?: InvoiceStatus;
	contactId?: number;
	dateFrom?: string;
	dateTo?: string;
	/** Story 21-6a (D10). `all` (défaut) n'est pas sérialisé. */
	paused?: PausedFilter;
	sortBy?: InvoiceSortBy;
	sortDirection?: SortDirection;
	limit?: number;
	offset?: number;
}

export interface ListResponse<T> {
	items: T[];
	total: number;
	limit: number;
	offset: number;
}

// Story 20-3b2 — Envoi de facture par e-mail (#224)

/** Langue de correspondance résolue par le backend (contact sinon instance). */
export type EmailLanguage = 'FR' | 'DE' | 'IT' | 'EN';

/** Réponse de `GET /api/v1/invoices/{id}/email-preview`. */
export interface EmailPreviewResponse {
	/** Destinataire verrouillé (`contacts.email`). `null` = contact sans e-mail → envoi désactivé. */
	to: string | null;
	language: EmailLanguage;
	subject: string;
	body: string;
}

/** Payload de `POST /api/v1/invoices/{id}/send-email`. Pas de champ `to` (destinataire verrouillé serveur). */
export interface SendInvoiceEmailRequest {
	subject: string;
	body: string;
}
