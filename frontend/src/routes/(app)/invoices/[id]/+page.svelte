<script lang="ts">
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import * as Dialog from '$lib/components/ui/dialog';
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import { Pencil, Trash2, ArrowLeft, CheckCircle2, BookOpen, Printer, Mail } from '@lucide/svelte';

	import {
		getInvoice,
		deleteInvoice,
		validateInvoice,
		settleInvoice,
			getInvoiceEmailPreview,
		sendInvoiceEmail,
		getInvoiceSettings,
	} from '$lib/features/invoices/invoices.api';
	import { fetchAccounts } from '$lib/features/accounts/accounts.api';
	import type { AccountResponse } from '$lib/features/accounts/accounts.types';
	import type { InvoiceSettingsResponse } from '$lib/features/invoices/invoices.types';
	import { invoiceRevenueAccountLabel } from '$lib/features/accounts/account-label';
	import PaymentStatusBadge from '$lib/features/invoices/PaymentStatusBadge.svelte';
	import SettleInvoiceDialog, {
		type SettlementPayload,
	} from '$lib/features/invoices/SettleInvoiceDialog.svelte';
	import {
		listBankAccounts,
		type BankAccountSummary,
	} from '$lib/features/bank-accounts/bank-accounts.api';
	import SendEmailDialog from '$lib/features/invoices/SendEmailDialog.svelte';
	import DunningPausedBadge from '$lib/features/invoices/DunningPausedBadge.svelte';
	import ReminderHistory from '$lib/features/reminders/ReminderHistory.svelte';
	import DunningPauseDialog from '$lib/features/reminders/DunningPauseDialog.svelte';
	import {
		listReminderHistory,
		pauseDunning,
		resumeDunning,
	} from '$lib/features/reminders/reminders.api';
	import type { ReminderResponse } from '$lib/features/reminders/reminders.types';
	import * as Tooltip from '$lib/components/ui/tooltip';
	import { featureFlags } from '$lib/shared/utils/feature-flags.svelte';
	import type {
		EmailPreviewResponse,
		InvoiceResponse,
	} from '$lib/features/invoices/invoices.types';
	import { formatInvoiceTotal } from '$lib/features/invoices/invoice-helpers';
	import { apiClient, isApiError } from '$lib/shared/utils/api-client';
	import {
		notifyError,
		notifyMissingFiscalYearOrFallback,
		notifySuccess,
		notifyWarning
	} from '$lib/shared/utils/notify';
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import { authState } from '$lib/app/stores/auth.svelte';
	import { createCreditNote } from '$lib/features/credit-notes/credit-notes.api';
	import { listProjects } from '$lib/features/projects/projects.api';
	import type { ProjectResponse } from '$lib/features/projects/projects.types';

	let creditNoteOpen = $state(false);
	let creditNoteSubmitting = $state(false);
	let creditNoteError = $state('');

	let canManage = $derived(
		authState.currentUser?.role === 'Admin' || authState.currentUser?.role === 'Comptable',
	);
	// #219 : la suppression définitive d'une facture validée est réservée Admin.
	let isAdmin = $derived(authState.currentUser?.role === 'Admin');

	async function confirmCreateCreditNote() {
		if (!invoice) return;
		creditNoteSubmitting = true;
		creditNoteError = '';
		try {
			const today = new Date().toISOString().slice(0, 10);
			const cn = await createCreditNote({ invoiceId: invoice.id, date: today });
			creditNoteOpen = false;
			notifySuccess(i18nMsg('credit-notes-created', 'Avoir créé'));
			await goto(`/credit-notes/${cn.id}`);
		} catch (err) {
			if (isApiError(err)) creditNoteError = err.message;
			else creditNoteError = i18nMsg('credit-notes-create-error', "Échec de la création de l'avoir");
		} finally {
			creditNoteSubmitting = false;
		}
	}

	let invoice = $state<InvoiceResponse | null>(null);
	let loading = $state(true);
	let errorMsg = $state('');
	let deleteOpen = $state(false);
	let deleteSubmitting = $state(false);
	let deleteError = $state('');
	// #219 : saisie de confirmation forte (retaper le n° de facture) pour la
	// suppression d'une facture validée.
	let deleteConfirmText = $state('');
	let validateOpen = $state(false);
	let validateSubmitting = $state(false);
	let validateError = $state('');

	let id = $derived(parseInt(page.params.id ?? '', 10));

	// Story 21-6c — historique des rappels + suspension.
	let reminders = $state<ReminderResponse[]>([]);
	let pauseOpen = $state(false);
	let pauseSubmitting = $state(false);
	let pauseError = $state('');

	/**
	 * Recharge l'historique des rappels. Échec toléré (historique secondaire) :
	 * pas de toast bloquant, la section reste vide.
	 */
	async function loadReminderHistory() {
		try {
			reminders = await listReminderHistory(id);
		} catch {
			// Historique secondaire — dégradation silencieuse.
		}
	}

	onMount(async () => {
		if (!Number.isFinite(id) || id <= 0) {
			errorMsg = 'Identifiant de facture invalide';
			loading = false;
			return;
		}
		try {
			invoice = await getInvoice(id);
		} catch (err) {
			if (isApiError(err)) errorMsg = err.message;
		} finally {
			loading = false;
		}
		// Story 21-6c : historique des rappels (après getInvoice, échec toléré).
		if (invoice) await loadReminderHistory();
		// Story 19-4 : référentiel projets (archivés inclus — un tag historique
		// doit rester lisible). Échec toléré : libellé fallback `#id`.
		try {
			projects = await listProjects(true);
		} catch {
			// Fallback #id via projectLabel.
		}
		// Story 16-1b (T5-bis/AC6-bis) : référentiel comptes + réglages, pour
		// afficher le compte de produit de chaque ligne.
		//
		// `fetchAccounts(true)` — archivés INCLUS, même motif que D11 côté
		// formulaire : une facture validée peut parfaitement référencer un compte
		// archivé depuis, et son libellé doit rester lisible. Sans le flag, la
		// colonne afficherait `#3900` au lieu du nom du compte.
		//
		// Échecs tolérés (l'écran de détail doit rester consultable) : la colonne
		// retombe alors sur `#id`, ou sur un tiret si le défaut est inconnu.
		try {
			accounts = await fetchAccounts(true);
		} catch {
			// Fallback `#id` via `revenueAccountCell`.
		}
		try {
			invoiceSettings = await getInvoiceSettings();
		} catch {
			// Fallback tiret pour les lignes sans compte.
		}
	});

	let accounts = $state<AccountResponse[]>([]);
	let invoiceSettings = $state<InvoiceSettingsResponse | null>(null);

	/**
	 * Libellé du compte de produit d'une ligne (Story 16-1b, AC6-bis).
	 *
	 * # Pourquoi cette colonne existe
	 *
	 * L'édition n'est offerte qu'aux brouillons. Dès qu'une facture est validée —
	 * c'est-à-dire dès l'instant où la ventilation produit son effet comptable —
	 * l'utilisateur n'avait plus AUCUN moyen de voir sur quels comptes ses lignes
	 * avaient été ventilées, sinon d'ouvrir l'écriture et de la lire à l'envers.
	 * La donnée était write-once, et c'est le geste de contrôle n°1 d'une
	 * fiduciaire qui reprend un dossier.
	 *
	 * Règle déléguée à `account-label` — l'écran d'AVOIR utilise la fonction
	 * jumelle, dont la seule différence est le traitement de `null`.
	 */
	function revenueAccountCell(revenueAccountId: number | null): string {
		return invoiceRevenueAccountLabel(
			accounts,
			revenueAccountId,
			invoiceSettings?.defaultRevenueAccountId ?? null,
			invoice?.status === 'draft'
		);
	}

	let projects = $state<ProjectResponse[]>([]);
	const projectLabel = $derived.by(() => {
		const pid = invoice?.projectId;
		if (!pid) return null;
		const p = projects.find((x) => x.id === pid);
		return p ? `${p.code} — ${p.name}` : `#${pid}`;
	});

	async function confirmDelete() {
		if (!invoice) return;
		deleteSubmitting = true;
		deleteError = '';
		try {
			await deleteInvoice(invoice.id);
			notifySuccess('Facture supprimée');
			goto('/invoices');
		} catch (err) {
			// Cohérence avec la modale de conflit (InvoiceForm) : erreur visible
			// dans la modale en plus du toast.
			if (isApiError(err)) {
				deleteError = err.message;
				notifyError(err.message);
			} else {
				deleteError = 'Erreur lors de la suppression';
				notifyError(deleteError);
			}
		} finally {
			deleteSubmitting = false;
		}
	}

	async function confirmValidate() {
		if (!invoice) return;
		validateSubmitting = true;
		validateError = '';
		try {
			const updated = await validateInvoice(invoice.id);
			invoice = updated;
			// Review edge #13 : fallback si invoiceNumber null (ne devrait pas arriver mais défensif).
			notifySuccess(
				updated.invoiceNumber ? `Facture validée — ${updated.invoiceNumber}` : 'Facture validée',
			);
			validateOpen = false;
		} catch (err) {
			if (isApiError(err)) {
				validateError = err.message;
				// Story 3.7 AC #22 — fallback toast actionnable pour FISCAL_YEAR_INVALID.
				if (notifyMissingFiscalYearOrFallback(err)) {
					validateOpen = false;
					return;
				}
				if (err.code === 'CONFIGURATION_REQUIRED') {
					if (authState.currentUser?.role === 'Admin') {
						validateError = `${err.message} — Configurez les comptes par défaut dans Paramètres > Facturation.`;
					} else {
						validateError =
							'Demandez à votre administrateur de configurer les comptes par défaut de facturation.';
					}
				}
				notifyError(validateError);
				// Si conflit d'état (409) : la facture a peut-être changé — reload.
				if (err.code === 'ILLEGAL_STATE_TRANSITION' || err.code === 'OPTIMISTIC_LOCK_CONFLICT') {
					try {
						invoice = await getInvoice(invoice.id);
					} catch {
						// laisser l'erreur affichée
					}
					validateOpen = false;
				}
				// Review P13 : fermer le dialog sur erreurs non-retryables
				// (l'utilisateur doit corriger la config ou l'exercice avant de réessayer).
				if (
					err.code === 'FISCAL_YEAR_INVALID' ||
					err.code === 'CONFIGURATION_REQUIRED'
				) {
					validateOpen = false;
				}
			} else {
				validateError = 'Erreur lors de la validation';
				notifyError(validateError);
			}
		} finally {
			validateSubmitting = false;
		}
	}

	// Story 5.4 — mark/unmark paid
	let markOpen = $state(false);
	let markSubmitting = $state(false);
	let markError = $state('');
	// ⛔ Story 24-3 (#372) : plus de « dé-marquer ». Annuler un règlement demande
	// une CONTRE-PASSATION, pas un retrait de drapeau — issue #414.
	let settleAccounts = $state<AccountResponse[]>([]);
	let settleBankAccounts = $state<BankAccountSummary[]>([]);
	$effect(() => {
		void fetchAccounts(false)
			.then((a) => (settleAccounts = a))
			.catch(() => (settleAccounts = []));
		void listBankAccounts(false)
			.then((b) => (settleBankAccounts = b))
			.catch(() => (settleBankAccounts = []));
	});

	function paymentStatus(inv: InvoiceResponse): 'paid' | 'unpaid' | 'overdue' | 'partial' {
		if (inv.paidAt) return 'paid';
		// Story 24-2 (#371) — « partiellement payée » : un règlement est entré
		// sans éteindre la créance.
		//
		// ⚠️ Le test porte sur `amountSettled`, pas sur `amountDue` : `null` veut
		// dire « non calculé », et il ne faut surtout pas le lire comme un zéro.
		// La facture reste par ailleurs relançable — c'est voulu, il reste dû.
		if (inv.amountSettled !== null && Number(inv.amountSettled) > 0) return 'partial';
		// P6 (review pass 2) : `isOverdue` est calculé backend → single source
		// of truth pour « aujourd'hui » (évite la désync TZ client/serveur).
		// Pass 2 G2 D : strict `=== true` symétrique avec la page échéancier
		// (defensive contre `undefined` en rollout échelonné).
		return inv.isOverdue === true ? 'overdue' : 'unpaid';
	}

	async function handleSettleConfirm(payload: SettlementPayload) {
		if (!invoice) return;
		markSubmitting = true;
		markError = '';
		try {
			const res = await settleInvoice(invoice.id, payload);
			invoice = res.invoice;
			notifySuccess(
				res.fullySettled
					? i18nMsg('invoice-settle-success-full', 'Règlement enregistré — facture soldée')
					: i18nMsg('invoice-settle-success-partial', 'Règlement partiel enregistré'),
			);
			markOpen = false;
		} catch (err) {
			if (isApiError(err)) {
				markError = err.message;
				notifyError(err.message);
			} else {
				markError = i18nMsg('common-error', 'Erreur inattendue');
			}
		} finally {
			markSubmitting = false;
		}
	}

	// Story 21-6c — suspension / reprise des rappels (#231, D-c1).
	//
	// ⚠️ Piège n°1 : pause/resume renvoient un `DunningPauseResponse` (PAS la
	// facture entière) et incrémentent `version`. On ré-applique immédiatement
	// `{ version, dunningPausedAt, dunningPausedNote }` à l'état `invoice` local,
	// sinon la prochaine action (mark-paid, unmark, validate, delete) enverra une
	// version périmée → 409. Fix structurel : la ré-application se fait dans le
	// même handler, juste après le retour de l'API, jamais ailleurs.

	/**
	 * Traite un code d'erreur toggle (toast + refetch/navigation selon le cas).
	 * Retourne `true` si le dialog appelant doit se fermer (état changé ou
	 * navigation), `false` si l'erreur est transitoire — le dialog reste alors
	 * ouvert pour réessayer sans retaper le motif (patron `handleMarkConfirm`).
	 */
	async function handleToggleError(err: unknown): Promise<boolean> {
		if (!invoice) return true;
		if (!isApiError(err)) {
			notifyError(i18nMsg('common-error', 'Erreur inattendue'));
			// Erreur inattendue = transitoire du point de vue de l'état : garder ouvert.
			return false;
		}
		notifyError(
			err.code === 'INVOICE_NOT_PAUSED'
				? i18nMsg('reminders-error-not-paused', "Cette facture n'est plus suspendue.")
				: err.message,
		);
		// Facture supprimée entre-temps (autre onglet/utilisateur) : fiche fantôme,
		// tout re-clic rejouerait le 404 → retour liste (patron `openSendEmail` :396,
		// review P1 ECH1).
		if (err.code === 'NOT_FOUND') {
			await goto('/invoices');
			return true;
		}
		// 409 (verrou optimiste) ou 422 (reprise d'une non-suspendue, race UI) :
		// l'état local est périmé → refetch (filet du piège n°1).
		if (err.code === 'OPTIMISTIC_LOCK_CONFLICT' || err.code === 'INVOICE_NOT_PAUSED') {
			try {
				invoice = await getInvoice(invoice.id);
			} catch {
				// laisser l'erreur affichée
			}
			await loadReminderHistory();
			return true;
		}
		// Erreur métier non-conflictuelle (VALIDATION_ERROR, transitoire réseau) :
		// l'état n'a pas changé → garder le dialog ouvert pour corriger/réessayer.
		return false;
	}

	async function confirmPause(note: string | null) {
		// Anti-double-submit : garde de ré-entrance en plus du bouton `disabled`.
		if (!invoice || pauseSubmitting) return;
		pauseSubmitting = true;
		pauseError = '';
		try {
			const res = await pauseDunning(invoice.id, { version: invoice.version, note });
			// Piège n°1 — ré-appliquer version + état de suspension.
			invoice = {
				...invoice,
				version: res.version,
				dunningPausedAt: res.dunningPausedAt,
				dunningPausedNote: res.dunningPausedNote,
			};
			pauseOpen = false;
			notifySuccess(i18nMsg('reminders-pause-success', 'Rappels suspendus'));
			await loadReminderHistory();
		} catch (err) {
			// Review P1 ECH2 : ne fermer QUE si l'état a changé/navigué. Sur une
			// erreur transitoire, garder la modale ouverte (le motif tapé est
			// préservé — le dialog ne le réinitialise qu'à la ré-ouverture) et
			// afficher l'erreur inline (patron `handleMarkConfirm`).
			const shouldClose = await handleToggleError(err);
			if (shouldClose) {
				pauseOpen = false;
			} else {
				pauseError = isApiError(err) ? err.message : i18nMsg('common-error', 'Erreur inattendue');
			}
		} finally {
			pauseSubmitting = false;
		}
	}

	async function confirmResume() {
		// La reprise agit directement (pas de modale, D-c1). Garde de ré-entrance.
		if (!invoice || pauseSubmitting) return;
		pauseSubmitting = true;
		try {
			const res = await resumeDunning(invoice.id, { version: invoice.version });
			// Piège n°1 — la reprise nulle `dunningPausedAt` ET `dunningPausedNote`.
			invoice = {
				...invoice,
				version: res.version,
				dunningPausedAt: res.dunningPausedAt,
				dunningPausedNote: res.dunningPausedNote,
			};
			notifySuccess(i18nMsg('reminders-resume-success', 'Rappels repris'));
			await loadReminderHistory();
		} catch (err) {
			await handleToggleError(err);
		} finally {
			pauseSubmitting = false;
		}
	}

	// Story 20-3b2 — envoi de la facture par e-mail (#224).
	let sendEmailOpen = $state(false);
	let sendEmailSubmitting = $state(false);
	let sendEmailError = $state('');
	let sendEmailPreview = $state<EmailPreviewResponse | null>(null);
	let previewLoading = $state(false);

	/** Clic bouton : fetch la preview PUIS ouvre la modale pré-remplie. */
	async function openSendEmail() {
		if (!invoice || previewLoading) return;
		previewLoading = true;
		try {
			sendEmailPreview = await getInvoiceEmailPreview(invoice.id);
			sendEmailError = '';
			sendEmailOpen = true;
		} catch (err) {
			// CONTACT_ARCHIVED (400) et toute autre erreur : toast, pas de modale.
			if (isApiError(err)) {
				notifyError(err.message);
				// Flag client périmé (review P1 BH) : même resync que le send.
				if (err.code === 'SMTP_NOT_CONFIGURED') featureFlags.setSmtpConfigured(false);
				// Facture supprimée pendant la consultation (review P3 BH) :
				// symétrique du send — fiche fantôme, retour liste.
				if (err.code === 'NOT_FOUND') await goto('/invoices');
			} else {
				notifyError(i18nMsg('common-error', 'Erreur inattendue'));
			}
		} finally {
			previewLoading = false;
		}
	}

	async function handleSendEmailConfirm(subject: string, body: string) {
		// Ré-entrance (review P1 ECH) : le POST déclenche un envoi SMTP réel,
		// non idempotent — ne pas compter que sur le disabled du bouton.
		if (!invoice || sendEmailSubmitting) return;
		sendEmailSubmitting = true;
		sendEmailError = '';
		try {
			invoice = await sendInvoiceEmail(invoice.id, { subject, body });
			notifySuccess(i18nMsg('invoice-send-email-success', 'Facture envoyée par e-mail'));
			sendEmailOpen = false;
		} catch (err) {
			if (!isApiError(err)) {
				sendEmailError = i18nMsg('common-error', 'Erreur inattendue');
				return;
			}
			switch (err.code) {
				// Contenu vide / échec SMTP (facture NON marquée) : inline, la
				// modale reste ouverte pour corriger ou réessayer.
				case 'INVOICE_EMAIL_EMPTY_CONTENT':
				case 'SMTP_SEND_FAILED':
					sendEmailError = err.message;
					break;
				// L'état du contact a changé sous nos pieds : toast + fermer.
				case 'CONTACT_EMAIL_MISSING':
				case 'CONTACT_ARCHIVED':
					notifyError(err.message);
					sendEmailOpen = false;
					break;
				// Flag client périmé : resynchroniser (le bouton se grise).
				case 'SMTP_NOT_CONFIGURED':
					notifyError(err.message);
					featureFlags.setSmtpConfigured(false);
					sendEmailOpen = false;
					break;
				case 'RATE_LIMITED':
					notifyError(err.message);
					break;
				// L'e-mail est PARTI mais la facture a été supprimée pendant
				// l'envoi (20-3b1 review ECH-1) : message anti-renvoi du
				// backend, la fiche n'existe plus → retour liste.
				case 'EMAIL_SENT_INVOICE_GONE':
					notifyWarning(err.message);
					sendEmailOpen = false;
					await goto('/invoices');
					break;
				// Facture supprimée AVANT l'envoi (entre preview et confirm) :
				// re-cliquer rejouerait le même 404 — fermer + retour liste
				// (review P1 ECH, symétrique du 409 ci-dessus).
				case 'NOT_FOUND':
					notifyError(err.message);
					sendEmailOpen = false;
					await goto('/invoices');
					break;
				// Erreurs PDF héritées (INVOICE_NOT_VALIDATED, etc.) : inline.
				default:
					sendEmailError = err.message;
			}
		} finally {
			sendEmailSubmitting = false;
		}
	}

	let pdfDownloading = $state(false);

	// D2 (review pass 1 G2 D) : whitelist explicite des codes d'erreur PDF
	// — empêche la construction dynamique de clés FTL depuis err.code
	// (potentiel mismatch silencieux si le backend renvoie un nouveau code).
	const PDF_ERROR_KEYS: Record<string, string> = {
		INVOICE_NOT_VALIDATED: 'invoice-pdf-error-invoice-not-validated',
		INVOICE_NOT_PDF_READY: 'invoice-pdf-error-invoice-not-pdf-ready',
		INVOICE_TOO_MANY_LINES_FOR_PDF: 'error-invoice-too-many-lines-for-pdf',
		// Story 16-3a (#151) — sans cette entrée, le message retombe sur le
		// générique et l'utilisateur ne sait pas QUOI raccourcir.
		INVOICE_PDF_HEADER_OVERFLOW: 'error-invoice-pdf-header-overflow',
		PDF_GENERATION_FAILED: 'invoice-pdf-error-pdf-generation-failed',
		NOT_FOUND: 'invoice-pdf-error-not-found',
	};

	async function downloadPdf() {
		if (!invoice) return;
		pdfDownloading = true;
		try {
			const res = await apiClient.getBlob(`/api/v1/invoices/${invoice.id}/pdf`);
			const blob = await res.blob();
			if (blob.size === 0) {
				notifyError(i18nMsg('invoice-pdf-error-empty', 'Le PDF reçu est vide.'));
				return;
			}
			const url = URL.createObjectURL(blob);
			// D2 (review pass 1 G2 D) : pattern <a download> hidden — aligné
			// sur le download CSV échéancier, évite popup-blocker (Firefox
			// strict / Safari mobile) et garantit semantics de téléchargement.
			const filename = `facture-${invoice.invoiceNumber ?? invoice.id}.pdf`;
			const a = document.createElement('a');
			a.href = url;
			a.download = filename;
			a.style.display = 'none';
			document.body.appendChild(a);
			a.click();
			document.body.removeChild(a);
			// Revoke différé pour laisser le navigateur récupérer le blob.
			setTimeout(() => URL.revokeObjectURL(url), 5_000);
		} catch (err) {
			if (isApiError(err)) {
				// Pass 2 : remappage vers les clés FTL réellement présentes —
		// `INVOICE_TOO_MANY_LINES_FOR_PDF` utilise la clé legacy
		// `error-invoice-too-many-lines-for-pdf` (existante FR/DE/IT/EN
		// avec les arguments {count}/{max}).
		const key = PDF_ERROR_KEYS[err.code] ?? 'invoice-pdf-error-generic';
				notifyError(i18nMsg(key, err.message));
			} else {
				notifyError(i18nMsg('invoice-pdf-error-generic', 'Erreur lors du téléchargement du PDF'));
			}
		} finally {
			pdfDownloading = false;
		}
	}

	// ⚠️ **Le même défaut que la liste, sans le doc-comment qui l'y désignait** : un grep du
	// commentaire copié ne l'aurait pas trouvé — c'est la garde qui le relève. Mêmes clés que
	// la liste : un statut ne doit pas se dire autrement selon l'écran qui l'affiche.
	function statusLabel(s: string): string {
		if (s === 'draft') return i18nMsg('invoice-status-draft', 'Brouillon');
		if (s === 'validated') return i18nMsg('invoice-status-validated', 'Validée');
		// ⚠️ **Le `else` implicite affirmait « Annulée » pour TOUTE valeur inattendue** — un
		// statut ajouté au backend et pas encore connu de ce bundle s'affichait donc comme
		// annulé sur une pièce comptable : une affirmation fausse, et silencieuse. Le repli sur
		// la valeur brute suit le patron des trois fonctions sœurs. (Passe 1 de revue.)
		if (s === 'cancelled') return i18nMsg('invoice-status-cancelled', 'Annulée');
		return s;
	}
