<!--
  #213 — Gestion des personnes de contact d'une entreprise (CRM, informatif).
  Affiché dans le formulaire d'édition d'un contact Entreprise existant.
-->
<script lang="ts">
	import { Input } from '$lib/components/ui/input';
	import { Button } from '$lib/components/ui/button';
	import { i18nMsg } from '$lib/shared/utils/i18n.svelte';
	import {
		listContactPersons,
		createContactPerson,
		deleteContactPerson,
		type ContactPerson
	} from './contact-persons.api';

	interface Props {
		contactId: number;
	}
	let { contactId }: Props = $props();

	let persons = $state<ContactPerson[]>([]);
	let loading = $state(false);
	let error = $state('');
	// Formulaire d'ajout inline.
	let addFirstName = $state('');
	let addLastName = $state('');
	let addRole = $state('');
	let addEmail = $state('');
	let addPhone = $state('');
	let adding = $state(false);

	async function load() {
		loading = true;
		error = '';
		try {
			persons = await listContactPersons(contactId);
		} catch {
			error = i18nMsg('contact-persons-load-error', 'Impossible de charger les personnes de contact');
		} finally {
			loading = false;
		}
	}

	$effect(() => {
		if (contactId) load();
	});

	async function addPerson() {
		if (!addFirstName.trim() || !addLastName.trim()) {
			error = i18nMsg('contact-persons-name-required', 'Prénom et nom obligatoires');
			return;
		}
		adding = true;
		error = '';
		try {
			await createContactPerson(contactId, {
				firstName: addFirstName.trim(),
				lastName: addLastName.trim(),
				role: addRole.trim() || null,
				email: addEmail.trim() || null,
				phone: addPhone.trim() || null
			});
			addFirstName = '';
			addLastName = '';
			addRole = '';
			addEmail = '';
			addPhone = '';
			await load();
		} catch {
			error = i18nMsg('contact-persons-add-error', "Impossible d'ajouter la personne");
		} finally {
			adding = false;
		}
	}

	async function removePerson(id: number) {
		try {
			await deleteContactPerson(id);
			await load();
		} catch {
			error = i18nMsg('contact-persons-delete-error', 'Impossible de supprimer la personne');
		}
	}
</script>

<div class="rounded-md border border-border p-3" data-testid="contact-persons-manager">
	<h4 class="mb-2 text-sm font-medium">
		{i18nMsg('contact-persons-title', 'Personnes de contact')}
		<span class="font-normal text-text-muted"> ({i18nMsg('contact-persons-hint', 'à titre informatif')})</span>
	</h4>

	{#if error}
		<p class="mb-2 text-sm text-red-600" role="alert">{error}</p>
	{/if}

	{#if loading}
		<p class="text-sm text-text-muted">{i18nMsg('loading', 'Chargement…')}</p>
	{:else if persons.length === 0}
		<p class="mb-2 text-sm text-text-muted" data-testid="contact-persons-empty">
			{i18nMsg('contact-persons-empty', 'Aucune personne de contact.')}
		</p>
	{:else}
		<ul class="mb-3 divide-y divide-border" data-testid="contact-persons-list">
			{#each persons as p (p.id)}
				<li class="flex items-center justify-between py-1.5 text-sm">
					<span>
						<strong>{p.firstName} {p.lastName}</strong>
						{#if p.role}<span class="text-text-muted"> — {p.role}</span>{/if}
						{#if p.email}<span class="text-text-muted"> · {p.email}</span>{/if}
						{#if p.phone}<span class="text-text-muted"> · {p.phone}</span>{/if}
					</span>
					<button
						type="button"
						class="text-red-600 hover:underline"
						onclick={() => removePerson(p.id)}
						data-testid={`contact-person-delete-${p.id}`}
					>
						{i18nMsg('delete', 'Supprimer')}
					</button>
				</li>
			{/each}
		</ul>
	{/if}

	<div class="grid grid-cols-2 gap-2">
		<Input placeholder={i18nMsg('field-first-name', 'Prénom')} bind:value={addFirstName} maxlength={70} />
		<Input placeholder={i18nMsg('field-last-name', 'Nom')} bind:value={addLastName} maxlength={70} />
		<Input placeholder={i18nMsg('contact-persons-role', 'Fonction')} bind:value={addRole} maxlength={100} />
		<Input placeholder={i18nMsg('contact-form-email', 'Email')} bind:value={addEmail} maxlength={320} />
		<Input placeholder={i18nMsg('contact-form-phone', 'Téléphone')} bind:value={addPhone} maxlength={50} />
		<Button type="button" onclick={addPerson} disabled={adding} data-testid="contact-person-add">
			{i18nMsg('contact-persons-add', 'Ajouter')}
		</Button>
	</div>
</div>
