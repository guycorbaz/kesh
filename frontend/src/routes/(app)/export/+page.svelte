<script lang="ts">
	// Story 9-2b — Page export global ZIP (souveraineté des données).
	//
	// Bouton unique « Lancer l'export » qui déclenche un GET
	// `/api/v1/exports/global.zip` (16 CSV + metadata.json packagés en ZIP).
	// Flag dédié `exporting` (Pass 1 ECH-H2 / AC #25), guard re-entrancy
	// first-line (Pass 1 code-review M12 / AC #26), fallback i18n
	// `export-global-error-generic` sur erreur non-`isApiError` (Pass 1 M13 /
	// AC #27).

	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import { isApiError } from '$lib/shared/utils/api-client';
	import { downloadGlobalExport } from '$lib/features/export/exports.api';

	let exporting = $state(false);
	let errorMsg = $state<string | null>(null);
	let successMsg = $state<string | null>(null);

	/**
	 * Formate l'erreur en string i18n.
	 *
	 * Pass 1 BH-MEDIUM-03 / ECH-LOW-04 : duplication locale (5 lignes
	 * identiques à `reports/+page.svelte:75`) — refactor vers
	 * `lib/shared/utils/error-format.ts` reporté Epic 15 v0.2 si > 2 pages.
	 */
	function formatError(err: unknown): string {
		if (isApiError(err)) {
			return err.message;
		}
		return i18nMsg(
			'export-global-error-generic',
			"Impossible de générer l'export global. Vérifiez votre connexion et réessayez.",
		);
	}

	async function startExport(): Promise<void> {
		// Pass 1 code-review M12 / AC #26 — guard first-line.
		if (exporting) return;

		exporting = true;
		errorMsg = null;
		successMsg = null;

		try {
			await downloadGlobalExport();
			successMsg = i18nMsg('export-global-success', 'Export téléchargé.');
		} catch (e) {
			errorMsg = formatError(e);
		} finally {
			exporting = false;
		}
	}
</script>

<svelte:head>
	<title>{i18nMsg('export-global-title', 'Export global de vos données')} – Kesh</title>
</svelte:head>

<section class="mx-auto max-w-3xl space-y-6 p-6">
	<header class="space-y-2">
		<h1 class="text-2xl font-bold">
			{i18nMsg('export-global-title', 'Export global de vos données')}
		</h1>
		<p class="text-sm text-gray-700">
			{i18nMsg(
				'export-global-description',
				"Exportez vos données comptables (comptes, écritures, contacts, produits, factures de vente, comptes et transactions bancaires) au format CSV dans un fichier ZIP. Utilisez cet export pour archiver ou conserver vos données 10 ans (CO suisse art. 958f). ⚠️ Il ne couvre pas encore l'ensemble de votre comptabilité : lisez ci-dessous ce qu'il ne contient pas avant de compter dessus pour migrer vers un autre logiciel.",
			)}
		</p>
	</header>

	<div class="rounded border border-gray-200 bg-gray-50 p-4 text-sm text-gray-700">
		<p>
			{i18nMsg(
				'export-global-content-includes',
				"L'export contient : plan comptable, écritures, contacts, produits, factures, comptes bancaires, transactions, règles de réconciliation, et un manifeste metadata.json avec hash SHA-256 de chaque fichier pour vérification d'intégrité.",
			)}
		</p>
		<p class="mt-2">
			{i18nMsg(
				'export-global-content-excludes',
				"Ne contient pas : factures fournisseurs et leurs lignes, avoirs, projets analytiques (les écritures portent un identifiant de projet, mais la table des projets est absente), lots de paiement, personnes de contact, pièces justificatives importées, utilisateurs (données personnelles et mots de passe), tokens de session, journal d'audit interne, état d'onboarding.",
			)}
		</p>
		<p class="mt-2 italic">
			{i18nMsg(
				'export-global-souverainete-note',
				'Vos données vous appartiennent. Kesh ne fait aucune copie de cet export sur ses serveurs.',
			)}
		</p>
	</div>

	<div class="flex items-center gap-3">
		<button
			type="button"
			class="rounded bg-blue-600 px-4 py-2 text-white shadow-sm hover:bg-blue-700 disabled:opacity-50"
			disabled={exporting}
			onclick={startExport}
			data-testid="export-global-start"
		>
			{#if exporting}
				{i18nMsg('export-global-loading', "Génération de l'export…")}
			{:else}
				{i18nMsg('export-global-button', "Lancer l'export")}
			{/if}
		</button>
	</div>

	{#if errorMsg}
		<p
			class="rounded bg-red-50 p-3 text-sm text-red-900"
			role="alert"
			data-testid="export-global-error"
		>
			{errorMsg}
		</p>
	{/if}

	{#if successMsg}
		<p
			class="rounded bg-green-50 p-3 text-sm text-green-900"
			role="status"
			data-testid="export-global-success"
		>
			{successMsg}
		</p>
	{/if}
</section>