</script>

<svelte:head>
	<title>Facture — Kesh</title>
</svelte:head>

<div class="mb-6 flex items-center justify-between">
	<Button variant="ghost" onclick={() => goto('/invoices')}>
		<ArrowLeft class="h-4 w-4" aria-hidden="true" />
		Retour
	</Button>
	{#if invoice?.status === 'draft'}
		<div class="flex gap-2">
			<Button onclick={() => (validateOpen = true)}>
				<CheckCircle2 class="h-4 w-4" aria-hidden="true" />
				Valider
			</Button>
			<Button variant="outline" onclick={() => goto(`/invoices/${invoice!.id}/edit`)}>
				<Pencil class="h-4 w-4" aria-hidden="true" />
				Modifier
			</Button>
			<Button variant="destructive" onclick={() => (deleteOpen = true)}>
				<Trash2 class="h-4 w-4" aria-hidden="true" />
				Supprimer
			</Button>
		</div>
	{:else if invoice?.status === 'validated'}
		<div class="flex gap-2">
			<!-- ⛔ Le bouton disparaît quand la facture est soldée, et rien ne le
			     remplace : annuler un règlement demande une contre-passation
			     (issue #414), pas un retrait de drapeau. -->
			{#if !invoice.paidAt}
				<Button variant="outline" onclick={() => (markOpen = true)} data-testid="settle-open">
					{i18nMsg('invoice-settle-button', 'Enregistrer un règlement')}
				</Button>
			{/if}
			<Button
				onclick={downloadPdf}
				disabled={pdfDownloading}
				aria-label={i18nMsg(
					'invoices-download-pdf-aria-label',
					`Télécharger la facture ${invoice.invoiceNumber ?? ''} au format PDF`,
					{ number: invoice.invoiceNumber ?? '' },
				)}
			>
				<Printer class="h-4 w-4" aria-hidden="true" />
				{i18nMsg('invoices-download-pdf', 'Imprimer / Télécharger PDF')}
			</Button>
			{#if canManage}
				{#if featureFlags.smtpConfigured}
					<Button
						variant="outline"
						onclick={openSendEmail}
						disabled={previewLoading}
						data-testid="send-email-button"
					>
						<Mail class="h-4 w-4" aria-hidden="true" />
						{invoice.emailedAt
							? i18nMsg('invoice-resend-email-button', 'Renvoyer par e-mail')
							: i18nMsg('invoice-send-email-button', 'Envoyer par e-mail')}
					</Button>
				{:else}
					<!-- Un <button disabled> ne fire pas le hover : le Trigger délègue
					     à un <span> englobant (snippet child, anti button-in-button).
					     Tooltip.Root embarque déjà son Provider (tooltip.svelte). -->
					<Tooltip.Root>
						<Tooltip.Trigger>
							{#snippet child({ props })}
								<span {...props} data-testid="send-email-disabled-wrapper">
									<Button variant="outline" disabled data-testid="send-email-button">
										<Mail class="h-4 w-4" aria-hidden="true" />
										{invoice?.emailedAt
											? i18nMsg('invoice-resend-email-button', 'Renvoyer par e-mail')
											: i18nMsg('invoice-send-email-button', 'Envoyer par e-mail')}
									</Button>
								</span>
							{/snippet}
						</Tooltip.Trigger>
						<Tooltip.Content class="max-w-xs">
							{i18nMsg(
								'invoice-send-email-smtp-tooltip',
								"L'envoi d'e-mails n'est pas configuré (variables KESH_SMTP_*) — voir le manuel administrateur.",
							)}
						</Tooltip.Content>
					</Tooltip.Root>
				{/if}
			{/if}
			{#if invoice.journalEntryId}
				<Button
					variant="outline"
					onclick={() => goto(`/journal-entries/${invoice!.journalEntryId}`)}
				>
					<BookOpen class="h-4 w-4" aria-hidden="true" />
					Voir l'écriture comptable
				</Button>
			{/if}
			{#if canManage && !invoice.paidAt}
				<Button variant="outline" onclick={() => (creditNoteOpen = true)}>
					{i18nMsg('credit-notes-create-button', 'Créer un avoir')}
				</Button>
			{/if}
			{#if canManage && !invoice.paidAt}
				<!-- Une facture payée est hors du périmètre dunning (jamais candidate
				     à un rappel) : les boutons Suspendre/Reprendre sont masqués, comme
				     les boutons voisins « Créer un avoir » / « Supprimer » (review P1 BH1). -->
				{#if invoice.dunningPausedAt === null}
					<Button
						variant="outline"
						onclick={() => (pauseOpen = true)}
						data-testid="dunning-pause-button"
					>
						{i18nMsg('reminders-pause-button', 'Suspendre les rappels')}
					</Button>
				{:else}
					<Button
						variant="outline"
						onclick={confirmResume}
						disabled={pauseSubmitting}
						data-testid="dunning-resume-button"
					>
						{i18nMsg('reminders-resume-button', 'Reprendre les rappels')}
					</Button>
				{/if}
			{/if}
			{#if isAdmin && !invoice.paidAt}
				<Button variant="destructive" onclick={() => (deleteOpen = true)}>
					<Trash2 class="h-4 w-4" aria-hidden="true" />
					Supprimer
				</Button>
			{/if}
		</div>
	{:else if invoice?.status === 'cancelled'}
		<Button variant="outline" onclick={() => goto('/credit-notes')}>
			{i18nMsg('credit-notes-view-list', 'Voir les avoirs')}
		</Button>
	{/if}
</div>

{#if loading}
	<p class="text-sm text-text-muted">Chargement…</p>
{:else if errorMsg}
	<p class="text-sm text-destructive">{errorMsg}</p>
{:else if invoice}
	<div class="space-y-6" data-testid="invoice-detail">
		<div>
			<h1 class="text-2xl font-semibold">Facture</h1>
			<p class="text-sm text-text-muted">
				{invoice.invoiceNumber ?? 'Brouillon'} — {statusLabel(invoice.status)}
				{#if invoice.status === 'validated'}
					<span class="ml-2">
						<PaymentStatusBadge status={paymentStatus(invoice)} />
					</span>
					{#if invoice.dunningPausedAt}
						<span class="ml-2">
							<DunningPausedBadge note={invoice.dunningPausedNote} />
						</span>
					{/if}
				{/if}
			</p>
		</div>

		<div class="grid grid-cols-2 gap-4 text-sm">
			<div>
				<div class="text-text-muted">Date</div>
				<div>{invoice.date}</div>
			</div>
			<div>
				<div class="text-text-muted">Échéance</div>
				<div>{invoice.dueDate ?? '—'}</div>
			</div>
			<div>
				<div class="text-text-muted">Conditions de paiement</div>
				<div>{invoice.paymentTerms ?? '—'}</div>
			</div>
			{#if projectLabel}
				<div>
					<div class="text-text-muted">
						{i18nMsg('invoice-detail-project-label', 'Projet analytique')}
					</div>
					<div data-testid="invoice-project">{projectLabel}</div>
				</div>
			{/if}
			{#if invoice.paidAt}
				<div>
					<div class="text-text-muted">{i18nMsg('invoice-detail-paid-at-label', 'Payée le')}</div>
					<div>{invoice.paidAt.slice(0, 10)}</div>
				</div>
			{/if}
			{#if invoice.emailedAt}
				<div>
					<div class="text-text-muted">
						{i18nMsg('invoice-detail-emailed-at-label', 'Envoyée le')}
					</div>
					<div data-testid="invoice-emailed-at">{invoice.emailedAt.slice(0, 10)}</div>
					<div class="text-text-muted" data-testid="invoice-emailed-to">{invoice.emailedTo}</div>
				</div>
			{/if}
		</div>

		<table class="w-full border-collapse text-sm">
			<thead>
				<tr class="border-b border-border text-left">
					<th class="py-2 pr-2">Description</th>
					<th class="py-2 pr-2">Quantité</th>
					<th class="py-2 pr-2">Prix unitaire</th>
					<th class="py-2 pr-2">TVA %</th>
					<!--
						AC6-bis : la colonne suit la convention du fichier qu'elle modifie.
						Les 5 en-têtes existants sont en français codé en dur ; les mettre
						en i18n imposerait de les ajouter aux 4 catalogues, hors périmètre.
						L'écran d'AVOIR, lui, utilise `i18nMsg` — et la colonne y suit
						cette convention-là.
					-->
					<th class="py-2 pr-2">Compte de produit</th>
					<th class="py-2 pr-2 text-right">Total</th>
				</tr>
			</thead>
			<tbody>
				{#each invoice.lines as l (l.id)}
					<tr class="border-b border-border">
						<td class="py-2 pr-2">{l.description}</td>
						<td class="py-2 pr-2">{l.quantity}</td>
						<td class="py-2 pr-2 font-mono">{formatInvoiceTotal(l.unitPrice)}</td>
						<td class="py-2 pr-2">{l.vatRate}%</td>
						<td class="py-2 pr-2" data-testid="invoice-line-revenue-account">
							{revenueAccountCell(l.revenueAccountId)}
						</td>
						<td class="py-2 pr-2 text-right font-mono">
							{formatInvoiceTotal(l.lineTotal)}
						</td>
					</tr>
				{/each}
			</tbody>
			<tfoot>
				<!-- #151 : récap TVA. Si des lignes sont taxées, on affiche
				     Sous-total HT → TVA {taux}% (par taux) → Total TTC. Sinon
				     (aucune TVA) un simple « Total » = TTC (== HT). -->
				{#if invoice.vatBreakdown.length > 0}
					<tr>
						<td colspan="5" class="py-1 text-right">Sous-total HT</td>
						<td class="py-1 text-right font-mono">{formatInvoiceTotal(invoice.totalAmount)}</td>
					</tr>
					{#each invoice.vatBreakdown as vb (vb.ratePercent)}
						<tr>
							<td colspan="5" class="py-1 text-right">TVA {Number(vb.ratePercent)}%</td>
							<td class="py-1 text-right font-mono">{formatInvoiceTotal(vb.vatAmount)}</td>
						</tr>
					{/each}
					<tr>
						<td colspan="5" class="py-3 text-right font-semibold">Total TTC</td>
						<td class="py-3 text-right font-mono text-lg font-semibold">
							{formatInvoiceTotal(invoice.totalTtc)}
						</td>
					</tr>
				{:else}
					<tr>
						<td colspan="5" class="py-3 text-right font-semibold">Total</td>
						<td class="py-3 text-right font-mono text-lg font-semibold">
							{formatInvoiceTotal(invoice.totalTtc)}
						</td>
					</tr>
				{/if}
				<!--
				  Story 24-2 (#371) — ce qui a été réglé, et ce qui reste dû.

				  ⚠️ Rendu SEULEMENT si un règlement existe : sur une facture
				  intacte, « Réglé 0.00 / Reste dû 100.00 » n'apprend rien et
				  répète le total juste au-dessus. Le bruit n'est pas neutre — il
				  fait passer inaperçu le cas où ces lignes DISENT quelque chose.
				-->
				{#if invoice.amountSettled !== null && Number(invoice.amountSettled) > 0}
					<tr class="border-t">
						<td colspan="5" class="py-1 text-right">
							{i18nMsg('invoice-amount-settled', 'Déjà réglé')}
						</td>
						<td class="py-1 text-right font-mono" data-testid="invoice-amount-settled">
							{formatInvoiceTotal(invoice.amountSettled)}
						</td>
					</tr>
					<tr>
						<td colspan="5" class="py-1 text-right font-semibold">
							{i18nMsg('invoice-amount-due', 'Reste dû')}
						</td>
						<td
							class="py-1 text-right font-mono font-semibold"
							data-testid="invoice-amount-due"
						>
							{formatInvoiceTotal(invoice.amountDue ?? '0')}
						</td>
					</tr>
				{/if}
			</tfoot>
		</table>

		{#if invoice.status === 'validated'}
			<!-- Story 21-6c : historique des rappels (rappels uniquement sur factures validées). -->
			<ReminderHistory {reminders} />
		{/if}
	</div>

	<Dialog.Root
		open={deleteOpen}
		onOpenChange={(o) => {
			deleteOpen = o;
			if (!o) {
				deleteError = '';
				deleteConfirmText = '';
			}
		}}
	>
		<Dialog.Content>
			<Dialog.Header>
				<Dialog.Title>Supprimer la facture</Dialog.Title>
			</Dialog.Header>
			{#if invoice?.status === 'validated'}
				<!-- #219 : confirmation forte pour une facture validée (efface la
				     facture ET son écriture comptable). -->
				<div class="space-y-3 text-sm">
					<div
						class="rounded-md border border-destructive bg-destructive/10 px-3 py-2 text-destructive"
					>
						<strong>Action irréversible.</strong> La facture
						<strong>{invoice.invoiceNumber}</strong> et son écriture comptable
						associée seront <strong>définitivement supprimées</strong>. À réserver
						à la correction d'une validation erronée ou à la purge d'essais.
					</div>
					<label class="block space-y-1">
						<span class="text-text-muted">
							Pour confirmer, retapez le numéro de facture
							<strong>{invoice.invoiceNumber}</strong>
						</span>
						<Input
							bind:value={deleteConfirmText}
							autocomplete="off"
							aria-label="Confirmer en retapant le numéro de facture"
						/>
					</label>
				</div>
			{:else}
				<p class="text-sm">Confirmer la suppression définitive de cette facture brouillon ?</p>
			{/if}
			{#if deleteError}
				<div class="rounded-md border border-destructive bg-destructive/10 px-3 py-2 text-sm text-destructive">
					{deleteError}
				</div>
			{/if}
			<Dialog.Footer>
				<Button variant="outline" onclick={() => (deleteOpen = false)}>Annuler</Button>
				<Button
					variant="destructive"
					onclick={confirmDelete}
					disabled={deleteSubmitting ||
						(invoice?.status === 'validated' &&
							deleteConfirmText.trim() !== invoice?.invoiceNumber)}
				>
					Supprimer
				</Button>
			</Dialog.Footer>
		</Dialog.Content>
	</Dialog.Root>

	<Dialog.Root
		open={validateOpen}
		onOpenChange={(o) => {
			validateOpen = o;
			if (!o) validateError = '';
		}}
	>
		<Dialog.Content data-testid="invoice-validate-dialog">
			<Dialog.Header>
				<!-- ⚠️ Ce titre était un NŒUD DE TEXTE : ni fonction, ni littéral JS, donc hors de
				     portée de la garde `i18n-libelle-en-dur.test.ts` — corrigé à la main, et le
				     motif est écrit dans son préambule. La clé existait DÉJÀ, traduite dans les
				     quatre catalogues ; c'est le câblage qui manquait.
				     ⚠️ `data-testid` ci-dessus : `fiscal-years.spec.ts` ciblait ce dialogue PAR SON
				     NOM français. Un sélecteur figé sur un libellé traduit se casse à la première
				     locale, et la réparation la moins coûteuse — y réécrire la chaîne traduite —
				     remettrait le verrou en place sous couvert de correctif. -->
				<Dialog.Title>{i18nMsg('invoice-validate-confirm-title', 'Valider la facture')}</Dialog.Title>
			</Dialog.Header>
			<p class="text-sm">
				Une fois validée, cette facture sera immuable, recevra un numéro définitif et
				générera une écriture comptable. Continuer&nbsp;?
			</p>
			{#if validateError}
				<div class="rounded-md border border-destructive bg-destructive/10 px-3 py-2 text-sm text-destructive">
					{validateError}
				</div>
			{/if}
			<Dialog.Footer>
				<Button variant="outline" onclick={() => (validateOpen = false)}>Annuler</Button>
				<Button onclick={confirmValidate} disabled={validateSubmitting}>Valider</Button>
			</Dialog.Footer>
		</Dialog.Content>
	</Dialog.Root>

	<SettleInvoiceDialog
		open={markOpen}
		onOpenChange={(o: boolean) => {
			markOpen = o;
			if (!o) markError = '';
		}}
		invoiceDate={invoice.date}
		amountDue={invoice.amountDue}
		accounts={settleAccounts}
		bankAccounts={settleBankAccounts}
		submitting={markSubmitting}
		errorMsg={markError}
		onConfirm={handleSettleConfirm}
	/>

	<SendEmailDialog
		open={sendEmailOpen}
		onOpenChange={(o) => {
			// Review 20-3b2 P1 (BH/ECH HIGH) : l'e-mail part côté serveur — la
			// modale ne doit PAS être fermable (ESC/X/clic-extérieur) pendant
			// l'envoi en vol, sinon l'erreur inline 422/500 serait perdue et
			// un re-clic déclencherait un vrai second envoi.
			if (!o && sendEmailSubmitting) return;
			sendEmailOpen = o;
			if (!o) sendEmailError = '';
		}}
		preview={sendEmailPreview}
		submitting={sendEmailSubmitting}
		errorMsg={sendEmailError}
		onConfirm={handleSendEmailConfirm}
	/>

	<DunningPauseDialog
		open={pauseOpen}
		onOpenChange={(o) => {
			// Anti-double-submit (patron SendEmailDialog) : la modale n'est pas
			// fermable (ESC/X/clic-extérieur) pendant la suspension en vol.
			if (!o && pauseSubmitting) return;
			pauseOpen = o;
			if (!o) pauseError = '';
		}}
		invoiceLabel={invoice.invoiceNumber ?? String(invoice.id)}
		submitting={pauseSubmitting}
		errorMsg={pauseError}
		onConfirm={confirmPause}
	/>

	<!-- ⛔ Story 24-3 (#372) : le dialogue « Dé-marquer payée » est SUPPRIMÉ.
	     Il ne coûtait rien parce que le marquage n'écrivait rien. Depuis que le
	     règlement produit son écriture, revenir en arrière demande une
	     contre-passation — une écriture inverse, à sa propre date, qui laisse les
	     deux visibles au grand livre. C'est l'objet de l'issue #414, qui couvre
	     aussi le côté fournisseur, lequel n'a jamais su le faire. -->

	<Dialog.Root
		open={creditNoteOpen}
		onOpenChange={(o) => {
			creditNoteOpen = o;
			if (!o) creditNoteError = '';
		}}
	>
		<Dialog.Content>
			<Dialog.Header>
				<Dialog.Title>{i18nMsg('credit-notes-create-button', 'Créer un avoir')}</Dialog.Title>
			</Dialog.Header>
			<p class="text-sm">
				{i18nMsg(
					'credit-notes-confirm-body',
					'Un avoir total sera créé et comptabilisé immédiatement : il contre-passe l’écriture de cette facture (le solde du client revient à zéro) et la facture passe au statut « annulée ». Cette action est définitive. Continuer ?',
				)}
			</p>
			{#if creditNoteError}
				<div class="rounded-md border border-destructive bg-destructive/10 px-3 py-2 text-sm text-destructive">
					{creditNoteError}
				</div>
			{/if}
			<Dialog.Footer>
				<Button variant="outline" onclick={() => (creditNoteOpen = false)}>
					{i18nMsg('common-cancel', 'Annuler')}
				</Button>
				<Button onclick={confirmCreateCreditNote} disabled={creditNoteSubmitting}>
					{i18nMsg('credit-notes-create-button', 'Créer un avoir')}
				</Button>
			</Dialog.Footer>
		</Dialog.Content>
	</Dialog.Root>
{/if}
