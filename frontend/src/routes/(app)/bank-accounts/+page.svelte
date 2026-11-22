<!--
  Story 8-5a-zero — Page de configuration des comptes bancaires.
  Story v014-1 — CRUD complet post-onboarding (création, édition complète,
  archivage, toggle archivés, affichage solde).
  Story 26-2a (#622) — description optionnelle : saisie (création, édition),
  nom du compte dans le tableau par `bankAccountName`.
  Story 28-4b (#642) — clore et rouvrir un compte bancaire : bascule des clos
  (masqués par défaut, C-28-4-13), badge « Clos le … », seule action « Rouvrir »
  sur une ligne close, « Clore » désactivé sur le principal, dialogue de clôture
  (date locale préremplie, avertissement d'un solde comptable non nul, C-28-4-6).
  Story 38-11d (AC-1) — case « Compte PayPal » sous le type *Compte bancaire* :
  libellé du compte PayPal, compte de frais, IBAN facultatif, QR-IBAN et
  « principal » masqués, compte du plan obligatoire et admis en devise ; ligne
  « PayPal · <compte> · <devise> » dans la liste ; pas de « Lier » (`PATCH`
  refusé par le serveur sur un compte PayPal).
-->
<script lang="ts">
	import { onMount, untrack } from 'svelte';
	import Big from 'big.js';
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import { errorMessageOf } from '$lib/shared/utils/error-message';
	import { isApiError } from '$lib/shared/utils/api-error';
	// Story 30-2b (C-30-2-20) : la classification QR-IBAN est partagée avec le
	// formulaire de facture fournisseur — une seule définition côté écran.
	import { qrIbanHasValidQrIid } from '$lib/shared/utils/iban';
	import { fetchAccounts } from '$lib/features/accounts/accounts.api';
	import {
		accountCurrency,
		isForeignCurrencyAccount,
		type AccountResponse,
	} from '$lib/features/accounts/accounts.types';
	import { withCurrentAccount } from '$lib/features/accounts/account-options';
	import { getInvoiceSettings } from '$lib/features/invoices/invoices.api';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { toast } from 'svelte-sonner';
	import BankAccountJournalLinkForm from '$lib/features/bank-accounts/BankAccountJournalLinkForm.svelte';
	import {
		listBankAccounts,
		createBankAccount,
		updateBankAccount,
		archiveBankAccount,
		closeBankAccount,
		reopenBankAccount,
		enterOpeningBalance,
		type BankAccountKind,
		type BankAccountSummary,
		type NewBankAccountPayload,
		type OpeningBalanceSide,
		type OpeningEntrySummary,
		type UpdateBankAccountPayload,
	} from '$lib/features/bank-accounts/bank-accounts.api';
	import {
		admitsLedgerType,
		cardLast4Payload,
		isValidCardLast4,
	} from '$lib/features/bank-accounts/credit-card';
	import {
		PROVIDER_ACCOUNT_MAX_CHARS,
		feeAccountCandidates,
		isProvider,
		payoutAccountCandidates,
		providerAccountLabels,
		providerPayload,
		providerSettingsOf,
		providerSummaryLine,
		type ConfigurableProvider,
	} from '$lib/features/bank-accounts/provider';
	import { formatSwissDate } from '$lib/features/reports/reports.api';
	import { todayLocal } from '$lib/features/open-items/open-items';
	import {
		BANK_ACCOUNT_DESCRIPTION_MAX,
		bankAccountFullId,
		bankAccountName,
		defaultOpeningSide,
		formatChfBalance,
		formatChfExact,
		linkableLedgerAccounts,
		openingAmountKind,
		openingAmountPayload,
		openingAmountPreview,
		type OpeningAmountKind,
	} from '$lib/features/bank-accounts/format';

	let bankAccounts = $state<BankAccountSummary[]>([]);
	let accounts = $state<AccountResponse[]>([]);
	// Story 15-6c (#474, AC7) : le compte débiteurs et le compte créanciers
	// DÉSIGNÉS dans les réglages — pas les comptes de rôle. Ensemble vide si les
	// réglages n'ont pas pu être lus : pas de filtre, le refus serveur reste le
	// filet.
	let claimAccountIds = $state<Set<number>>(new Set());
	let loading = $state(true);
	let loadError = $state<string | null>(null);
	let includeArchived = $state(false);
	// Story 28-4b (#642, C-28-4-13) — les comptes clos sont masqués par défaut.
	let includeClosed = $state(false);
	/**
	 * Revue P3 de 40-7d (P3-1) — tous les comptes bancaires, archivés et clos compris, lus à part
	 * pour le compte de versement d'un compte Stripe : sa destination, archivée ou close depuis,
	 * s'affiche alors même quand la liste de l'écran les masque. `null` si la lecture a échoué :
	 * on retombe sur la liste de l'écran, et une destination absente n'est jamais vidée.
	 */
	let payoutPool = $state<BankAccountSummary[] | null>(null);
	let payoutSource = $derived(payoutPool ?? bankAccounts);

	// UI state — modes d'édition. Un seul mode actif à la fois.
	type Mode =
		| { kind: 'none' }
		| { kind: 'create' }
		| { kind: 'edit-full'; id: number }
		| { kind: 'edit-journal'; id: number }
		| { kind: 'archive-confirm'; id: number }
		| { kind: 'close-confirm'; id: number }
		| { kind: 'reopen-confirm'; id: number }
		| { kind: 'opening-balance'; id: number };
	let mode = $state<Mode>({ kind: 'none' });
	let submitting = $state(false);

	// Form state — partagé create/edit-full.
	let formBankName = $state('');
	let formDescription = $state('');
	let formIban = $state('');
	let formQrIban = $state('');
	let formIsPrimary = $state(false);
	let formJournalAccountId = $state<number | null>(null);
	let formVersion = $state(0);
	let formError = $state<string | null>(null);
	// Story 34-2 (#533) — solde d'ouverture : formulaire de création et dialogue
	// de ligne (« Saisir le solde d'ouverture »). Le sens est présélectionné par
	// le type du compte lié (C-e341-7), l'utilisateur le change librement.
	let formOpeningAmount = $state('');
	let formOpeningSide = $state<OpeningBalanceSide>('debit');
	let openingError = $state<string | null>(null);

	// La présélection suit le compte lié choisi — elle ne relit pas `accounts`
	// d'elle-même (un rechargement ne défait pas un choix fait à la main).
	$effect(() => {
		const ledgerId = formJournalAccountId;
		untrack(() => {
			formOpeningSide = defaultOpeningSide(accounts, ledgerId);
		});
	});
	// Story 34-11a2 (#972) — type du compte (choisi à la création, affiché et figé
	// en édition) et quatre derniers chiffres d'une carte.
	let formKind = $state<BankAccountKind>('bank');
	let formCardLast4 = $state('');
	let isCard = $derived(formKind === 'credit_card');
	// Story 38-11d (AC-1) — compte PayPal (sous le type *Compte bancaire*
	// seulement) : libellé du compte PayPal et compte de frais. Story 40-7d
	// (AC-1) — généralisé au compte Stripe (une case par prestataire, l'une
	// exclut l'autre), qui porte en plus son compte bancaire de versement.
	let formProvider = $state<ConfigurableProvider | null>(null);
	let formProviderAccount = $state('');
	let formFeeAccountId = $state<number | null>(null);
	let formPayoutBankAccountId = $state<number | null>(null);
	// En édition : le prestataire du compte à l'ouverture (décocher envoie `paypal: null` ou `stripe: null`).
	let formWasProvider = $state<ConfigurableProvider | null>(null);
	let isProviderForm = $derived(!isCard && formProvider !== null);
	let isStripeForm = $derived(isProviderForm && formProvider === 'stripe');
	// Devise du compte du plan choisi (`null` = CHF ou aucun).
	let formLedgerCurrency = $derived.by(() => {
		const ledger = accounts.find((a) => a.id === formJournalAccountId);
		return ledger ? accountCurrency(ledger) : null;
	});
	let feeCandidates = $derived(feeAccountCandidates(accounts, formLedgerCurrency));
	let providerLabels = $derived(formProvider ? providerAccountLabels(bankAccounts, formProvider) : []);
	// Story 40-7d (AC-1) — comptes de versement d'un compte Stripe : même devise que le compte du
	// plan choisi, sans prestataire ou Wise, jamais le compte édité.
	let payoutCandidates = $derived(
		payoutAccountCandidates(
			payoutSource,
			accounts,
			formLedgerCurrency,
			mode.kind === 'edit-full' ? mode.id : null,
			payoutKeepId(),
		),
	);
	/**
	 * Revue P2 de 40-7d (P2-1) — la destination choisie reste offerte, archivée ou close, tant que le
	 * compte du plan est celui de l'ouverture : le serveur ne la rejuge pas alors (`payout_is_unchanged`)
	 * et la refuse (`STRIPE_PAYOUT_ACCOUNT_INVALID`) dès que le compte du plan change.
	 */
	function payoutKeepId(): number | null {
		return ledgerUnchanged() ? formPayoutBankAccountId : null;
	}

	/** Revue P3 de 40-7d (P3-1) — en modification, le compte du plan est-il encore celui de l'ouverture ? */
	function ledgerUnchanged(): boolean {
		if (mode.kind !== 'edit-full') return false;
		const id = mode.id;
		const original = bankAccounts.find((b) => b.id === id);
		return original !== undefined && original.journalAccountId === formJournalAccountId;
	}
	// Le compte de frais se vide seulement si sa devise n'est plus admise par celle du compte du
	// plan choisi (revue P1, L7 : le vider à tout changement effaçait en silence un compte choisi
	// avant le compte du plan ; revue P2, L3 : ni un compte archivé ou non imputable, que le
	// serveur juge et refuse sous son code). Il reste affiché tant que les comptes ne sont pas
	// chargés.
	$effect(() => {
		const ledger = formLedgerCurrency;
		untrack(() => {
			if (accounts.length === 0 || formFeeAccountId === null) return;
			const fee = accounts.find((a) => a.id === formFeeAccountId);
			if (!fee) return;
			const c = accountCurrency(fee);
			if (c !== null && c !== ledger) formFeeAccountId = null;
		});
	});
	// Story 40-7d (AC-1) — le compte de versement se vide seulement quand l'utilisateur change le
	// compte du plan, et seulement si la destination, connue, n'est plus admise : autre devise
	// (même devise que le solde), ou destination archivée ou close (le serveur la rejuge alors,
	// revue P2, P2-1). Tant que le compte du plan est celui de l'ouverture, rien ne se vide : la
	// destination archivée ou close reste choisie, marquée (revue P1, M1), et un simple renommage
	// la renvoie telle quelle (revue P3, P3-1). Une destination absente de la liste lue (lecture
	// en échec) n'est jamais vidée : on ne retire pas ce qu'on ne voit pas. Rien ne se vide non
	// plus tant que les comptes du plan ne sont pas chargés.
	$effect(() => {
		const ledger = formLedgerCurrency;
		// Le compte du plan lui-même est suivi : un changement à même devise retire une destination
		// archivée ou close (revue P2, P2-1).
		void formJournalAccountId;
		untrack(() => {
			if (accounts.length === 0 || formPayoutBankAccountId === null || ledgerUnchanged()) return;
			const id = formPayoutBankAccountId;
			if (!payoutSource.some((b) => b.id === id)) return;
			const kept = payoutAccountCandidates(
				payoutSource,
				accounts,
				ledger,
				mode.kind === 'edit-full' ? mode.id : null,
				// Le compte du plan a changé : aucune destination archivée ou close n'est gardée.
				null,
			);
			if (!kept.some((b) => b.id === id)) formPayoutBankAccountId = null;
		});
	});
	// Un compte du plan en devise n'admet pas de solde d'ouverture par cet écran.
	let ledgerIsForeign = $derived(formLedgerCurrency !== null);
	// Passe de textes de la 0.17.6 (#1040) — le solde d'ouverture porte-t-il sur une
	// carte ? Au formulaire de création, le type choisi ; au dialogue de ligne, celui du
	// compte de la ligne. Sert au seul libellé du sens « Dette ».
	let openingForCard = $derived.by(() => {
		const m = mode;
		if (m.kind !== 'opening-balance') return isCard;
		return bankAccounts.find((b) => b.id === m.id)?.accountKind === 'credit_card';
	});

	// Story 28-4b — dialogues de clôture et de réouverture. Le refus du serveur
	// s'affiche DANS le dialogue (là où l'on a agi), par `errorMessageOf`.
	let closeDate = $state('');
	let closeError = $state<string | null>(null);
	let reopenError = $state<string | null>(null);
	// Story 28-4b (passe ciblée P2, M1) — vrai quand `reload()` a refermé un mode dont le
	// compte visé n'est plus dans la liste relue (voir `closeOrphanMode`).
	let staleNotice = $state(false);

	/**
	 * Story 28-4b (passe ciblée P2, M1) — ferme la classe « mode orphelin » là où elle
	 * naît : à la fin de `reload()`, et nulle part ailleurs. Un mode qui vise un compte —
	 * `edit-full`, `edit-journal`, `archive-confirm`, `close-confirm`, `reopen-confirm`, soit
	 * tous ceux qui portent un `id` — et dont le compte n'est plus dans la liste relue (clos
	 * ou archivé ailleurs, ou masqué par une bascule) se referme, avec un message neutre :
	 * sans quoi son dialogue disparaissait avec la ligne et le mode, resté ouvert, masquait
	 * les actions de toutes les lignes. `create` ne vise aucun compte : il reste ouvert.
	 *
	 * Les gestes qui retirent eux-mêmes leur compte de la liste (archivage, clôture,
	 * réouverture) referment leur mode AVANT de recharger : la garde ne parle pas d'un geste
	 * réussi.
	 */
	function closeOrphanMode() {
		const cible = 'id' in mode ? mode.id : null;
		if (cible !== null && !bankAccounts.some((b) => b.id === cible)) {
			closeForm();
			staleNotice = true;
		}
	}

	async function reload() {
		// Revue P3 de 40-7d (P3-1) — les destinations de versement, archivées et closes comprises,
		// lues avant la liste de l'écran ; la même lecture quand l'écran montre déjà tout.
		const pool = includeArchived && includeClosed ? null : listBankAccounts(true, true);
		const shown = listBankAccounts(includeArchived, includeClosed);
		const [baResult, poolResult, accResult, settingsResult] = await Promise.allSettled([
			shown,
			pool ?? shown,
			fetchAccounts(false),
			getInvoiceSettings(),
		]);
		payoutPool = poolResult.status === 'fulfilled' ? poolResult.value : null;
		if (baResult.status === 'fulfilled') {
			bankAccounts = baResult.value;
			loadError = null;
			staleNotice = false;
			closeOrphanMode();
		} else {
			loadError = errorMessageOf(baResult.reason);
		}
		if (accResult.status === 'fulfilled') {
			accounts = accResult.value;
		} else {
			accounts = [];
		}
		// Échec des réglages : aucune erreur affichée pour ce seul appel.
		claimAccountIds =
			settingsResult.status === 'fulfilled'
				? new Set(
						[
							settingsResult.value.defaultReceivableAccountId,
							settingsResult.value.defaultPayableAccountId,
						].filter((id): id is number => id !== null && id !== undefined),
					)
				: new Set();
	}

	onMount(async () => {
		await reload();
		loading = false;
	});

	// Filtrer accounts dropdown : classes 1 (Asset) + 2 (Liability) actifs.
	// Story 15-5b (AC13, #271) : les deux `<select>` passent par
	// `withCurrentAccount`, pour qu'un compte lié devenu non imputable reste
	// affiché ; `BankAccountJournalLinkForm` reçoit la liste COMPLÈTE et filtre
	// lui-même (choix C26) — une liste déjà filtrée l'empêchait de retrouver ce
	// compte, et le champ s'affichait vide.
	//
	// Story 15-6c (#474, AC7) : le compte débiteurs et le compte créanciers
	// désignés sont écartés AVANT `withCurrentAccount` — un lien existant vers
	// l'un d'eux (donnée antérieure) reste affiché, mais n'est plus proposé.
	//
	// Story 34-1 (#966, C-e341-5) : le filtre vit dans `linkableLedgerAccounts`
	// (`format.ts`), source unique partagée avec l'étape bancaire de l'assistant
	// et le formulaire de lien.
	// Story 34-11a2 (#972, C-e341-33) : pour une carte, la liste se réduit aux
	// passifs (`admitsLedgerType`, miroir de la règle serveur).
	// Story 38-11d (AC-1) : les comptes en devise ne sont offerts qu'au compte
	// PayPal (case cochée) — `allowForeignCurrency`.
	let linkableAccounts = $derived(
		linkableLedgerAccounts(accounts, claimAccountIds, {
			liabilityOnly: formKind === 'credit_card',
			allowForeignCurrency: isProviderForm,
		}),
	);

	function accountLabel(journalAccountId: number | null): string {
		if (journalAccountId === null) {
			return i18nMsg('bank-accounts-labels-not-configured', 'Non configuré');
		}
		const acc = accounts.find((a) => a.id === journalAccountId);
		if (!acc) return `#${journalAccountId}`;
		return `${acc.number} — ${acc.name}`;
	}

	/** Story 38-11d — le compte du plan lié porte-t-il une devise ? (pas de solde d'ouverture par cet écran) */
	function isForeignLedger(journalAccountId: number): boolean {
		const acc = accounts.find((a) => a.id === journalAccountId);
		return acc !== undefined && isForeignCurrencyAccount(acc);
	}

	function formatBalance(balance: string | null): string {
		if (balance === null) {
			return i18nMsg(
				'bank-accounts-labels-balance-unavailable',
				'Solde non disponible (lier au plan comptable)',
			);
		}
		return formatChfBalance(balance);
	}

	function resetForm() {
		formBankName = '';
		formDescription = '';
		formIban = '';
		formQrIban = '';
		formIsPrimary = false;
		formJournalAccountId = null;
		formVersion = 0;
		formError = null;
		formKind = 'bank';
		formCardLast4 = '';
		formOpeningAmount = '';
		formOpeningSide = 'debit';
		openingError = null;
		formProvider = null;
		formProviderAccount = '';
		formFeeAccountId = null;
		formPayoutBankAccountId = null;
		formWasProvider = null;
	}


	/**
	 * Story 35-6 (#1026, D6) — le refus, à l'écran, d'une saisie qui ne part pas au
	 * serveur ; `null` pour une saisie claire. Ambiguë (« 1.234 », « 1,234 » : mille ou
	 * un franc ?), illisible (« 1. 234 », « 1e3 »), négative ou vide (montant positif
	 * demandé). Clés et replis littéraux, lus par les gardes i18n.
	 */
	function openingAmountRefusal(kind: OpeningAmountKind): string | null {
		switch (kind) {
			case 'ambiguous':
				return i18nMsg(
					'bank-accounts-opening-error-amount-ambiguous',
					"Montant ambigu : un point ou une virgule suivis de trois chiffres peuvent séparer les milliers. Écrivez 1'234 pour mille deux cent trente-quatre, ou 1.2340 pour un franc vingt-trois et quatre dixièmes de centime.",
				);
			case 'unreadable':
				return i18nMsg(
					'bank-accounts-opening-error-amount-unreadable',
					"Montant illisible : écrivez un nombre positif, par exemple 1'234.50.",
				);
			case 'negative':
			case 'empty':
				return i18nMsg('bank-accounts-opening-error-amount-required', 'Saisissez un montant positif.');
			case 'clear':
				return null;
		}
	}

	/**
	 * Story 34-2 (#533, AC-7) — ouvre le dialogue « Saisir le solde d'ouverture »
	 * d'un compte ouvert et lié ; le sens est présélectionné par le type du compte lié.
	 */
	function openOpeningBalance(ba: BankAccountSummary) {
		formOpeningAmount = '';
		formOpeningSide = defaultOpeningSide(accounts, ba.journalAccountId);
		openingError = null;
		mode = { kind: 'opening-balance', id: ba.id };
	}

	/**
	 * Story 34-2 (#533, AC-6, AC-7) — passe l'écriture d'ouverture d'un compte
	 * existant. Le refus du serveur (compte mouvementé, aucun exercice ouvert…)
	 * reste affiché dans le dialogue.
	 */
	async function confirmOpeningBalance(id: number) {
		openingError = null;
		// Story 35-6 — la saisie est classée avant tout envoi ; seule une saisie claire part.
		const reading = openingAmountKind(formOpeningAmount);
		const refusal = openingAmountRefusal(reading.kind);
		if (refusal !== null || reading.payload === undefined) {
			openingError = refusal;
			return;
		}
		const amount = reading.payload;
		submitting = true;
		try {
			const entry = await enterOpeningBalance(id, { amount, side: formOpeningSide });
			toast.success(
				i18nMsg(
					'bank-accounts-toast-opening-success',
					'Écriture d\'ouverture n° { $number } du { $date } passée : { $amount }.',
					// Avis qui nomme l'écriture (numéro, date suisse) et le montant écrit
					// (revue P1, B-1) ; clé et arguments littéraux, lus par les gardes i18n.
					{
						number: String(entry.entryNumber),
						date: formatSwissDate(entry.entryDate),
						amount: formatChfExact(amount),
					},
				),
			);
			closeForm();
			await reload();
		} catch (err) {
			openingError = errorMessageOf(err);
		} finally {
			submitting = false;
		}
	}

	/**
	 * Story 34-11a2 — changer le type à la création : un compte lié que le
	 * nouveau type n'admet pas (un actif pour une carte) est retiré, et le
	 * « principal » d'une carte décoché.
	 */
	function setFormKind(kind: BankAccountKind) {
		formKind = kind;
		if (kind === 'credit_card') {
			formIsPrimary = false;
			formProvider = null;
		}
		const linked = accounts.find((a) => a.id === formJournalAccountId);
		if (linked && !admitsLedgerType(kind, linked.accountType)) formJournalAccountId = null;
	}

	/**
	 * Story 34-11a2 — contrôles de l'écran propres à une carte : quatre chiffres
	 * ou rien, compte lié obligatoire. Le serveur reste l'autorité.
	 */
	function validateCardFields(): boolean {
		if (!isCard) return true;
		if (!isValidCardLast4(formCardLast4)) {
			formError = i18nMsg(
				'bank-accounts-error-card-last4',
				'Saisissez exactement quatre chiffres, ou laissez le champ vide.',
			);
			return false;
		}
		if (formJournalAccountId === null) {
			formError = i18nMsg(
				'bank-accounts-error-card-ledger-required',
				'Choisissez le compte de passif lié à la carte.',
			);
			return false;
		}
		return true;
	}

	/**
	 * Story 38-11d (AC-1) — cocher ou décocher « Compte PayPal » ; Story 40-7d, de
	 * même « Compte Stripe » (l'une exclut l'autre). Cochée : le QR-IBAN et
	 * « principal » tombent (refusés par le serveur), et le compte de versement,
	 * propre à Stripe, se vide en passant à PayPal. Décochée : un compte du plan en
	 * devise, que seul un compte de prestataire admet, est retiré.
	 */
	function setFormProvider(provider: ConfigurableProvider, checked: boolean) {
		if (checked) {
			formProvider = provider;
			formQrIban = '';
			formIsPrimary = false;
			if (provider !== 'stripe') formPayoutBankAccountId = null;
		} else if (formProvider === provider) {
			formProvider = null;
			formPayoutBankAccountId = null;
			const ledger = accounts.find((a) => a.id === formJournalAccountId);
			if (ledger && isForeignCurrencyAccount(ledger)) formJournalAccountId = null;
		}
	}

	/**
	 * Story 38-11d (AC-1) — contrôles de l'écran d'un compte de prestataire :
	 * libellé de 1 à 120 caractères, compte du plan obligatoire. Le serveur reste
	 * l'autorité (`PROVIDER_LEDGER_REQUIRED`, …). Clés et replis littéraux par
	 * prestataire, lus par les gardes i18n.
	 */
	function validateProviderFields(): boolean {
		if (!isProviderForm) return true;
		const label = formProviderAccount.trim();
		if (label === '' || label.length > PROVIDER_ACCOUNT_MAX_CHARS) {
			formError =
				formProvider === 'stripe'
					? i18nMsg(
							'bank-accounts-error-stripe-account',
							'Saisissez le libellé du compte Stripe (1 à 120 caractères), par exemple « Stripe boutique ».',
						)
					: i18nMsg(
							'bank-accounts-error-paypal-account',
							'Saisissez le libellé du compte PayPal (1 à 120 caractères), par exemple « PayPal www ».',
						);
			return false;
		}
		if (formJournalAccountId === null) {
			formError =
				formProvider === 'stripe'
					? i18nMsg(
							'bank-accounts-error-stripe-ledger-required',
							'Choisissez le compte du plan lié : sa devise est celle du solde Stripe.',
						)
					: i18nMsg(
							'bank-accounts-error-paypal-ledger-required',
							'Choisissez le compte du plan lié : sa devise est celle du solde PayPal.',
						);
			return false;
		}
		return true;
	}

	/** Story 40-7d — la charge du prestataire du formulaire (`{ paypal }` ou `{ stripe }`), `null` hors prestataire. */
	function formProviderPayload() {
		return isProviderForm && formProvider !== null
			? providerPayload(formProvider, formProviderAccount, formFeeAccountId, isStripeForm ? formPayoutBankAccountId : null)
			: null;
	}

	/** Story 38-11d — l'IBAN envoyé par un compte de prestataire : `null` si le champ est vide. */
	function providerIban(): string | null {
		return formIban.trim() === '' ? null : formIban;
	}

	function kindLabel(kind: BankAccountKind): string {
		return kind === 'credit_card'
			? i18nMsg('bank-accounts-kind-credit-card', 'Carte de crédit')
			: i18nMsg('bank-accounts-kind-bank', 'Compte bancaire');
	}

	function openCreate() {
		resetForm();
		mode = { kind: 'create' };
	}

	function openEditFull(ba: BankAccountSummary) {
		formBankName = ba.bankName;
		formDescription = ba.description ?? '';
		// Story 34-11a2 (#972, points hérités E5/B7) — une carte s'édite sans
		// IBAN : le payload d'une carte n'en porte pas (`submitUpdateFull`).
		formIban = ba.iban ?? '';
		formQrIban = ba.qrIban ?? '';
		formIsPrimary = ba.isPrimary;
		formJournalAccountId = ba.journalAccountId;
		formVersion = ba.version;
		formError = null;
		formKind = ba.accountKind;
		formCardLast4 = ba.cardLast4 ?? '';
		formProvider = ba.provider === 'paypal' || ba.provider === 'stripe' ? ba.provider : null;
		formWasProvider = formProvider;
		const settings = formProvider ? providerSettingsOf(ba, formProvider) : null;
		formProviderAccount = settings?.account ?? '';
		formFeeAccountId = settings?.feeAccountId ?? null;
		formPayoutBankAccountId = ba.stripe?.payoutBankAccountId ?? null;
		mode = { kind: 'edit-full', id: ba.id };
	}

	function openEditJournal(id: number) {
		mode = { kind: 'edit-journal', id };
	}

	function openArchiveConfirm(id: number) {
		mode = { kind: 'archive-confirm', id };
	}

	/** Story 28-4b — ouvre le dialogue de clôture, date préremplie à la date LOCALE du jour. */
	function openCloseConfirm(id: number) {
		closeDate = todayLocal();
		closeError = null;
		mode = { kind: 'close-confirm', id };
	}

	function openReopenConfirm(id: number) {
		reopenError = null;
		mode = { kind: 'reopen-confirm', id };
	}

	function closeForm() {
		mode = { kind: 'none' };
		closeError = null;
		reopenError = null;
		resetForm();
	}

	/**
	 * Story 28-4b (C-28-4-6) — vrai si le solde comptable du compte lié est connu
	 * et non nul : le dialogue de clôture **avertit**, il ne bloque pas (une
	 * écriture de virement ou de frais peut suivre la clôture).
	 */
	function hasNonZeroBalance(ba: BankAccountSummary): boolean {
		return ba.currentBalance !== null && !new Big(ba.currentBalance).eq(0);
	}

	/**
	 * Story 28-4b (AC11, C-28-4-46) — un compte archivé ou clos s'affiche atténué.
	 * L'atténuation porte sur les cellules de DONNÉES, non sur la ligne : `opacity`
	 * ne se défait pas dans un enfant, et l'action « Rouvrir » d'un compte clos doit
	 * rester pleinement lisible.
	 */
	function dimClass(ba: BankAccountSummary): string {
		return ba.archived || ba.closedOn !== null ? 'opacity-50' : '';
	}

	/** Story 28-4b (AC11) — pourquoi « Clore » est désactivé sur le principal (`title` et aide). */
	function primaryCloseHint(): string {
		return i18nMsg(
			'bank-accounts-close-primary-hint',
			"Le compte principal ne se clôt pas : désignez d'abord un autre compte principal.",
		);
	}

	/**
	 * Story 26-2a — la description envoyée : `null` si le champ est vide ou fait
	 * d'espaces. Toujours présente dans les deux payloads : au `PUT`, l'omettre
	 * l'effacerait (C-26-2-19). La normalisation fait foi côté serveur.
	 */
	function descriptionPayload(): string | null {
		return formDescription.trim() === '' ? null : formDescription;
	}

	/** Retourne false (et positionne `formError`) si le QR-IBAN saisi n'en est pas un. */
	function validateQrIbanField(): boolean {
		const qrIban = formQrIban.trim();
		if (qrIban !== '' && !qrIbanHasValidQrIid(qrIban)) {
			formError = i18nMsg(
				'bank-accounts-error-qr-iban-not-qr',
				"Cet IBAN n'est pas un QR-IBAN. Si votre banque ne vous a pas fourni de QR-IBAN dédié aux QR-factures, laissez ce champ vide : votre IBAN normal suffit."
			);
			return false;
		}
		return true;
	}

	async function submitCreate(e: Event) {
		e.preventDefault();
		formError = null;
		if (!isCard && !validateQrIbanField()) return;
		if (!validateCardFields()) return;
		if (!validateProviderFields()) return;
		// Story 34-2 (revue P1, B-1), Story 35-6 — avec un compte lié, la saisie est
		// classée avant tout envoi : « 1.234 » ou « 1,234 » (mille ou un franc ?), une
		// saisie illisible ou négative ne partent pas ; une saisie vide n'envoie aucun solde.
		const opening =
			formJournalAccountId === null || ledgerIsForeign ? null : openingAmountKind(formOpeningAmount);
		if (opening !== null && opening.kind !== 'empty') {
			const refusal = openingAmountRefusal(opening.kind);
			if (refusal !== null) {
				formError = refusal;
				return;
			}
		}
		submitting = true;
		try {
			// Story 34-11a2 — une carte n'envoie ni IBAN, ni QR-IBAN, ni
			// « principal » ; un compte bancaire garde le payload de la 0.17.5.
			const provider = formProviderPayload();
			const payload: NewBankAccountPayload = isCard
				? {
						accountKind: 'credit_card',
						bankName: formBankName,
						description: descriptionPayload(),
						cardLast4: cardLast4Payload(formCardLast4),
						journalAccountId: formJournalAccountId,
					}
				: provider !== null
					? {
							// Story 38-11d (AC-1) — compte PayPal (Story 40-7d : ou Stripe) : IBAN
							// facultatif, ni QR-IBAN ni principal.
							bankName: formBankName,
							description: descriptionPayload(),
							iban: providerIban(),
							qrIban: null,
							isPrimary: false,
							journalAccountId: formJournalAccountId,
							...provider,
						}
					: {
						bankName: formBankName,
						description: descriptionPayload(),
							iban: formIban,
							qrIban: formQrIban.trim() === '' ? null : formQrIban,
							isPrimary: formIsPrimary,
							journalAccountId: formJournalAccountId,
						};
			// Story 34-2 (#533, AC-7) — sans compte lié, les champs du solde sont
			// masqués et rien n'est envoyé ; un montant vide n'envoie rien non plus. La
			// charge est celle du classement (Story 35-6), que l'aperçu a montrée.
			if (opening?.kind === 'clear' && opening.payload !== undefined) {
				payload.openingBalance = { amount: opening.payload, side: formOpeningSide };
			}
			const created = await createBankAccount(payload);
			const openingEntry = created?.openingEntry ?? null;
			// Passe de textes de la 0.17.6 (#1040) : une carte a ses propres avis.
			// Clés et arguments littéraux à chaque appel, lus par les gardes i18n.
			toast.success(
				isCard
					? openingEntry
						? i18nMsg(
								'bank-accounts-toast-create-card-success-with-opening',
								'Carte de crédit créée, avec l\'écriture d\'ouverture n° { $number } du { $date } : { $amount }.',
								{
									number: String(openingEntry.entryNumber),
									date: formatSwissDate(openingEntry.entryDate),
									amount: formatChfExact(payload.openingBalance?.amount ?? '0'),
								},
							)
						: i18nMsg('bank-accounts-toast-create-card-success', 'Carte de crédit créée.')
					: openingEntry
						? i18nMsg(
								'bank-accounts-toast-create-success-with-opening',
								'Compte bancaire créé, avec l\'écriture d\'ouverture n° { $number } du { $date } : { $amount }.',
								{
									number: String(openingEntry.entryNumber),
									date: formatSwissDate(openingEntry.entryDate),
									amount: formatChfExact(payload.openingBalance?.amount ?? '0'),
								},
							)
						: i18nMsg('bank-accounts-toast-create-success', 'Compte bancaire créé.'),
			);
			await reload();
			closeForm();
		} catch (err) {
			formError = errorMessageOf(err);
		} finally {
			submitting = false;
		}
	}

	async function submitUpdateFull(e: Event, id: number) {
		e.preventDefault();
		formError = null;
		if (!isCard && !validateQrIbanField()) return;
		if (!validateCardFields()) return;
		if (!validateProviderFields()) return;
		submitting = true;
		try {
			// Story 34-11a2 (points hérités E5/B7) — le `PUT` d'une carte n'envoie
			// ni IBAN ni type ; ses quatre chiffres sont toujours présents (`null`
			// pour les effacer).
			const provider = formProviderPayload();
			const payload: UpdateBankAccountPayload = isCard
				? {
						bankName: formBankName,
						description: descriptionPayload(),
						cardLast4: cardLast4Payload(formCardLast4),
						journalAccountId: formJournalAccountId,
						version: formVersion,
					}
				: provider !== null
					? {
							// Story 38-11d (AC-1) — compte PayPal (Story 40-7d : ou Stripe) :
							// paramètres posés ou remplacés.
							bankName: formBankName,
							description: descriptionPayload(),
							iban: providerIban(),
							qrIban: null,
							isPrimary: false,
							journalAccountId: formJournalAccountId,
							version: formVersion,
							...provider,
						}
					: {
							bankName: formBankName,
							description: descriptionPayload(),
							iban: formIban,
							qrIban: formQrIban.trim() === '' ? null : formQrIban,
							isPrimary: formIsPrimary,
							journalAccountId: formJournalAccountId,
							version: formVersion,
							// Décochée sur un compte qui en portait : retirés ; sinon absents (inchangés).
							...(formWasProvider === 'paypal'
								? { paypal: null }
								: formWasProvider === 'stripe'
									? { stripe: null }
									: {}),
						};
			await updateBankAccount(id, payload);
			toast.success(i18nMsg('bank-accounts-toast-update-success', 'Compte bancaire modifié.'));
			await reload();
			closeForm();
		} catch (err) {
			formError = errorMessageOf(err);
		} finally {
			submitting = false;
		}
	}

	async function confirmArchive(id: number) {
		submitting = true;
		try {
			const ba = bankAccounts.find((b) => b.id === id);
			if (!ba) return;
			await archiveBankAccount(id, ba.version);
			toast.success(i18nMsg('bank-accounts-toast-archive-success', 'Compte bancaire archivé.'));
			// Mode refermé AVANT le rechargement : le compte archivé quitte la liste, et
			// `closeOrphanMode` ne doit pas annoncer un geste réussi comme un changement.
			closeForm();
			await reload();
		} catch (err) {
			toast.error(errorMessageOf(err));
		} finally {
			submitting = false;
		}
	}

	/**
	 * Story 28-4b (#642, AC11) — clôt le compte. Les refus du serveur (`412`
	 * principal, transactions non rapprochées, lot généré ; `409` version) restent
	 * affichés dans le dialogue, par leur message traduit.
	 */
	async function confirmClose(id: number) {
		const ba = bankAccounts.find((b) => b.id === id);
		if (!ba) return;
		submitting = true;
		closeError = null;
		try {
			await closeBankAccount(id, ba.version, closeDate);
			toast.success(
				includeClosed
					? i18nMsg('bank-accounts-toast-close-success', 'Compte bancaire clos.')
					: i18nMsg(
							'bank-accounts-toast-close-success-hidden',
							'Compte clos. Affichez les comptes clos pour le retrouver.',
						),
			);
			closeForm(); // avant le rechargement, comme l'archivage (`closeOrphanMode`)
			await reload();
		} catch (err) {
			closeError = errorMessageOf(err);
			await refreshAfterVersionConflict(err);
		} finally {
			submitting = false;
		}
	}

	/** Story 28-4b (#642, AC11) — rouvre un compte clos ; il ne redevient pas principal. */
	async function confirmReopen(id: number) {
		const ba = bankAccounts.find((b) => b.id === id);
		if (!ba) return;
		submitting = true;
		reopenError = null;
		try {
			await reopenBankAccount(id, ba.version);
			toast.success(i18nMsg('bank-accounts-toast-reopen-success', 'Compte bancaire rouvert.'));
			closeForm(); // avant le rechargement, comme l'archivage (`closeOrphanMode`)
			await reload();
		} catch (err) {
			reopenError = errorMessageOf(err);
			await refreshAfterVersionConflict(err);
		} finally {
			submitting = false;
		}
	}

	/**
	 * Story 28-4b (revue P1, B-2) — après un `409 OPTIMISTIC_LOCK_CONFLICT`, la liste est
	 * relue : le dialogue reste ouvert avec son message, et le second essai part avec la
	 * version relue au lieu de rejouer la périmée. Si le compte n'est plus dans la liste
	 * relue (clos ailleurs, clos masqués ici), `closeOrphanMode` referme le dialogue avec
	 * un message (passe ciblée P2, M1). Les autres refus ne rechargent rien.
	 */
	async function refreshAfterVersionConflict(err: unknown): Promise<void> {
		if (isApiError(err) && err.code === 'OPTIMISTIC_LOCK_CONFLICT') await reload();
	}

	/**
	 * Une bascule (archivés, clos) recharge la liste sous un indicateur de chargement.
	 * Story 28-4b : elle ne referme RIEN elle-même (passe ciblée P2, L1) — un mode dont le
	 * compte disparaît de la liste relue est refermé par `closeOrphanMode`, à la fin de
	 * `reload()` ; un formulaire de création, ou un mode dont le compte reste visible, est
	 * conservé avec sa saisie.
	 *
	 * ⚠️ Angle mort assumé (B-3) : un `reload()` lancé par une clôture ou une réouverture
	 * réussie et celui d'une bascule cliquée pendant ce temps ne sont pas ordonnés — la
	 * dernière réponse arrivée gagne (fenêtre d'un aller-retour réseau ; même schéma que la
	 * bascule des archivés d'avant la 28-4b). Un nouveau clic sur la bascule rétablit la
	 * liste attendue.
	 */
	async function toggleAndReload(flip: () => void) {
		flip();
		loading = true;
		try {
			await reload();
		} finally {
			loading = false;
		}
	}

	async function handleToggleClosed() {
		await toggleAndReload(() => (includeClosed = !includeClosed));
	}

	function handleJournalUpdated(updated: BankAccountSummary) {
		bankAccounts = bankAccounts.map((ba) => (ba.id === updated.id ? updated : ba));
		closeForm();
	}

	async function handleToggleArchived() {
		// F18 Pass 1 code review : indicateur de chargement pendant le reload
		// pour feedback UX cohérent avec onMount.
		await toggleAndReload(() => (includeArchived = !includeArchived));
	}
</script>

<svelte:head>
	<title>{i18nMsg('bank-accounts-labels-page-title', 'Comptes bancaires')} - Kesh</title>
</svelte:head>

<!-- Story 34-2 (#533, AC-7) — les champs du solde d'ouverture, communs au formulaire
     de création et au dialogue de ligne : montant, sens, l'aide qui dit la date et la
     contrepartie, et la conséquence sur l'écran Soldes de départ, avant le geste. -->
{#snippet openingFields(prefix: string)}
	<label for="{prefix}-opening-amount" class="block text-sm font-medium text-text mb-1">
		{prefix === 'form'
			? i18nMsg('bank-accounts-opening-label-amount', "Solde d'ouverture (optionnel)")
			: i18nMsg('bank-accounts-opening-label-amount-row', 'Montant du solde')}
	</label>
	<div class="flex items-center gap-2">
		<Input
			id="{prefix}-opening-amount"
			bind:value={formOpeningAmount}
			inputmode="decimal"
			placeholder="0.00"
			data-testid="{prefix}-opening-amount"
		/>
		<label for="{prefix}-opening-side" class="sr-only">
			{i18nMsg('bank-accounts-opening-label-side', 'Sens du solde')}
		</label>
		<select
			id="{prefix}-opening-side"
			bind:value={formOpeningSide}
			class="rounded border border-border px-3 py-2 text-sm"
			data-testid="{prefix}-opening-side"
		>
			<option value="debit">{i18nMsg('bank-accounts-opening-side-debit', 'Avoir')}</option>
			<option value="credit">
				<!-- Passe de textes de la 0.17.6 (#1040) : « découvert » n'a pas de sens pour une carte. -->
				{openingForCard
					? i18nMsg('bank-accounts-opening-side-credit-card', 'Dette (solde dû)')
					: i18nMsg('bank-accounts-opening-side-credit', 'Dette (découvert)')}
			</option>
		</select>
	</div>
	{#if openingAmountPreview(formOpeningAmount) !== null}
		<!-- Revue P1 (B-1) — le montant tel qu'il sera écrit, avant le geste. Passe de textes
		     de la 0.17.6 (#1027) : un montant nul n'écrit rien, et l'aperçu le dit. -->
		<p class="mt-1 text-sm font-medium text-text" data-testid="{prefix}-opening-preview">
			{#if /^0+(\.0+)?$/.test(openingAmountPayload(formOpeningAmount) ?? '')}
				{i18nMsg(
					'bank-accounts-opening-preview-zero',
					"Montant nul : aucune écriture d'ouverture ne sera passée.",
				)}
			{:else}
				{i18nMsg('bank-accounts-opening-preview', 'Montant qui sera écrit : { $amount }', {
					amount: openingAmountPreview(formOpeningAmount) ?? '',
				})}
			{/if}
		</p>
	{/if}
	<p class="mt-1 text-xs text-text-muted" data-testid="{prefix}-opening-help">
		{i18nMsg(
			'bank-accounts-opening-help',
			"L'écriture est datée du premier jour du premier exercice s'il est ouvert et hors période verrouillée, sinon d'aujourd'hui : saisissez le solde du relevé à cette date. Sa contrepartie va au compte Bénéfice/perte reporté. Elle est refusée si le compte lié a déjà des mouvements.",
		)}
	</p>
	<p class="mt-1 rounded border border-warning bg-warning-soft p-2 text-xs text-text" role="note" data-testid="{prefix}-opening-consequence">
		{i18nMsg(
			'bank-accounts-opening-consequence',
			"Si l'ouverture générale de Soldes de départ est encore proposée, ce solde la retire : les soldes de départ des autres comptes se saisiront un à un, mode Compléter. Pour saisir toute votre ouverture d'un coup, passez d'abord par Soldes de départ.",
		)}
	</p>
{/snippet}

{#snippet providerCheckbox(prefix: string, provider: ConfigurableProvider, primaryLocked: boolean)}
	<!-- Story 40-7d (AC-1) — la case d'un prestataire (« Compte PayPal », « Compte Stripe ») ;
	     cocher l'une décoche l'autre. -->
	<div class="flex items-center gap-2">
		<input
			id="{prefix}-is-{provider}"
			type="checkbox"
			checked={formProvider === provider}
			disabled={primaryLocked}
			onchange={(e) => setFormProvider(provider, (e.currentTarget as HTMLInputElement).checked)}
			data-testid="{prefix}-is-{provider}"
		/>
		<label for="{prefix}-is-{provider}" class="block text-sm font-medium text-text">
			{provider === 'stripe'
				? i18nMsg('bank-accounts-labels-is-stripe', 'Compte Stripe')
				: i18nMsg('bank-accounts-labels-is-paypal', 'Compte PayPal')}
		</label>
	</div>
{/snippet}

{#snippet paypalFields(prefix: string, primaryLocked: boolean)}
	<!-- Story 38-11d (AC-1) — les cases des prestataires et leurs champs, communs aux
	     formulaires de création et de modification ; Story 40-7d (AC-1) — la case
	     « Compte Stripe » et son compte de versement. -->
	<div class="col-span-2">
		<div class="flex flex-wrap items-center gap-6">
			{@render providerCheckbox(prefix, 'paypal', primaryLocked)}
			{@render providerCheckbox(prefix, 'stripe', primaryLocked)}
		</div>
		{#if primaryLocked}
			<p class="mt-1 text-xs text-text-muted" data-testid="{prefix}-is-paypal-locked">
				{i18nMsg(
					'bank-accounts-help-provider-primary',
					"Le compte principal ne peut pas être un compte PayPal ni Stripe : désignez d'abord un autre compte principal.",
				)}
			</p>
		{:else if formProvider === 'stripe'}
			<p class="mt-1 text-xs text-text-muted" data-testid="{prefix}-is-stripe-help">
				{i18nMsg(
					'bank-accounts-help-stripe',
					'Un compte bancaire par compte Stripe et par devise : son rapport Stripe s’importe et se rapproche par la page Stripe, non par l’import bancaire.',
				)}
			</p>
		{:else}
			<p class="mt-1 text-xs text-text-muted" data-testid="{prefix}-is-paypal-help">
				{i18nMsg(
					'bank-accounts-help-paypal',
					'Un compte bancaire par compte PayPal et par devise : son relevé PayPal s’importe et se rapproche par la page PayPal, non par l’import bancaire.',
				)}
			</p>
		{/if}
	</div>
	{#if isProviderForm && formProvider !== null}
		<div>
			<label for="{prefix}-{formProvider}-account" class="block text-sm font-medium text-text mb-1">
				{formProvider === 'stripe'
					? i18nMsg('bank-accounts-labels-stripe-account', 'Compte Stripe')
					: i18nMsg('bank-accounts-labels-paypal-account', 'Compte PayPal')}
			</label>
			<Input
				id="{prefix}-{formProvider}-account"
				bind:value={formProviderAccount}
				maxlength={PROVIDER_ACCOUNT_MAX_CHARS}
				list="{prefix}-{formProvider}-labels"
				placeholder={formProvider === 'stripe' ? 'Stripe boutique' : 'PayPal www'}
				data-testid="{prefix}-{formProvider}-account"
			/>
			<datalist id="{prefix}-{formProvider}-labels" data-testid="{prefix}-{formProvider}-labels">
				{#each providerLabels as label (label)}
					<option value={label}></option>
				{/each}
			</datalist>
			<p class="mt-1 text-xs text-text-muted">
				{formProvider === 'stripe'
					? i18nMsg(
							'bank-accounts-help-stripe-account',
							'Le même libellé pour les comptes de chaque devise d’un même compte Stripe.',
						)
					: i18nMsg(
							'bank-accounts-help-paypal-account',
							'Le même libellé pour les comptes de chaque devise d’un même compte PayPal.',
						)}
			</p>
		</div>
		<div>
			<label for="{prefix}-fee-account" class="block text-sm font-medium text-text mb-1">
				{i18nMsg('bank-accounts-labels-fee-account', 'Compte de frais')}
			</label>
			<select
				id="{prefix}-fee-account"
				bind:value={formFeeAccountId}
				class="w-full rounded border border-border px-3 py-2 text-sm"
				data-testid="{prefix}-fee-account"
			>
				<option value={null}>{i18nMsg('bank-accounts-labels-not-configured', 'Non configuré')}</option>
				{#each withCurrentAccount(feeCandidates, formFeeAccountId, accounts) as acc (acc.id)}
					<option value={acc.id}>{acc.number} — {acc.name}</option>
				{/each}
			</select>
			<p class="mt-1 text-xs text-text-muted">
				{i18nMsg(
					'bank-accounts-help-fee-account',
					'Compte de charge, en CHF ou dans la devise du compte du plan. Sans compte de frais, les frais vont « à examiner ».',
				)}
			</p>
		</div>
		{#if isStripeForm}
			<div>
				<label for="{prefix}-payout-account" class="block text-sm font-medium text-text mb-1">
					{i18nMsg('bank-accounts-labels-payout-account', 'Compte bancaire de versement')}
				</label>
				<select
					id="{prefix}-payout-account"
					bind:value={formPayoutBankAccountId}
					class="w-full rounded border border-border px-3 py-2 text-sm"
					data-testid="{prefix}-payout-account"
				>
					<option value={null}>{i18nMsg('bank-accounts-labels-not-configured', 'Non configuré')}</option>
					{#each payoutCandidates as b (b.id)}
						<option value={b.id}
							>{bankAccountName(b)}{#if b.archived}
								({i18nMsg('bank-accounts-labels-archived-badge', 'Archivé')}){:else if b.closedOn !== null}
								({i18nMsg('bank-accounts-labels-closed-badge', 'Clos le { $date }', {
									date: formatSwissDate(b.closedOn),
								})}){/if}</option
						>
					{/each}
				</select>
				<p class="mt-1 text-xs text-text-muted">
					{i18nMsg(
						'bank-accounts-help-payout-account',
						'Le compte bancaire où Stripe verse, dans la devise du compte du plan (une banque ou Wise, jamais un compte Stripe ni PayPal). Sans compte de versement, les versements vont « à examiner ».',
					)}
				</p>
			</div>
		{/if}
	{/if}
{/snippet}

<div class="flex items-center justify-between">
	<div>
		<h1 class="text-2xl font-semibold text-text" data-testid="bank-accounts-page-title">
			{i18nMsg('bank-accounts-labels-page-title', 'Comptes bancaires')}
		</h1>
		<p class="mt-2 text-sm text-text-muted">
			{i18nMsg(
				'bank-accounts-labels-page-subtitle',
				'Lier chaque compte bancaire à un compte du plan comptable (classe 1 typique : 1010 Poste, 1020 Banque).',
			)}
		</p>
	</div>
	<div class="flex items-center gap-3">
		<Button
			variant="ghost"
			onclick={handleToggleArchived}
			data-testid="toggle-archived"
		>
			{includeArchived
				? i18nMsg('bank-accounts-actions-hide-archived', 'Masquer les archivés')
				: i18nMsg('bank-accounts-actions-show-archived', 'Afficher les archivés')}
		</Button>
		<Button
			variant="ghost"
			onclick={handleToggleClosed}
			data-testid="toggle-closed"
		>
			{includeClosed
				? i18nMsg('bank-accounts-actions-hide-closed', 'Masquer les comptes clos')
				: i18nMsg('bank-accounts-actions-show-closed', 'Afficher les comptes clos')}
		</Button>
		<Button onclick={openCreate} data-testid="create-bank-account-button" disabled={mode.kind === 'create'}>
			{i18nMsg('bank-accounts-actions-create', 'Nouveau compte bancaire')}
		</Button>
	</div>
</div>

<div class="mt-6">
	{#if loading}
		<p class="text-text-muted">
			{i18nMsg('bank-accounts-labels-loading', 'Chargement…')}
		</p>
	{:else if loadError}
		<p class="text-red-600" role="alert" data-testid="load-error">{loadError}</p>
	{:else}
		{#if staleNotice}
			<!-- Story 28-4b (passe ciblée P2, M1) — un mode a été refermé par `closeOrphanMode`. -->
			<p class="mb-4 rounded border border-warning bg-warning-soft p-2 text-sm text-text" role="status" data-testid="bank-accounts-stale-notice">
				{i18nMsg(
					'bank-accounts-notice-stale',
					"Le compte visé n'est plus affiché : il a été modifié entre-temps, ou il est désormais masqué. L'action en cours a été refermée.",
				)}
			</p>
		{/if}
		<!-- Formulaire création inline -->
		{#if mode.kind === 'create'}
			<form
				onsubmit={submitCreate}
				class="mb-6 rounded border border-border bg-surface-alt p-4"
				data-testid="create-bank-account-form"
			>
				<h2 class="mb-3 text-base font-semibold" data-testid="create-bank-account-title">
					<!-- Passe de textes de la 0.17.6 (#1037) : le titre suit le type choisi. -->
					{isCard
						? i18nMsg('bank-accounts-form-title-create-card', 'Nouvelle carte de crédit')
						: i18nMsg('bank-accounts-actions-create', 'Nouveau compte bancaire')}
				</h2>
				<!-- Story 34-11a2 (#972) — le type se choisit à la création, et seulement là. -->
				<fieldset class="mb-3" data-testid="form-kind">
					<legend class="block text-sm font-medium text-text mb-1">{i18nMsg('bank-accounts-labels-kind', 'Type de compte')}</legend>
					<div class="flex gap-4 text-sm">
						<label class="flex items-center gap-1">
							<input type="radio" name="form-kind" value="bank" checked={formKind === 'bank'} onchange={() => setFormKind('bank')} data-testid="form-kind-bank" />
							{i18nMsg('bank-accounts-kind-bank', 'Compte bancaire')}
						</label>
						<label class="flex items-center gap-1">
							<input type="radio" name="form-kind" value="credit_card" checked={formKind === 'credit_card'} onchange={() => setFormKind('credit_card')} data-testid="form-kind-credit-card" />
							{i18nMsg('bank-accounts-kind-credit-card', 'Carte de crédit')}
						</label>
					</div>
				</fieldset>
				<div class="grid grid-cols-2 gap-3">
					<div>
						<label for="form-bank-name" class="block text-sm font-medium text-text mb-1">{isCard ? i18nMsg('bank-accounts-labels-card-issuer', 'Émetteur de la carte') : i18nMsg('bank-accounts-labels-bank-name', 'Banque')}</label>
						<Input id="form-bank-name" bind:value={formBankName} required data-testid="form-bank-name" />
					</div>
					<div>
						<label for="form-description" class="block text-sm font-medium text-text mb-1">{i18nMsg('bank-accounts-labels-description', 'Description (optionnelle)')}</label>
						<Input id="form-description" bind:value={formDescription} maxlength={BANK_ACCOUNT_DESCRIPTION_MAX} data-testid="form-description" />
						<p class="mt-1 text-xs text-text-muted" data-testid="form-description-help">{i18nMsg('bank-accounts-help-description', 'Par exemple « Personnel » ou « Épargne » : pour distinguer deux comptes de la même banque. 60 caractères au plus.')}</p>
					</div>
					{#if isCard}
						<div>
							<label for="form-card-last4" class="block text-sm font-medium text-text mb-1">{i18nMsg('bank-accounts-labels-card-last4', 'Quatre derniers chiffres (facultatif)')}</label>
							<!-- Revue P1 (E5) : pas de `maxlength` — un numéro complet collé ici serait
							     tronqué à ses quatre PREMIERS chiffres sans refus ; il est refusé à l'envoi. -->
							<Input id="form-card-last4" bind:value={formCardLast4} inputmode="numeric" autocomplete="off" placeholder="1234" data-testid="form-card-last4" />
							<p class="mt-1 text-xs text-text-muted" data-testid="form-card-last4-help">{i18nMsg('bank-accounts-help-card-last4', 'Kesh ne garde jamais le numéro complet de la carte : seulement ses quatre derniers chiffres, pour la reconnaître.')}</p>
						</div>
					{:else}
						{@render paypalFields('form', false)}
						<div>
							<label for="form-iban" class="block text-sm font-medium text-text mb-1">{isProviderForm ? i18nMsg('bank-accounts-labels-iban-optional', 'IBAN (facultatif)') : i18nMsg('bank-accounts-labels-iban', 'IBAN')}</label>
							<Input id="form-iban" bind:value={formIban} required={!isProviderForm} placeholder="CH..." data-testid="form-iban" />
						</div>
						{#if !isProviderForm}
						<div>
							<label for="form-qr-iban" class="block text-sm font-medium text-text mb-1">{i18nMsg('bank-accounts-labels-qr-iban', 'QR-IBAN (optionnel)')}</label>
							<Input id="form-qr-iban" bind:value={formQrIban} placeholder="CH..." data-testid="form-qr-iban" />
							<p class="mt-1 text-xs text-text-muted" data-testid="form-qr-iban-help">{i18nMsg('bank-accounts-help-qr-iban', 'À remplir uniquement si votre banque vous a fourni un QR-IBAN dédié aux QR-factures (numéro spécial avec un identifiant 30000–31999). Sinon, laissez ce champ vide : votre IBAN normal suffit pour générer des QR-factures.')}</p>
						</div>
						{/if}
					{/if}
					<div>
						<label for="form-journal-account" class="block text-sm font-medium text-text mb-1">{isCard ? i18nMsg('bank-accounts-labels-card-ledger', 'Compte de dette lié') : i18nMsg('bank-accounts-labels-journal-account-id', 'Compte comptable lié')}</label>
						<select
							id="form-journal-account"
							bind:value={formJournalAccountId}
							class="w-full rounded border border-border px-3 py-2 text-sm"
							data-testid="form-journal-account"
						>
							<option value={null}>{i18nMsg('bank-accounts-labels-not-configured', 'Non configuré')}</option>
							{#each withCurrentAccount(linkableAccounts, formJournalAccountId, accounts) as acc}
								<option value={acc.id}>{acc.number} — {acc.name}</option>
							{/each}
						</select>
						<p class="mt-1 text-xs text-text-muted" data-testid="form-journal-account-help">
							{#if isCard}
								{i18nMsg(
									'bank-accounts-help-card-ledger',
									'Obligatoire : un compte de passif, la dette de la carte (par exemple un sous-compte de 2100 Dettes bancaires à court terme).',
								)}
							{:else if isStripeForm}
								{i18nMsg(
									'bank-accounts-help-stripe-ledger',
									'Obligatoire : le compte du plan du solde Stripe, dans sa devise (un compte en devise exige le multi-devises).',
								)}
							{:else if isProviderForm}
								{i18nMsg(
									'bank-accounts-help-paypal-ledger',
									'Obligatoire : le compte du plan du solde PayPal, dans sa devise (un compte en devise exige le multi-devises).',
								)}
							{:else}
								{i18nMsg(
									'bank-accounts-tooltip-journal-account',
									'Lie ce compte bancaire à un compte du plan comptable (typiquement 1010 Poste, 1020 Banque). Permet à la réconciliation automatique de créer les écritures vers le bon compte, et l\'affichage du solde sur la page d\'accueil. Note multi-comptes : si plusieurs comptes courants distincts, lier au sous-compte spécifique (1020.001 BCV CHF), pas au parent 1020.',
								)}
							{/if}
						</p>
					</div>
					{#if formJournalAccountId !== null && !ledgerIsForeign}
						<!-- Story 34-2 (#533, AC-7) — le solde d'ouverture, actif dès qu'un compte
						     du plan est lié ; masqué (et non envoyé) sans compte lié. -->
						<div data-testid="form-opening-balance">
							{@render openingFields('form')}
						</div>
					{/if}
					{#if !isCard && !isProviderForm}
						<div class="col-span-2 flex items-center gap-2">
							<input
								id="form-is-primary"
								type="checkbox"
								bind:checked={formIsPrimary}
								data-testid="form-is-primary"
							/>
							<label for="form-is-primary" class="block text-sm font-medium text-text mb-1">{i18nMsg('bank-accounts-labels-is-primary', 'Compte principal')}</label>
						</div>
					{/if}
				</div>
				{#if formError}
					<p class="mt-3 text-sm text-red-600" role="alert" data-testid="form-error">{formError}</p>
				{/if}
				<div class="mt-4 flex gap-2">
					<Button type="submit" disabled={submitting} data-testid="form-submit">
						{i18nMsg('bank-accounts-actions-submit-create', 'Créer')}
					</Button>
					<Button type="button" variant="ghost" onclick={closeForm} data-testid="form-cancel">
						{i18nMsg('bank-accounts-actions-cancel', 'Annuler')}
					</Button>
				</div>
			</form>
		{/if}

		<!-- Liste -->
		{#if bankAccounts.length === 0}
			<!-- Story 28-4b (revue P1, A-L6) — une bascule au moins masque des comptes : le
			     message dit où les trouver (une société dont les seuls comptes restants sont
			     clos voyait « Aucun compte bancaire configuré » sur l'écran de gestion). -->
			<p class="text-text-muted" data-testid="bank-accounts-empty">
				<!-- Passe ciblée P2 (L2) : le message ne nomme que ce qui reste masqué. -->
				{#if includeClosed && includeArchived}
					{i18nMsg('bank-accounts-labels-empty', 'Aucun compte bancaire configuré.')}
				{:else if includeClosed}
					{i18nMsg(
						'bank-accounts-labels-empty-hidden-archived',
						'Aucun compte bancaire à afficher. Les comptes archivés éventuels sont masqués : affichez-les par « Afficher les archivés ».',
					)}
				{:else if includeArchived}
					{i18nMsg(
						'bank-accounts-labels-empty-hidden-closed',
						'Aucun compte bancaire à afficher. Les comptes clos éventuels sont masqués : affichez-les par « Afficher les comptes clos ».',
					)}
				{:else}
					{i18nMsg(
						'bank-accounts-labels-empty-hidden',
						'Aucun compte bancaire ouvert. Les comptes clos ou archivés éventuels sont masqués : affichez-les par les boutons ci-dessus.',
					)}
				{/if}
			</p>
		{:else}
			<table class="w-full text-sm" data-testid="bank-accounts-list">
				<thead>
					<tr class="border-b border-border text-left">
						<th class="py-2 pr-4 font-semibold">{i18nMsg('bank-accounts-labels-bank-name', 'Banque')}</th>
						<th class="py-2 pr-4 font-semibold">{i18nMsg('bank-accounts-labels-iban', 'IBAN')}</th>
						<th class="py-2 pr-4 font-semibold text-right">{i18nMsg('bank-accounts-labels-balance', 'Solde comptable')}</th>
						<th class="py-2 pr-4 font-semibold">{i18nMsg('bank-accounts-labels-journal-account-id', 'Compte comptable lié')}</th>
						<th class="py-2"></th>
					</tr>
				</thead>
				<tbody>
					{#each bankAccounts as ba (ba.id)}
						<tr class="border-b border-border" data-testid="bank-account-row-{ba.id}">
							<td class="py-2 pr-4 {dimClass(ba)}">
								<span data-testid="bank-account-name-{ba.id}">{bankAccountName(ba)}</span>
								{#if providerSummaryLine(ba, accounts) !== null}
									<!-- Story 38-11d (AC-1) — « PayPal · <compte> · <devise> » ; Story 40-7d, « Stripe · … ». -->
									<span class="block text-xs text-text-muted" data-testid="bank-account-{ba.provider}-{ba.id}">{providerSummaryLine(ba, accounts)}</span>
								{/if}
								{#if ba.isPrimary}
									<span class="ml-2 rounded bg-primary-light/20 px-1.5 py-0.5 text-xs text-primary" data-testid="primary-badge-{ba.id}">
										{i18nMsg('bank-accounts-labels-primary-badge', 'Principal')}
									</span>
								{/if}
								{#if ba.archived}
									<span class="ml-2 rounded bg-gray-200 px-1.5 py-0.5 text-xs text-gray-700" data-testid="archived-badge-{ba.id}">
										{i18nMsg('bank-accounts-labels-archived-badge', 'Archivé')}
									</span>
								{/if}
								{#if ba.closedOn !== null}
									<span class="ml-2 rounded bg-gray-200 px-1.5 py-0.5 text-xs text-gray-700" data-testid="closed-badge-{ba.id}">
										{i18nMsg('bank-accounts-labels-closed-badge', 'Clos le { $date }', {
											date: formatSwissDate(ba.closedOn),
										})}
									</span>
								{/if}
							</td>
							<!-- Story 34-11a1 (#972) — IBAN, « •••• 1234 » d'une carte, ou vide. -->
							<td class="py-2 pr-4 font-mono text-xs {dimClass(ba)}" title={bankAccountFullId(ba) ?? ''} data-testid="iban-cell-{ba.id}">{bankAccountFullId(ba) ?? ''}</td>
							<td class="py-2 pr-4 text-right font-mono {dimClass(ba)}" data-testid="balance-cell-{ba.id}">
								{formatBalance(ba.currentBalance)}
							</td>
							<td class="py-2 pr-4 {dimClass(ba)}" data-testid="journal-account-cell-{ba.id}">
								{accountLabel(ba.journalAccountId)}
							</td>
							<td class="py-2 whitespace-nowrap">
								{#if !ba.archived && mode.kind === 'none' && ba.closedOn !== null}
									<!-- Story 28-4b — un compte clos n'offre qu'une action : le rouvrir. -->
									<button
										type="button"
										class="rounded border border-border px-2 py-1 text-xs"
										onclick={() => openReopenConfirm(ba.id)}
										data-testid="reopen-{ba.id}"
									>
										{i18nMsg('bank-accounts-actions-reopen', 'Rouvrir')}
									</button>
								{:else if !ba.archived && mode.kind === 'none'}
									{#if !isProvider(ba)}
										<!-- Story 38-11d (AC-1) — pas de « Lier » sur un compte de prestataire (PayPal ;
										     Stripe, 40-7d) : le serveur refuse tout `PATCH` d'un tel compte ; son compte du
										     plan se change par « Modifier ». -->
										<button
											type="button"
											class="mr-1 rounded border border-border px-2 py-1 text-xs"
											onclick={() => openEditJournal(ba.id)}
											data-testid="link-button-{ba.id}"
										>
											{i18nMsg('bank-accounts-actions-link-account', 'Lier')}
										</button>
									{/if}
									<button
										type="button"
										class="rounded border border-border px-2 py-1 text-xs"
										onclick={() => openEditFull(ba)}
										data-testid="edit-button-{ba.id}"
									>
										{i18nMsg('bank-accounts-actions-edit', 'Modifier')}
									</button>
									{#if ba.journalAccountId !== null && ba.lastTransactionDate === null && !isForeignLedger(ba.journalAccountId)}
										<!-- Story 34-2 (#533, AC-7) — compte ouvert, lié, et dont le compte du
										     plan n'a encore aucune ligne (revue P1, E-5 : `lastTransactionDate`
										     est la date de sa dernière ligne). Le serveur reste l'autorité. -->
										<button
											type="button"
											class="ml-1 rounded border border-border px-2 py-1 text-xs"
											onclick={() => openOpeningBalance(ba)}
											data-testid="opening-balance-button-{ba.id}"
										>
											{i18nMsg('bank-accounts-actions-opening-balance', "Saisir le solde d'ouverture")}
										</button>
									{/if}
									<button
										type="button"
										class="ml-1 rounded border border-border px-2 py-1 text-xs text-red-600"
										onclick={() => openArchiveConfirm(ba.id)}
										data-testid="archive-button-{ba.id}"
									>
										{i18nMsg('bank-accounts-actions-archive', 'Archiver')}
									</button>
									<!-- Story 28-4b — « Clore » ouvre un dialogue ; désactivé sur le
									     principal (le serveur le refuse de toute façon, 412). -->
									<button
										type="button"
										class="ml-1 rounded border border-border px-2 py-1 text-xs disabled:cursor-not-allowed disabled:opacity-50"
										onclick={() => openCloseConfirm(ba.id)}
										disabled={ba.isPrimary}
										title={ba.isPrimary ? primaryCloseHint() : undefined}
										data-testid="close-{ba.id}"
									>
										{i18nMsg('bank-accounts-actions-close', 'Clore')}
									</button>
									{#if ba.isPrimary}
										<p class="mt-1 text-xs text-text-muted whitespace-normal" data-testid="close-primary-hint-{ba.id}">
											{primaryCloseHint()}
										</p>
									{/if}
								{/if}
							</td>
						</tr>
						<!-- Form journal_account_id inline (legacy 8-5a-zero) -->
						{#if mode.kind === 'edit-journal' && mode.id === ba.id}
							<tr>
								<td colspan="5" class="py-3">
									<BankAccountJournalLinkForm
										bankAccount={ba}
										accounts={accounts}
										{claimAccountIds}
										onSuccess={handleJournalUpdated}
										onCancel={closeForm}
									/>
								</td>
							</tr>
						{/if}
						<!-- Form édition complète inline -->
						{#if mode.kind === 'edit-full' && mode.id === ba.id}
							<tr>
								<td colspan="5" class="py-3">
									<form
										onsubmit={(e) => submitUpdateFull(e, ba.id)}
										class="rounded border border-border bg-surface-alt p-4"
										data-testid="edit-bank-account-form"
									>
										<h2 class="mb-3 text-base font-semibold">
											{i18nMsg('bank-accounts-actions-edit', 'Modifier')}
										</h2>
										<!-- Story 34-11a2 (#972) — le type est affiché, non modifiable. -->
										<p class="mb-3 text-sm" data-testid="edit-kind">
											<span class="font-medium">{i18nMsg('bank-accounts-labels-kind', 'Type de compte')} :</span>
											{kindLabel(formKind)}
											<span class="ml-2 text-xs text-text-muted">{i18nMsg('bank-accounts-help-kind-fixed', "Le type d'un compte ne se change pas après sa création.")}</span>
										</p>
										<div class="grid grid-cols-2 gap-3">
											<div>
												<label for="edit-bank-name" class="block text-sm font-medium text-text mb-1">{isCard ? i18nMsg('bank-accounts-labels-card-issuer', 'Émetteur de la carte') : i18nMsg('bank-accounts-labels-bank-name', 'Banque')}</label>
												<Input id="edit-bank-name" bind:value={formBankName} required data-testid="edit-bank-name" />
											</div>
											<div>
												<label for="edit-description" class="block text-sm font-medium text-text mb-1">{i18nMsg('bank-accounts-labels-description', 'Description (optionnelle)')}</label>
												<Input id="edit-description" bind:value={formDescription} maxlength={BANK_ACCOUNT_DESCRIPTION_MAX} data-testid="edit-description" />
												<p class="mt-1 text-xs text-text-muted" data-testid="edit-description-help">{i18nMsg('bank-accounts-help-description', 'Par exemple « Personnel » ou « Épargne » : pour distinguer deux comptes de la même banque. 60 caractères au plus.')}</p>
											</div>
											{#if isCard}
												<div>
													<label for="edit-card-last4" class="block text-sm font-medium text-text mb-1">{i18nMsg('bank-accounts-labels-card-last4', 'Quatre derniers chiffres (facultatif)')}</label>
													<Input id="edit-card-last4" bind:value={formCardLast4} inputmode="numeric" autocomplete="off" data-testid="edit-card-last4" />
													<p class="mt-1 text-xs text-text-muted" data-testid="edit-card-last4-help">{i18nMsg('bank-accounts-help-card-last4', 'Kesh ne garde jamais le numéro complet de la carte : seulement ses quatre derniers chiffres, pour la reconnaître.')}</p>
												</div>
											{:else}
												{@render paypalFields('edit', ba.isPrimary)}
												<div>
													<label for="edit-iban" class="block text-sm font-medium text-text mb-1">{isProviderForm ? i18nMsg('bank-accounts-labels-iban-optional', 'IBAN (facultatif)') : i18nMsg('bank-accounts-labels-iban', 'IBAN')}</label>
													<Input id="edit-iban" bind:value={formIban} required={!isProviderForm} data-testid="edit-iban" />
												</div>
												{#if !isProviderForm}
												<div>
													<label for="edit-qr-iban" class="block text-sm font-medium text-text mb-1">{i18nMsg('bank-accounts-labels-qr-iban', 'QR-IBAN (optionnel)')}</label>
													<Input id="edit-qr-iban" bind:value={formQrIban} data-testid="edit-qr-iban" />
													<p class="mt-1 text-xs text-text-muted" data-testid="edit-qr-iban-help">{i18nMsg('bank-accounts-help-qr-iban', 'À remplir uniquement si votre banque vous a fourni un QR-IBAN dédié aux QR-factures (numéro spécial avec un identifiant 30000–31999). Sinon, laissez ce champ vide : votre IBAN normal suffit pour générer des QR-factures.')}</p>
												</div>
												{/if}
											{/if}
											<div>
												<label for="edit-journal-account" class="block text-sm font-medium text-text mb-1">{isCard ? i18nMsg('bank-accounts-labels-card-ledger', 'Compte de dette lié') : i18nMsg('bank-accounts-labels-journal-account-id', 'Compte comptable lié')}</label>
												<select
													id="edit-journal-account"
													bind:value={formJournalAccountId}
													class="w-full rounded border border-border px-3 py-2 text-sm"
													data-testid="edit-journal-account"
												>
													<option value={null}>{i18nMsg('bank-accounts-labels-not-configured', 'Non configuré')}</option>
													{#each withCurrentAccount(linkableAccounts, formJournalAccountId, accounts) as acc}
														<option value={acc.id}>{acc.number} — {acc.name}</option>
													{/each}
												</select>
											</div>
											{#if !isCard && !isProviderForm}
												<div class="col-span-2 flex items-center gap-2">
													<input
														id="edit-is-primary"
														type="checkbox"
														bind:checked={formIsPrimary}
														data-testid="edit-is-primary"
													/>
													<label for="edit-is-primary" class="block text-sm font-medium text-text mb-1">{i18nMsg('bank-accounts-labels-is-primary', 'Compte principal')}</label>
												</div>
											{/if}
										</div>
										{#if formError}
											<p class="mt-3 text-sm text-red-600" role="alert" data-testid="edit-form-error">{formError}</p>
										{/if}
										<div class="mt-4 flex gap-2">
											<Button type="submit" disabled={submitting} data-testid="edit-form-submit">
												{i18nMsg('bank-accounts-actions-submit-update', 'Enregistrer')}
											</Button>
											<Button type="button" variant="ghost" onclick={closeForm} data-testid="edit-form-cancel">
												{i18nMsg('bank-accounts-actions-cancel', 'Annuler')}
											</Button>
										</div>
									</form>
								</td>
							</tr>
						{/if}
						<!-- Confirm dialog archive inline -->
						{#if mode.kind === 'archive-confirm' && mode.id === ba.id}
							<tr>
								<td colspan="5" class="py-3">
									<div class="rounded border border-red-200 bg-red-50 p-4" data-testid="archive-confirm">
										<p class="text-sm">
											{i18nMsg(
												'bank-accounts-confirm-archive',
												'Confirmer l\'archivage de ce compte bancaire ? Cette action est irréversible v0.1.',
											)}
										</p>
										<div class="mt-3 flex gap-2">
											<Button
												onclick={() => confirmArchive(ba.id)}
												disabled={submitting}
												data-testid="archive-confirm-button"
												variant="destructive"
											>
												{i18nMsg('bank-accounts-actions-confirm-archive', 'Archiver')}
											</Button>
											<Button type="button" variant="ghost" onclick={closeForm} data-testid="archive-cancel-button">
												{i18nMsg('bank-accounts-actions-cancel', 'Annuler')}
											</Button>
										</div>
									</div>
								</td>
							</tr>
						{/if}
						<!-- Story 28-4b (#642, AC11) — dialogue de clôture inline -->
						{#if mode.kind === 'close-confirm' && mode.id === ba.id}
							<tr>
								<td colspan="5" class="py-3">
									<div class="rounded border border-border bg-surface-alt p-4" data-testid="close-confirm">
										<h2 class="mb-2 text-base font-semibold">
											{i18nMsg('bank-accounts-close-dialog-title', 'Clore ce compte bancaire ?')}
										</h2>
										<p class="text-sm">
											{i18nMsg(
												'bank-accounts-close-dialog-body',
												"Le compte passe en lecture seule : plus aucun relevé, importé ou saisi, ni rapprochement, règlement ou lot de paiement ne s'y enregistre. Son historique reste entier et consultable (relevés, écritures, rapports, export). Vous pourrez le rouvrir.",
											)}
										</p>
										<div class="mt-3 max-w-xs">
											<label for="close-date" class="block text-sm font-medium text-text mb-1">
												{i18nMsg('bank-accounts-close-dialog-date', 'Date de clôture')}
											</label>
											<Input id="close-date" type="date" bind:value={closeDate} data-testid="close-date" />
										</div>
										{#if hasNonZeroBalance(ba) && ba.currentBalance !== null}
											<p class="mt-3 rounded border border-warning bg-warning-soft p-2 text-sm text-text" role="note" data-testid="close-balance-warning">
												{i18nMsg(
													'bank-accounts-close-dialog-balance-warning',
													'Le solde comptable du compte lié est de { $amount } : vérifiez-le avant de clore (écriture de virement ou de frais).',
													{ amount: formatChfBalance(ba.currentBalance) },
												)}
											</p>
										{/if}
										{#if closeError}
											<p class="mt-3 text-sm text-red-600" role="alert" data-testid="close-error">{closeError}</p>
										{/if}
										<div class="mt-4 flex gap-2">
											<Button
												onclick={() => confirmClose(ba.id)}
												disabled={submitting || closeDate === ''}
												data-testid="close-confirm-button"
												variant="destructive"
											>
												{i18nMsg('bank-accounts-close-dialog-confirm', 'Clore le compte')}
											</Button>
											<Button type="button" variant="outline" onclick={closeForm} data-testid="close-cancel-button">
												{i18nMsg('bank-accounts-actions-cancel', 'Annuler')}
											</Button>
										</div>
									</div>
								</td>
							</tr>
						{/if}
						<!-- Story 34-2 (#533, AC-7) — dialogue du solde d'ouverture inline -->
						{#if mode.kind === 'opening-balance' && mode.id === ba.id}
							<tr>
								<td colspan="5" class="py-3">
									<div class="rounded border border-border bg-surface-alt p-4" data-testid="opening-balance-dialog">
										<h2 class="mb-2 text-base font-semibold">
											{i18nMsg('bank-accounts-opening-dialog-title', "Solde d'ouverture — { $account }", {
												account: bankAccountName(ba),
											})}
										</h2>
										<div class="max-w-xl">
											{@render openingFields('row')}
										</div>
										{#if openingError}
											<p class="mt-3 text-sm text-red-600" role="alert" data-testid="opening-error">{openingError}</p>
										{/if}
										<div class="mt-4 flex gap-2">
											<Button
												onclick={() => confirmOpeningBalance(ba.id)}
												disabled={submitting}
												data-testid="opening-confirm-button"
											>
												{i18nMsg('bank-accounts-opening-dialog-confirm', "Passer l'écriture")}
											</Button>
											<Button type="button" variant="outline" onclick={closeForm} data-testid="opening-cancel-button">
												{i18nMsg('bank-accounts-actions-cancel', 'Annuler')}
											</Button>
										</div>
									</div>
								</td>
							</tr>
						{/if}
						<!-- Story 28-4b (#642, AC11) — dialogue de réouverture inline -->
						{#if mode.kind === 'reopen-confirm' && mode.id === ba.id}
							<tr>
								<td colspan="5" class="py-3">
									<div class="rounded border border-border bg-surface-alt p-4" data-testid="reopen-confirm">
										<p class="text-sm">
											{i18nMsg(
												'bank-accounts-reopen-dialog-body',
												"Rouvrir ce compte bancaire ? Il redevient utilisable pour les imports, les rapprochements et les règlements ; il ne redevient pas compte principal.",
											)}
										</p>
										{#if reopenError}
											<p class="mt-3 text-sm text-red-600" role="alert" data-testid="reopen-error">{reopenError}</p>
										{/if}
										<div class="mt-3 flex gap-2">
											<Button onclick={() => confirmReopen(ba.id)} disabled={submitting} data-testid="reopen-confirm-button">
												{i18nMsg('bank-accounts-actions-reopen', 'Rouvrir')}
											</Button>
											<Button type="button" variant="outline" onclick={closeForm} data-testid="reopen-cancel-button">
												{i18nMsg('bank-accounts-actions-cancel', 'Annuler')}
											</Button>
										</div>
									</div>
								</td>
							</tr>
						{/if}
					{/each}
				</tbody>
			</table>
		{/if}
	{/if}
</div>
