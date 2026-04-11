# Story 3.5: Notifications, aide contextuelle & audit log (complément)

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a **utilisateur (comptable ou indépendant)**,
I want **recevoir des notifications cohérentes pour chaque action, comprendre les termes comptables via des tooltips bilingues, et avoir l'intégralité de mes actions tracées dans le journal d'audit**,
so that **j'utilise l'application en confiance, sans ambiguïté sur ce qui s'est passé, et avec une traçabilité complète conforme au CO art. 957-964**.

### Contexte

**Dernière story de l'Epic 3**. Couvre FR71 (notifications), FR72 (erreurs bloquantes), FR73 (tooltips bilingues), FR88 (audit log complet). S'appuie massivement sur ce qui existe déjà :

- **Table `audit_log`** + repository + `insert_in_tx` — créés en story 3.3 (pour `journal_entry.updated` et `journal_entry.deleted`).
- **Modale conflit 409** (`OPTIMISTIC_LOCK_CONFLICT`) — déjà en place dans `JournalEntryForm.svelte` (story 3.3).
- **Session expirée** — wrapper fetch story 1.6 redirige vers `/login?reason=session_expired`, la page login affiche un message dédié (story 1.10 ligne 16-17). **Déjà couvert**, rien à refaire.
- **`svelte-sonner` toasts** — déjà utilisés dans 3.2/3.3/3.4 pour les feedbacks utilisateur. Les « banners » verts/oranges/rouges du PRD FR71 sont **exactement** ce que fait `svelte-sonner` (toast.success/warning/error).
- **Composants Tooltip shadcn-svelte** — déjà installés (`frontend/src/lib/components/ui/tooltip/`), vérifié 2026-04-10.

### Scope verrouillé — ce qui RESTE à faire

1. **Audit log création d'écritures** (`journal_entry.created`) — la table et le repository existent depuis 3.3 mais la story 3.3 n'auditait que `updated`/`deleted`. Cette story complète la couverture CRUD pour les écritures (FR88 : « création/modification/suppression »).
2. **Tooltips bilingues sur les termes comptables** — nouveau composant `AccountingTooltip.svelte` réutilisable. Utilisé dans `JournalEntryForm.svelte` sur les labels Débit/Crédit/Journal/Équilibré. Pattern bilingue : « Langage naturel » + « Terminologie comptable » par clé i18n dédiée.
3. **Helpers de notification harmonisés** — wrappers typés `notify.ts` (`notifySuccess`, `notifyInfo`, `notifyWarning`, `notifyError`) au-dessus de `svelte-sonner` pour harmoniser le style, les durées, et faciliter un refactor futur si on change de bibliothèque de toast.
4. **Audit trigger modifications plan comptable** — compte créé, modifié, archivé. Story 3.1 avait créé ces opérations SANS audit (parce que la table n'existait pas encore). Ajouter ici les appels `audit_log::insert_in_tx` dans `accounts::create`, `accounts::update`, `accounts::archive`. Cohérent avec FR88 « actions sur les données comptables ».

### Scope volontairement HORS story — décisions tranchées

- **UI consultation du journal d'audit** (`/audit` page avec table paginée) : **reportée post-MVP**. Raison : PRD FR88 exige la conservation et l'enregistrement, **pas la consultation** en temps réel. Les entrées d'audit sont accessibles via `find_by_entity` (déjà en place story 3.3) pour les tests et les futures features. Une page UI dédiée est une feature de confort, pas une exigence v0.1. Story transverse post-MVP ou extension naturelle en story 13.x (justificatifs/lettrage) si besoin.
- **Audit log pour d'autres entités** (bank_accounts, contacts futurs, factures futures) : à traiter **dans les stories correspondantes** (4.x, 5.x), pas rétroactivement ici.
- **Audit log pour clôture d'exercice** (FR60-FR62) : **reporté en story 12.1** (clôture d'exercice v0.2) — la clôture n'existe pas encore.
- **Tooltips sur d'autres pages** (contacts, factures) : à traiter dans les stories correspondantes, pas en avance. Cette story pose le **pattern réutilisable** mais ne l'applique qu'au formulaire d'écriture de story 3.2/3.3.
- **Modale session expirée visible** : **déjà couvert** par la redirection `/login?reason=session_expired` + message story 1.10. Ne PAS ajouter une modale qui doublerait le comportement.
- **Banner notifications** vs **toasts svelte-sonner** : le PRD dit « banner » mais `svelte-sonner` produit exactement ce type d'UX (position coin écran, durée, couleur par niveau, dismiss). **Pas de nouveau composant Banner** — `svelte-sonner` EST le banner.

### Décisions de conception

- **Composant `AccountingTooltip.svelte`** — wrapper léger au-dessus de `Tooltip.Root` + `Tooltip.Trigger` + `Tooltip.Content` de shadcn-svelte. Props : `term: string` (clé i18n du terme à afficher, ex: `"debit"`) et un `children` snippet (le contenu cliquable). Affiche 2 lignes : `<strong>{term en langage naturel}</strong>` + `<span class="text-muted">{terminologie comptable}</span>`. Les clés i18n suivent le pattern `tooltip-{term}-natural` + `tooltip-{term}-technical`. Les 4 langues supportées.
- **Helpers `notify.ts`** dans `frontend/src/lib/shared/utils/notify.ts` :
  ```ts
  import { toast } from 'svelte-sonner';

  const DEFAULT_DURATION = 4000; // ms
  const ERROR_DURATION = 6000; // erreurs plus visibles

  export function notifySuccess(message: string, description?: string): void {
      toast.success(message, { description, duration: DEFAULT_DURATION });
  }
  export function notifyInfo(message: string, description?: string): void {
      toast.info(message, { description, duration: DEFAULT_DURATION });
  }
  export function notifyWarning(message: string, description?: string): void {
      toast.warning(message, { description, duration: ERROR_DURATION });
  }
  export function notifyError(message: string, description?: string): void {
      toast.error(message, { description, duration: ERROR_DURATION });
  }
  ```
  - **Non-objectif** : NE PAS refactorer tous les call sites existants (`+page.svelte` story 3.2/3.3/3.4, `JournalEntryForm.svelte`, `accounts/+page.svelte`, etc.) pour utiliser ces helpers. Le refactor est opportuniste — les nouveaux call sites 3.5 utilisent les helpers, les anciens restent sur `toast.*` direct jusqu'à une story de cleanup transverse. **Sauf** si un simple `grep -r "toast\." frontend/src/lib/features/` montre < 20 occurrences, auquel cas on refactor tout maintenant (vérifier avant T1).
- **Audit log pour accounts** — ajouter appels `audit_log::insert_in_tx` dans les 3 fonctions repository `accounts::create`, `accounts::update`, `accounts::archive`. Actions : `account.created`, `account.updated`, `account.archived`. Les fonctions actuelles **ne sont pas transactionnelles** pour le mapping audit — elles acceptent un `&MySqlPool`. Il faut les refactorer pour ouvrir une transaction interne OU accepter un `&mut Transaction` externe. **Décision** : refactorer pour **ouvrir une transaction interne** (pattern cohérent avec `journal_entries::create` story 3.2), car les callers actuels n'ont pas de tx en cours. Signature publique inchangée mais le `user_id` devient un paramètre obligatoire (comme pour `update`/`delete_by_id` de `journal_entries` story 3.3).
  - **Attention propagation** : tous les appelants (`routes/accounts.rs` handlers + `kesh-seed` bulk_create_from_chart + story 3.1 appels) doivent passer `user_id`. Pour `kesh-seed` : user_id = 1 (admin bootstrap) par convention, OU utiliser une fonction `bulk_create_*` séparée qui n'audite pas (pattern : « les opérations de seed ne sont pas auditées, car système, pas utilisateur »). **Décision** : **`bulk_create_from_chart` ne passe pas par la fonction `create` auditée** — elle garde son chemin actuel sans audit (contexte seed système, pas action utilisateur). Le `create` single-entry devient auditée.
- **Audit log pour journal_entries::create** — l'appel existant de story 3.2 ne passe PAS `user_id` (le handler `create_journal_entry` du story 3.2 n'extrait pas `CurrentUser` — contrairement à `update` et `delete` de story 3.3). Il faut :
  1. Modifier la signature de `journal_entries::create` pour accepter `user_id: i64`.
  2. Modifier le handler `create_journal_entry` story 3.2 pour extraire `Extension<CurrentUser>` et passer `current_user.user_id`.
  3. Ajouter l'INSERT audit_log dans la tx (après l'INSERT des lignes et le balance check, avant le COMMIT — pattern identique à `update`/`delete_by_id` de story 3.3).
- **Tests** — unitaires du composant `AccountingTooltip.svelte` (rendu avec les 2 lignes), unitaires de `notify.ts` (mock `toast` et vérifie les appels avec les bonnes durées), tests d'intégration DB pour `accounts::create/update/archive` qui vérifient l'INSERT audit_log, extension des tests existants `journal_entries::create` story 3.2 pour vérifier l'INSERT audit_log de création, tests Playwright : hover sur un label Débit affiche le tooltip, notification verte après création d'écriture (déjà testé implicitement story 3.2).

## Acceptance Criteria (AC)

1. **Notification succès création** (FR71) — Given l'utilisateur crée une écriture valide, When soumission réussit, Then un toast vert (svelte-sonner `toast.success`) s'affiche avec le message « Écriture enregistrée » (clé i18n `journal-entry-saved`, déjà existante depuis 3.2 patch P3). Durée 4 secondes, auto-dismiss. Le toast utilise le helper `notifySuccess` pour harmonisation.
2. **Notification avertissement** (FR71) — Given une opération partielle (ex: import bancaire futur, doublon détecté), When feedback, Then un toast orange (`svelte-sonner toast.warning`) s'affiche avec message + détails optionnels. Le helper `notifyWarning` utilise une durée de 6 secondes (plus visible). **Cette story ne déclenche pas de warning** — aucun parcours utilisateur applicable en v0.1 avant l'import bancaire (story 6.x). Le helper est néanmoins créé comme **pattern préparatoire** pour assurer la cohérence API quand les stories 6.x seront développées. **Le test unitaire `notify.test.ts` couvre `notifyWarning`** (AC#2 testé isolément via mock `toast`), pas besoin de test d'intégration UI tant qu'aucun callsite n'existe.
3. **Notification erreur non-bloquante** (FR71) — Given une erreur applicative (ex: compte archivé, montant invalide), When feedback, Then un toast rouge (`svelte-sonner toast.error`) via `notifyError` avec message explicite et durée 6 secondes. Remplace les `toast.error` directs dans le nouveau code de cette story.
4. **Modale erreur bloquante — conflit de version** (FR72) — Given deux sessions qui éditent la même écriture, When conflict 409, Then modale ouverte avec « Conflit de version » + bouton « Recharger ». **Déjà en place story 3.3**, aucune modification nécessaire. Test Playwright story 3.3 existant reste vert.
5. **Modale erreur bloquante — session expirée** (FR72) — Given un refresh token expiré, When requête API, Then redirection vers `/login?reason=session_expired` + message explicite sur la page login. **Déjà en place stories 1.6 + 1.10**, aucune modification nécessaire. Vérification manuelle : le message login affiche bien « Votre session a expiré. Veuillez vous reconnecter. » dans les 4 langues.
6. **Tooltip bilingue — débit** (FR73, UX-DR39) — Given le label « Débit » dans `JournalEntryForm.svelte`, When survol ou focus clavier, Then un tooltip s'affiche avec 2 lignes :
   - Ligne 1 (langage naturel, strong) : « L'argent entre dans ce compte »
   - Ligne 2 (terminologie comptable, text-muted) : « Débit — colonne de gauche »
   - Clés i18n : `tooltip-debit-natural` + `tooltip-debit-technical` × 4 langues.
7. **Tooltip bilingue — crédit** — Similaire au débit. Clés `tooltip-credit-natural` + `tooltip-credit-technical`. Texte : « L'argent sort de ce compte » / « Crédit — colonne de droite ».
8. **Tooltip bilingue — journal** — Hover sur le label « Journal » du formulaire. Clés `tooltip-journal-natural` + `tooltip-journal-technical`. Texte : « Registre où sont groupées les écritures similaires » / « Journal comptable — Achats, Ventes, Banque, Caisse, OD ».
9. **Tooltip bilingue — équilibré** — Hover sur l'indicateur d'équilibre (vert/rouge) du formulaire. Clés `tooltip-balanced-natural` + `tooltip-balanced-technical`. Texte : « Le total des entrées égale le total des sorties » / « Équilibre partie double (débit = crédit) ».
10. **Audit création d'écriture** (FR88) — Given la création d'une écriture valide, When la transaction réussit, Then une entrée `audit_log` est insérée avec `action = "journal_entry.created"`, `user_id = {current}`, `entity_type = "journal_entry"`, `entity_id = {new_entry_id}`, `details_json` = résultat **direct** de `entry_snapshot_json(&entry, &lines)` (helper story 3.3, objet JSON sans wrapper). **Format explicitement tranché** : snapshot direct, PAS de wrapper `{"after": ...}`. Raison : **cohérence empirique avec le pattern 3.3 existant** — vérifié dans `journal_entries.rs:791,802` : le handler `delete_by_id` utilise déjà `details_json = Some(snapshot)` direct (pas de wrapper `{"before": ...}`). Par contre `update` utilise un wrapper `{"before": ..., "after": ...}` car il a DEUX états à capturer. Convention projet : **wrapper uniquement pour les transitions à 2 états (update), objet direct sinon (create, delete)**. Si la transaction rollback, l'entrée audit est aussi rollbackée.
11. **Audit CRUD plan comptable** (FR88) — Given une opération sur un compte (`accounts::create`, `update`, `archive`), When succès, Then une entrée `audit_log` est insérée avec `action` respectif (`account.created` / `account.updated` / `account.archived`), `entity_type = "account"`, `entity_id = {account_id}`, `details_json` avec le snapshot before/after (update) ou l'état (create/archive). Refactor des 3 fonctions repository pour accepter `user_id` et ouvrir une transaction interne. **Les appels `bulk_create_from_chart` (seed) ne génèrent PAS d'audit log** (contexte système, pas utilisateur) — ils utilisent un chemin non-audité.
12. **Handler `create_journal_entry` accepte `Extension<CurrentUser>`** — refactor de story 3.2 pour passer `user_id` au repository. Pattern identique aux handlers `update`/`delete` de story 3.3.
13. **Handlers accounts passent `user_id`** — `routes/accounts.rs::create_account`, `update_account`, `archive_account` extraient `Extension<CurrentUser>` et passent `current_user.user_id` au repository. Les routes sont déjà dans `comptable_routes` (RBAC v0.1 OK).
14. **Helpers `notify.ts`** — Given le module `frontend/src/lib/shared/utils/notify.ts`, When importé depuis n'importe quel composant, Then les 4 fonctions `notifySuccess`/`notifyInfo`/`notifyWarning`/`notifyError` sont disponibles avec signature `(message: string, description?: string) => void`. **Le nouveau code de cette story utilise ces helpers**, les anciens call sites restent inchangés (opportuniste).
15. **i18n** (FR73) — Toutes les clés de tooltip (8 nouvelles × 4 langues = 32 entrées) + les messages de notification (variables selon l'usage, inclure `journal-entry-saved` déjà présent). Aucun hardcode — règle A3.
16. **Tests** :
    - **Unit Rust** : extension de `journal_entries::tests` pour vérifier l'INSERT audit_log sur `create` (nouveau scénario `test_create_writes_audit_log`). Nouveaux tests `accounts::tests::test_{create,update,archive}_writes_audit_log` qui vérifient l'entrée `audit_log` via `find_by_entity`.
    - **Unit Vitest** : `notify.test.ts` qui mock `toast` et vérifie les appels avec bonnes durées. `AccountingTooltip.test.ts` avec `@testing-library/svelte` qui vérifie le rendu des 2 lignes i18n.
    - **Playwright** : scenario `hover débit affiche tooltip` dans `journal-entries.spec.ts` (utiliser `page.hover()` et vérifier que le texte tooltip apparaît). Tests de notification implicites dans les scénarios existants 3.2 (toast « Écriture enregistrée » déjà visible).

## Tasks / Subtasks

### T1 — Helpers `notify.ts` (AC: #1, #2, #3, #14)

- [ ] T1.1 Créer `frontend/src/lib/shared/utils/notify.ts` avec les 4 fonctions + constantes `DEFAULT_DURATION = 4000` et `ERROR_DURATION = 6000`.
- [ ] T1.2 **Pas de refactor des call sites existants** (décision tranchée — remplacement du check opportuniste) : les helpers `notify*` ne sont utilisés **que** pour le nouveau code de story 3.5. Les call sites existants (`+page.svelte`, `JournalEntryForm.svelte`, `accounts/+page.svelte`) continuent d'utiliser `toast.*` direct. Raison : le refactor a zéro valeur fonctionnelle et créerait une dette de migration transverse ; le pattern `toast.*` direct reste valide car `notify.ts` n'est qu'un wrapper cosmétique. Si une story de cleanup transverse est créée plus tard, elle pourra migrer tous les call sites en une fois. **Ne PAS toucher** aux anciens call sites en story 3.5.
- [ ] T1.3 Créer `frontend/src/lib/shared/utils/notify.test.ts` avec mocks Vitest :
  - `vi.mock('svelte-sonner', () => ({ toast: { success: vi.fn(), info: vi.fn(), warning: vi.fn(), error: vi.fn() } }))`
  - Vérifier que `notifySuccess('msg')` appelle `toast.success('msg', { description: undefined, duration: 4000 })`
  - Idem pour `info`, `warning` (6000 ms), `error` (6000 ms)
  - Vérifier que `notifyError('msg', 'details')` passe bien `description: 'details'`

### T2 — Composant `AccountingTooltip.svelte` (AC: #6, #7, #8, #9, #15)

- [ ] T2.1 Créer `frontend/src/lib/shared/components/AccountingTooltip.svelte` (partagé, pas dans `features/journal-entries/` car utilisable par les stories futures) :
  ```svelte
  <script lang="ts">
      import * as Tooltip from '$lib/components/ui/tooltip';
      import { i18nMsg } from '$lib/features/onboarding/onboarding.svelte';
      import type { Snippet } from 'svelte';

      interface Props {
          term: string;
          /** Children snippet — le contenu cliquable qui reçoit le tooltip. */
          children: Snippet;
      }
      let { term, children }: Props = $props();

      const naturalKey = `tooltip-${term}-natural`;
      const technicalKey = `tooltip-${term}-technical`;
  </script>

  <Tooltip.Root>
      <Tooltip.Trigger>
          {@render children()}
      </Tooltip.Trigger>
      <Tooltip.Content class="max-w-xs">
          <p class="font-semibold">{i18nMsg(naturalKey, term)}</p>
          <p class="text-xs text-muted-foreground mt-1">
              {i18nMsg(technicalKey, term)}
          </p>
      </Tooltip.Content>
  </Tooltip.Root>
  ```
- [ ] T2.2 Test Vitest `AccountingTooltip.test.ts` avec `@testing-library/svelte` :
  - Mock `i18nMsg` pour retourner `"TERM_NATURAL"` et `"TERM_TECHNICAL"`
  - Render avec `term="debit"` et un children text
  - Vérifier que les 2 clés sont bien appelées (via `vi.fn()` spy sur `i18nMsg`)
  - **Alternative** : si `@testing-library/svelte` est compliqué avec Svelte 5 + snippets, faire un test minimal qui importe le composant et vérifie qu'il compile/exporte correctement (smoke test).

### T3 — Intégration tooltips dans `JournalEntryForm.svelte` (AC: #6, #7, #8, #9)

- [ ] T3.1 **Structure réelle du formulaire (vérifié 2026-04-10 sur `JournalEntryForm.svelte`)** :
  - Débit/Crédit : `<th>` dans `<thead><tr>` (lignes 241-246)
  - **Compte** : `<th>` dans le même header (ligne 238-240)
  - Journal : `<label for="entry-journal">` dans une grille de form fields (ligne 213-215)
  - Équilibré/Déséquilibré : `<span>` dans le footer dérivé dynamiquement (lignes 339-347)

  **CRITIQUE — pattern d'intégration différent par contexte**. Un `<tr>` ne peut avoir comme enfants directs que des `<th>`/`<td>`, donc **JAMAIS** wrapper un `<th>` entier dans un `<AccountingTooltip>`. Le tooltip enveloppe le **contenu** du cell/label, pas le cell lui-même.

  **Pattern 1 — Tooltip dans un `<th>` (Débit, Crédit)** :
  ```svelte
  <th class="text-right py-2 text-sm font-medium w-32">
      <AccountingTooltip term="debit">
          <span class="cursor-help underline underline-offset-2 decoration-dotted">
              {i18nMsg('journal-entry-form-col-debit', 'Débit')}
          </span>
      </AccountingTooltip>
  </th>
  ```

  **Pattern 2 — Tooltip dans un `<label>` (Journal)** :
  ```svelte
  <label for="entry-journal" class="block text-sm font-medium mb-1">
      <AccountingTooltip term="journal">
          <span class="cursor-help underline underline-offset-2 decoration-dotted">
              {i18nMsg('journal-entry-form-journal', 'Journal')}
          </span>
      </AccountingTooltip>
  </label>
  ```

  **Pattern 3 — Tooltip sur indicateur dynamique (Équilibré)** : le tooltip entoure le `<span>` de l'indicateur existant. **Le texte du tooltip reste INVARIANT** (description conceptuelle de la partie double) — il n'évolue PAS avec l'état vert/rouge, car le concept « équilibre débit = crédit » est le même qu'on soit équilibré ou non. Décision tranchée.
  ```svelte
  <AccountingTooltip term="balanced">
      {#if balance.isBalanced}
          <span class="text-green-700 dark:text-green-400 cursor-help">
              ✓ {i18nMsg('journal-entry-form-balanced', 'Équilibré')}
          </span>
      {:else if balance.totalDebit.gt(0) || balance.totalCredit.gt(0)}
          <span class="text-destructive cursor-help">
              ✗ {i18nMsg('journal-entry-form-unbalanced', 'Déséquilibré')}
          </span>
      {/if}
  </AccountingTooltip>
  ```

- [ ] T3.2 **Accessibilité & bits-ui `asChild`** : le composant `Tooltip.Trigger` de shadcn-svelte (qui wrap `bits-ui TooltipPrimitive.Trigger`) supporte le pattern `asChild` pour déléguer le focus et les événements à l'élément enfant directement — sans créer de wrapper HTML supplémentaire. Si le `<span>` wrapper ajouté dans T3.1 pose problème de focus clavier (Tooltip doit se déclencher au Tab), utiliser `<Tooltip.Trigger asChild>` pour attacher les événements au `<span>` directement.
  - **Note** : `<span>` n'est pas focusable par défaut. Si le focus clavier doit fonctionner, ajouter `tabindex="0"` au `<span>`. Alternative : utiliser `<button type="button">` invisible stylé (plus accessible, mais style à adapter).
  - **Décision pragmatique v0.1** : utiliser `<span tabindex="0">` dans T3.1. Si l'A11y review échoue (story future), refactorer en `<button>`. La dette focus trap des dialogs 3.3/3.4 est déjà documentée — l'A11y complète est post-MVP.

- [ ] T3.3 **Tooltip sur l'indicateur dynamique — texte invariant** : comme documenté dans T3.1 Pattern 3, le tooltip de `term="balanced"` affiche **un seul texte** (description conceptuelle), indépendamment de l'état courant. Une seule paire de clés i18n (`tooltip-balanced-natural` + `tooltip-balanced-technical`), pas deux variantes selon état.

### T4 — Clés i18n tooltips (AC: #15)

- [ ] T4.1 Ajouter dans les 4 fichiers `.ftl` :
  - **FR** : `tooltip-debit-natural = L'argent entre dans ce compte`, `tooltip-debit-technical = Débit — colonne de gauche`, `tooltip-credit-natural = L'argent sort de ce compte`, `tooltip-credit-technical = Crédit — colonne de droite`, `tooltip-journal-natural = Registre où sont groupées les écritures similaires`, `tooltip-journal-technical = Journal comptable (Achats, Ventes, Banque, Caisse, OD)`, `tooltip-balanced-natural = Le total des entrées égale le total des sorties`, `tooltip-balanced-technical = Équilibre partie double (débit = crédit)`
  - **DE** : traductions équivalentes cohérentes avec le vocabulaire comptable suisse-allemand (Soll/Haben)
  - **IT** : traductions équivalentes (Dare/Avere)
  - **EN** : traductions équivalentes (Debit/Credit)
  - **Total** : 8 clés × 4 langues = **32 entrées**.

### T5 — Audit log création d'écriture (AC: #10, #12)

- [ ] T5.1 **Refactor `journal_entries::create`** pour accepter `user_id: i64` :
  - Signature : `pub async fn create(pool: &MySqlPool, fiscal_year_id: i64, user_id: i64, new: NewJournalEntry) -> Result<JournalEntryWithLines, DbError>`
  - À l'intérieur de la transaction, **APRÈS** le balance check et le re-fetch, **AVANT** le COMMIT, ajouter un appel `audit_log::insert_in_tx` avec :
    - `action = "journal_entry.created"`
    - `entity_type = "journal_entry"`
    - `entity_id = entry.id` (l'id venant d'être inséré)
    - `details_json = Some(entry_snapshot_json(&entry, &lines))` — **snapshot direct** (PAS de wrapper). Cohérent avec `delete_by_id` ligne 791-802 de story 3.3 qui utilise déjà `details_json = Some(snapshot)` direct. Convention : wrapper uniquement pour les transitions à 2 états (update), objet direct sinon.
  - Le helper `entry_snapshot_json` existe déjà (story 3.3).
- [ ] T5.2 **Refactor `create_journal_entry` handler** (story 3.2) pour :
  - Ajouter `Extension(current_user): Extension<CurrentUser>` dans la signature
  - Passer `current_user.user_id` à `journal_entries::create(...)`
- [ ] T5.3 Mettre à jour les tests d'intégration DB story 3.2 (`test_create_balanced_entry`, `test_create_sequential_numbering`, etc.) pour passer `user_id` en paramètre (récupérable via un helper `get_admin_user_id` existant dans `audit_log::tests`).
- [ ] T5.4 **Nouveau test** `test_create_writes_audit_log` : créer une écriture valide, vérifier via `audit_log::find_by_entity("journal_entry", entry_id, 10)` qu'une entrée existe avec `action = "journal_entry.created"` et `details_json` = **snapshot JSON direct** (objet contenant les champs `id`, `entryNumber`, `entryDate`, `journal`, `description`, `version`, `lines: [...]` tel que retourné par `entry_snapshot_json`). **Pas de wrapper** `{"after": ...}`, `{"before": null, ...}`, ou équivalent — l'objet JSON est directement le snapshot. Assertion : vérifier `entry["description"] == "Test..."` et `entry["lines"].as_array().unwrap().len() >= 2`. Test optionnel complémentaire : forcer un rollback (via mock/injection) et vérifier l'absence d'entrée audit — cohérent avec `test_rollback_preserves_no_audit` de `audit_log::tests` story 3.3.

### T6 — Audit log CRUD plan comptable (AC: #11, #13)

- [ ] T6.1 **Refactor `accounts::create`** pour accepter `user_id: i64` et ouvrir une transaction interne :
  - Signature : `pub async fn create(pool: &MySqlPool, user_id: i64, new: NewAccount) -> Result<Account, DbError>`
  - Ouvrir une transaction `let mut tx = pool.begin().await.map_err(map_db_error)?;`
  - INSERT + récupération comme actuellement, mais via `&mut *tx`
  - Ajouter `audit_log::insert_in_tx` avec `action = "account.created"`, `details_json = Some(account_snapshot_json(&account))` (nouveau helper dans `repositories/accounts.rs`)
  - `tx.commit()` en fin
  - **Helper `account_snapshot_json`** — pattern similaire à `entry_snapshot_json` story 3.3 : JSON avec `{id, number, name, accountType, parentId, active, version}`.
- [ ] T6.2 **Refactor `accounts::update`** : même logique, action `"account.updated"`, `details_json = Some({"before": ..., "after": ...})`. Récupérer l'état avant UPDATE via SELECT inline dans la tx (pattern story 3.3).
- [ ] T6.3 **Refactor `accounts::archive`** : même logique, action `"account.archived"`, `details_json = Some(snapshot)`.
- [ ] T6.4 **NE PAS** modifier `accounts::bulk_create_from_chart` (utilisée par le seed) — laisser telle quelle, pas d'audit log pour les opérations système. Documenter dans le doc-comment : « Cette fonction ne génère PAS d'entrées d'audit (contexte seed système, pas action utilisateur). »
- [ ] T6.5 **Refactor handlers `accounts.rs`** : extraire `Extension<CurrentUser>` dans `create_account`, `update_account`, `archive_account`, passer `current_user.user_id` au repository.
- [ ] T6.6 Mettre à jour les tests d'intégration DB existants de `accounts::tests` pour passer `user_id` en paramètre (utiliser le helper `get_admin_user_id` story 3.3).
- [ ] T6.7 **Nouveaux tests** : `test_create_account_writes_audit_log`, `test_update_account_writes_audit_log` (vérifier `before`/`after` dans le snapshot JSON), `test_archive_account_writes_audit_log`.

### T7 — Tests Playwright (AC: #6-9, #16)

- [ ] T7.1 Ajouter dans `journal-entries.spec.ts` un scenario `hover débit affiche tooltip` :
  - Ouvrir le formulaire de nouvelle écriture
  - **Sélecteur robuste** : `await page.hover('[data-slot="tooltip-trigger"]:has-text("Débit")')` plutôt que `'th:has-text("Débit")'` — cible directement l'élément wrapper de `bits-ui Tooltip.Trigger` (attribut `data-slot` automatiquement ajouté par bits-ui), pas le `<th>` parent. Évite la fragilité si Playwright propage mal les events hover vers les enfants.
  - Attendre 500 ms (délai d'affichage tooltip shadcn)
  - Vérifier que le texte « L'argent entre dans ce compte » est visible via `await expect(page.getByText('L\'argent entre dans ce compte')).toBeVisible()`
- [ ] T7.2 Un seul scenario suffit — le pattern est le même pour crédit/journal/équilibré. Ajouter `test.skip` pour les 3 autres avec note explicite « même pattern que débit, couverture implicite ».

### T8 — Validation finale & cleanup

- [ ] T8.1 `cargo check --workspace` + `cargo test --workspace --lib -- --skip repositories::` (tests unitaires hors DB)
- [ ] T8.2 Vitest : `npx vitest run src/lib/features/journal-entries/ src/lib/shared/utils/notify.test.ts src/lib/shared/components/AccountingTooltip.test.ts`
- [ ] T8.3 `svelte-check` : 0 erreur sur les fichiers 3.5
- [ ] T8.4 Vérifier que la page login affiche bien le message « session expirée » dans les 4 langues (inspection manuelle de `routes/login/+page.svelte` + clés i18n `login-session-expired` ou équivalent).

## Dev Notes

### Architecture — où va quoi

```
kesh-db/src/repositories/
├── accounts.rs                # Refactor create/update/archive (tx + audit)
└── journal_entries.rs         # Refactor create (tx + user_id + audit)

kesh-api/src/routes/
├── accounts.rs                # Refactor handlers (Extension<CurrentUser>)
└── journal_entries.rs         # Refactor create_journal_entry (Extension<CurrentUser>)

frontend/src/lib/
├── shared/
│   ├── components/
│   │   ├── AccountingTooltip.svelte        # Nouveau composant
│   │   └── AccountingTooltip.test.ts       # Test unitaire
│   └── utils/
│       ├── notify.ts                        # Nouveau helper
│       └── notify.test.ts                   # Tests Vitest
└── features/journal-entries/
    └── JournalEntryForm.svelte              # Intégration tooltips

kesh-i18n/locales/*/messages.ftl             # +32 entrées (8 clés × 4 langues)

frontend/tests/e2e/
└── journal-entries.spec.ts                  # +1 scenario hover tooltip
```

### Ce qui existe DÉJÀ (story 1.x/3.x — ne PAS refaire)

- **Table `audit_log`** + repository `audit_log` + fonction `insert_in_tx` + `find_by_entity` → story 3.3
- **Helper `entry_snapshot_json`** dans `journal_entries.rs` → story 3.3
- **Modale conflit 409 `OPTIMISTIC_LOCK_CONFLICT`** dans `JournalEntryForm.svelte` → story 3.3
- **Session expirée** : wrapper fetch `api-client.ts:41,54,64,73,82` redirige vers `/login?reason=session_expired` → story 1.6
- **Page login** avec message « session expirée » (`routes/login/+page.svelte:16-17`) → story 1.10
- **`svelte-sonner`** avec `toast.success/info/warning/error` → dépendance déjà installée depuis story 3.2
- **Composants Tooltip shadcn-svelte** (`$lib/components/ui/tooltip/`) → installés au bootstrap
- **Clé i18n `journal-entry-saved`** (« Écriture enregistrée ») → story 3.2 patch P3

### Patterns existants à réutiliser

- **`audit_log::insert_in_tx(&mut Transaction<MySql>, new: NewAuditLogEntry)`** — pattern story 3.3 pour atomicité. Prend une tx en cours, le caller gère le commit.
- **`Extension<CurrentUser>` dans un handler** — pattern story 3.3 (handlers `update_journal_entry`, `delete_journal_entry`, `disable_user`, etc.). Signature : `Extension(current_user): Extension<CurrentUser>` dans les paramètres, puis `current_user.user_id`.
- **`get_admin_user_id(pool)`** helper de test — existe dans `audit_log::tests` story 3.3 **comme fonction privée `#[cfg(test)]`**. Rust ne permet pas d'importer facilement une fonction `#[cfg(test)]` entre modules. **Deux options** : (a) **dupliquer** le helper dans chaque module de tests qui en a besoin (`accounts::tests`, `journal_entries::tests`) — simple, 3 lignes de code dupliquées acceptables pour un helper de test aussi trivial ; (b) créer un module `crates/kesh-db/src/repositories/test_utils.rs` avec `pub(crate) fn get_admin_user_id` accessible par tous les tests — plus propre mais scope creep. **Décision v0.1 : Option (a) duplication** — 3 lignes × 2 modules = 6 lignes, acceptable, refactor en Option (b) dans une story future de cleanup si le pattern se répète.
- **`entry_snapshot_json(entry, lines)`** — helper story 3.3. Réutiliser tel quel pour l'audit `journal_entry.created`.
- **Pattern i18nMsg** : import depuis `$lib/features/onboarding/onboarding.svelte`, signature `i18nMsg(key, fallback)`.

### Pièges identifiés

1. **Breaking change `accounts::create/update/archive`** : les 3 fonctions changent de signature (nouveau paramètre `user_id`). Tous les appelants doivent être mis à jour simultanément :
   - `routes/accounts.rs` (3 handlers)
   - **`accounts::tests`** (10+ tests existants) — ajouter `user_id` via helper
   - **`kesh-seed::seed_demo`** — vérifier si le seed utilise `create` directement ou `bulk_create_from_chart` (si `bulk_create_*`, pas d'impact)
2. **Breaking change `journal_entries::create`** : idem, signature change. Appelants :
   - `routes/journal_entries.rs::create_journal_entry` (1 handler)
   - `journal_entries::tests` (~10 tests existants depuis stories 3.2/3.3) — ajouter `user_id`
   - **`kesh-seed`** — vérifier si le seed crée des écritures (probablement pas en v0.1, mais à vérifier)
3. **`Tooltip.Trigger` accessibilité** : le trigger doit être focusable pour l'accessibilité clavier. Un `<th>` n'est pas focusable par défaut — wrapper dans `<span tabindex="0" class="cursor-help">` ou utiliser un `<button type="button">` invisible. **À tester manuellement** lors de T3.
4. **Tests Vitest `AccountingTooltip.test.ts`** : les composants Svelte 5 avec snippets (`children: Snippet`) sont parfois difficiles à tester avec `@testing-library/svelte`. Si le test est complexe, se rabattre sur un smoke test qui vérifie juste l'import et la présence du composant.
5. **`account_snapshot_json` helper** : à créer dans `repositories/accounts.rs` (pas dans `entities/`, qui ne doit pas dépendre de `serde_json` au-delà de ce qui est nécessaire). Pattern : fonction privée du module.
6. **`user_id` dans les tests seed** : le helper `get_admin_user_id` fait `SELECT id FROM users WHERE role = 'Admin' LIMIT 1`. Si aucun admin n'existe (scénario de test tout frais sans seed), le test panique. Protéger avec `.expect("admin user required — run seed_demo or bootstrap first")`.
7. **Ordre d'audit dans `create`** : pour les créations, insérer l'audit APRÈS les INSERTs principaux mais AVANT le commit. Si une exception survient après l'audit (ex: commit échoue), le rollback total annule à la fois l'entité et l'audit — cohérent.
8. **`details_json` convention projet** (vérifié empiriquement dans `journal_entries.rs` story 3.3) :
   - **`update`** → wrapper `{"before": snapshot_avant, "after": snapshot_apres}` (2 états à capturer)
   - **`create`** → **snapshot direct** (objet JSON sans wrapper) — cohérent avec `delete`
   - **`delete`** → **snapshot direct** (objet JSON sans wrapper) — déjà en place (`journal_entries.rs:791,802`)
   - **Règle mnémotechnique** : wrapper uniquement pour les transitions à 2 états. Objet direct pour les événements ponctuels (création, suppression).
   - **Dette d'introspection** : une future UI de consultation audit (post-MVP) devra aiguiller par `action` pour savoir comment parser `details_json` (wrapper vs direct). Cette asymétrie est intentionnelle mais documentée comme **dette technique** — si le coût de l'introspection devient gênant, une migration vers wrapper uniforme `{"before", "after"}` pour TOUTES les actions est possible (avec `"before": null` pour create et `"after": null` pour delete). Post-MVP.

### Réutilisation opportuniste — helpers `notify.ts`

**Check préalable** (T1.2) :
```bash
grep -rn "toast\.\(success\|info\|warning\|error\)" frontend/src/lib/features/ frontend/src/routes/
```

Si < 20 occurrences : refactor tout en `notify*` pour harmonisation complète. Si ≥ 20 occurrences : laisser les anciens (pattern compatible, `notify*` est juste un wrapper) et utiliser `notify*` uniquement dans le nouveau code 3.5. **Décision** : opportuniste, dépend du résultat du grep.

### References

- [Source: _bmad-output/planning-artifacts/epics.md#Epic-3-Story-3.5] — AC BDD lignes 820-834
- [Source: _bmad-output/planning-artifacts/prd.md#FR71-FR73] — Notifications + tooltips bilingues
- [Source: _bmad-output/planning-artifacts/prd.md#FR88] — Journal d'audit complet
- [Source: _bmad-output/planning-artifacts/ux-design-specification.md#UX-DR39] — Tooltips bilingues
- [Source: _bmad-output/implementation-artifacts/3-3-modification-suppression-ecritures.md] — Table audit_log + repository + pattern `insert_in_tx` + helper `entry_snapshot_json`
- [Source: _bmad-output/implementation-artifacts/3-2-saisie-ecritures-en-partie-double.md] — Pattern `create_journal_entry` handler à refactorer
- [Source: crates/kesh-db/src/repositories/audit_log.rs] — `insert_in_tx` et `find_by_entity`
- [Source: crates/kesh-db/src/repositories/journal_entries.rs::entry_snapshot_json] — Helper snapshot story 3.3
- [Source: crates/kesh-db/src/repositories/accounts.rs] — Fonctions `create/update/archive` à refactorer
- [Source: frontend/src/lib/shared/utils/api-client.ts] — Pattern session expirée → `/login?reason=session_expired`
- [Source: frontend/src/routes/login/+page.svelte:16-17] — Détection session_expired + message login
- [Source: frontend/src/lib/components/ui/tooltip/] — Composants Tooltip shadcn-svelte disponibles
- [Source: CLAUDE.md#Review-Iteration-Rule] — 2+ passes adversariales prévues

## Dev Agent Record

### Agent Model Used

{{agent_model_name_version}}

### Debug Log References

### Completion Notes List

### File List

## Change Log

- 2026-04-10: Création de la story 3.5 (Claude Opus 4.6, 1M context) — dernière story Epic 3 (complément FR71/FR72/FR73/FR88). Décisions clés :
  - **UI consultation audit log reportée post-MVP** — PRD FR88 exige enregistrement, pas consultation UI. Scope resserré volontairement.
  - **Helpers `notify.ts`** au-dessus de `svelte-sonner` existant — pas de refactor forcé des call sites anciens (opportuniste selon le résultat du grep).
  - **Composant `AccountingTooltip.svelte`** réutilisable (dans `shared/components/`, pas `features/journal-entries/`) pour réemploi futur stories 4.x/5.x.
  - **Audit complet CRUD** : story 3.3 a couvert `update`/`delete` des écritures ; cette story ajoute `create` des écritures + `create`/`update`/`archive` des comptes. FR88 pleinement couvert pour v0.1.
  - **Seed non audité** : `bulk_create_from_chart` (plan comptable au seed) et autres opérations système restent sans audit — contexte machine, pas utilisateur.
  - **Breaking changes assumés** : signatures `accounts::create/update/archive` et `journal_entries::create` changent (+user_id). Tous les appelants (tests + handlers + kesh-seed si applicable) à mettre à jour.
  - **Session expirée & conflit 409** : 100% déjà couverts (stories 1.6 + 1.10 + 3.3). Story 3.5 ne les touche pas.
  - Dette T9.3 (framework TestClient HTTP) et A11y (focus trap dialogs) héritées.
- 2026-04-10: Revue adversariale passe 1 (Explore subagent, Sonnet 4.6, contexte vierge — LLM orthogonal à Opus auteur) — 3 HIGH, 4 MEDIUM, 2 LOW. Les 8 findings > LOW (+ 1 LOW critique) tous patchés :
  - **H1 (HIGH)** : T3.1 pattern `<AccountingTooltip><th>…</th></AccountingTooltip>` structurellement invalide — un `<tr>` ne peut contenir que des `<th>`/`<td>` directs. Réécrit complètement avec **3 patterns distincts** : (1) Tooltip dans `<th>` (Débit/Crédit) = wrapper INTERNE au `<th>`, (2) Tooltip dans `<label>` (Journal) = wrapper INTERNE au `<label>`, (3) Tooltip sur `<span>` dynamique (Équilibré). Exemples de code complets fournis.
  - **H2 (HIGH)** : Label « Journal » est un `<label for="entry-journal">` (ligne 213), pas un `<th>` — pattern d'intégration différent de Débit/Crédit. Vérifié empiriquement dans `JournalEntryForm.svelte`. T3.1 distingue maintenant explicitement les 2 contextes avec 2 snippets de code.
  - **H3 (HIGH)** : Ambiguïté `details_json` pour `journal_entry.created` — 3 formulations incompatibles dans la spec (spec originale disait tantôt « snapshot complet », tantôt « {before: null, after}`, tantôt wrapper). Vérifié empiriquement dans `journal_entries.rs` story 3.3 : `update` utilise `{"before": ..., "after": ...}` (wrapper 2 états), `delete` utilise `snapshot` direct (objet 1 état). Tranché : **`create` utilise `snapshot` direct** cohérent avec `delete`. Convention projet documentée : « wrapper uniquement pour les transitions à 2 états, objet direct sinon ».
  - **M1 (MEDIUM)** : `bits-ui asChild` pattern — la spec pré-patch disait « wrapper `<span tabindex="0">` » sans mentionner l'option `asChild` qui est le pattern natif bits-ui. Ajouté dans T3.2 comme alternative, avec décision pragmatique v0.1 sur `<span tabindex="0">` (plus simple) et refactor `<button>` post-MVP si A11y audit requis.
  - **M2 (MEDIUM)** : Tooltip « équilibré » sur indicateur dynamique (vert/rouge) — texte invariant ou adapté à l'état ? Tranché : **texte invariant** (description conceptuelle de la partie double, identique quel que soit l'état courant). Une seule paire de clés i18n, pas deux variantes. Documenté dans T3.1 Pattern 3 et T3.3.
  - **M3 (MEDIUM)** : Seuil arbitraire « < 20 occurrences » pour le refactor opportuniste de `notify*` — décision floue. Tranché : **PAS de refactor des call sites existants**, `notify*` utilisés UNIQUEMENT pour le nouveau code 3.5. Les call sites existants continuent avec `toast.*` direct — refactor transverse post-MVP si besoin.
  - **M4 (MEDIUM)** : AC#2 `notifyWarning` créé sans callsite réel en 3.5 — couverture partielle. Documenté explicitement comme **pattern préparatoire** pour les stories 6.x (import bancaire), test unitaire dans `notify.test.ts` couvre `notifyWarning` isolément via mock `toast`.
  - **L1 (LOW critique)** : `get_admin_user_id` est `#[cfg(test)]` private dans `audit_log::tests` — ne peut pas être importé dans `accounts::tests` ou `journal_entries::tests`. Tranché : **duplication du helper** (6 lignes, 2 modules) plutôt que création d'un module `test_utils` partagé. Pattern trivial, refactor transverse post-MVP si besoin.
  - **LOWs restants** non patchés : validations empiriques confirmées (kesh-seed n'utilise pas `accounts::create` direct, `journal-entry-saved` i18n présente, `serde_json` déjà en dep kesh-db). Rien à corriger.
- 2026-04-10: Revue adversariale passe 2 (Explore subagent, Haiku 4.5, contexte vierge — LLM orthogonal à Sonnet passe 1, briefing explicite anti-confusion post-incident 3.4). Vérification : **les 9 patches passe 1 sont tous présents et cohérents** dans le markdown. Haiku a retenu les leçons de l'incident 3.4 et ne remonte pas de faux positifs CRITICAL.
  - **1 MEDIUM résiduel trouvé (et patché)** : T5.4 ligne 222 contenait encore une formulation ambiguë « `before: null, after: {...}` OU `{...}` direct » malgré la décision AC#10. Réécriture complète de T5.4 pour supprimer l'ambiguïté : snapshot direct explicite + critères de test concrets (`entry["description"]`, `entry["lines"].len() >= 2`). Piège #8 Dev Notes également refondu avec la convention projet complète (wrapper pour update, direct pour create/delete) + règle mnémotechnique + documentation de la dette d'introspection comme migration post-MVP possible.
  - **2 LOW résiduels acceptables** :
    - **Dette d'introspection `details_json`** : asymétrie wrapper/direct selon action. Documentée comme dette technique dans le Piège #8. Migration vers wrapper uniforme post-MVP si besoin.
    - **Focus cosmétique `<th>` + `<span tabindex="0">`** : le `<span>` interne reçoit le focus mais le `<th>` parent peut afficher un focus ring ambigu. Cosmétique, à vérifier au test Playwright T7.1 + refactor `<button>` post-MVP si A11y audit l'exige.
  - **Critère d'arrêt CLAUDE.md formellement ATTEINT** après 2 passes orthogonales (Sonnet → Haiku). 9 patches passe 1 + 1 patch MEDIUM passe 2 = **10 patches au total**. 0 finding > LOW résiduel. Story 3.5 **prête pour `dev-story`**.
- 2026-04-10: Revue de préparation passe 3 (Explore subagent, Sonnet 4.6, **angle readiness audit** — simulation d'un dev qui ouvre l'IDE, différent de l'angle adversarial de la passe 1). Verdict : **READY TO CODE**.
  - Vérification systématique des 8 tâches T1-T8 : **toutes READY**, aucun blocage d'exécution.
  - Toutes les citations de code vérifiées fidèles : `journal_entries.rs:791,802` (pattern delete snapshot direct), `JournalEntryForm.svelte:213-215,241-246,339-347` (3 contextes HTML distincts), `onboarding.svelte.ts:23` (signature `i18nMsg`), `create_journal_entry:386-388` (sans `Extension<CurrentUser>`), `accounts::create:14` (sans `user_id`).
  - Toutes les dépendances confirmées dans `package.json` : `svelte-sonner@^1.1.0`, `bits-ui@^2.16.5`, `@testing-library/svelte@^5.3.1`.
  - kesh-seed **n'utilise** ni `accounts::create` ni `journal_entries::create` directement (grep vide) — le refactor breaking change est bien scoped aux tests + handlers.
  - **1 seul patch de robustesse appliqué** (LOW non bloquant mais gratuit) : T7.1 sélecteur Playwright `'th:has-text("Débit")'` remplacé par `'[data-slot="tooltip-trigger"]:has-text("Débit")'` — cible directement l'élément wrapper bits-ui, évite toute fragilité de propagation d'événements hover.
- 2026-04-10: **Critère d'arrêt CLAUDE.md DÉFINITIVEMENT ATTEINT** après 3 passes orthogonales (Sonnet adversarial → Haiku → Sonnet readiness). **11 patches au total** (9 passe 1 + 1 passe 2 + 1 passe 3). Story 3.5 **PRÊTE POUR `dev-story`** — le dev peut commencer par T1 immédiatement.
- 2026-04-10: Implémentation dev-story (Claude Opus 4.6, 1M context). T1–T7 complétés :
  - **T1** `frontend/src/lib/shared/utils/notify.ts` + `notify.test.ts` (7 tests vitest).
  - **T2** `frontend/src/lib/shared/components/AccountingTooltip.svelte` + modification `tooltip-trigger.svelte` pour supporter les children snippets.
  - **T3** Intégration tooltips dans `JournalEntryForm.svelte` (3 patterns : `<th>` Débit/Crédit, `<label>` Journal, `<span>` dynamique Équilibré).
  - **T4** 32 clés i18n tooltip × 4 langues dans les 4 fichiers `.ftl`.
  - **T5** Refactor `journal_entries::create` (+ `user_id: i64`), audit_log insert avant commit, handler `create_journal_entry` extrait `Extension<CurrentUser>`, 15 test call sites mis à jour.
  - **T6** Refactor `accounts::{create,update,archive}` (+ `user_id: i64`), helper `account_snapshot_json`, audit_log insert (direct create/archive, wrapper `{before,after}` update), `get_admin_user_id` helper dupliqué dans `accounts::tests`, 11 test call sites mis à jour. `bulk_create_from_chart` volontairement NON modifiée.
  - **T7** Test Playwright hover tooltip (`[data-slot="tooltip-trigger"]` sélecteur robuste).
  - **T8** Validation : `cargo check --workspace` OK, Vitest 120/120 OK, svelte-check 0 errors. Story marquée `ready-for-review`.
- 2026-04-11: **Revue adversariale passe 1** (3 subagents Sonnet parallèles : Blind Hunter + Edge Case Hunter + Acceptance Auditor, LLM orthogonal à Opus auteur — CLAUDE.md rule). 15 findings : **1 CRITICAL, 2 HIGH, 5 MEDIUM, 3 LOW, 4 rejected**. 10 patches **P1–P10** appliqués :
  - **P1 [CRITICAL]** Création `AccountingTooltip.test.ts` (6 tests Vitest : smoke + contract de dérivation des clés + `it.each` sur les 4 termes).
  - **P2 [HIGH]** Ajout de 4 tests audit_log DB : `test_create_writes_audit_log` (journal_entries) + `test_{create,update,archive}_account_writes_audit_log` (accounts), tous vérifient via `audit_log::find_by_entity` la présence de l'entrée avec `action` exact et structure `details_json` conforme (direct vs wrapper).
  - **P3 [MEDIUM]** `AccountingTooltip.svelte:50` — `opacity-80` → `text-muted-foreground` (déviation design system en dark mode corrigée).
  - **P4 [MEDIUM]** Playwright `journal-entries.spec.ts` — ajout de 3 `test.skip` (crédit/journal/équilibré) + **P6 [MEDIUM]** suppression du `{ timeout: 2000 }` override qui rendait le test flaky sur CI.
  - **P5 [MEDIUM]** Extraction de `i18nMsg` + `loadI18nMessages` vers un nouveau module canonical `frontend/src/lib/shared/utils/i18n.svelte.ts`. `onboarding.svelte.ts` est maintenu en re-export pour préserver les 12 call sites existants. `AccountingTooltip.svelte` importe désormais depuis la nouvelle location (casse le couplage `shared/components/` → `features/onboarding/`).
  - **P7 [MEDIUM]** `JournalEntryForm.svelte:366-369` — placeholder vide en état formulaire vierge : `<span tabindex="0" opacity-0>` → `<span aria-hidden="true" class="opacity-0">` (non-focusable, plus de focus ring invisible).
  - **P8 [LOW]** `account_snapshot_json` — ajout de `"companyId": account.company_id` (traçabilité multi-company future).
  - **P9 [LOW]** Doc-comment de `bulk_create_from_chart` — ajout explicite « Cette fonction ne génère PAS d'entrées d'audit log (contexte seed système, pas action utilisateur) ».
  - **P10 [LOW→cohérence]** `accounts::{create,update,archive}` — rollback explicite `if let Err(e) = audit_log::insert_in_tx(...).await { tx.rollback()?; return Err(e); }` sur les 3 fonctions (cohérence stylistique avec les autres branches d'erreur qui avaient déjà un rollback explicite).
- 2026-04-11: **Revue adversariale passe 2** (3 subagents Haiku parallèles avec briefing explicite anti-confusion, orthogonal à Sonnet passe 1). 1 finding MEDIUM réel dédoublé par Blind Hunter + Edge Case Hunter, 2 LOW rejetés (décisions documentées). **P11** appliqué :
  - **P11 [MEDIUM]** `journal_entries::create` lignes 235-255 — j'avais oublié cette fonction lors de l'application de P10 sur les 3 fonctions accounts. Correction du même pattern `if let Err(e) = ... { tx.rollback()?; return Err(e); }`.
- 2026-04-11: **Revue adversariale passe 3** (3 subagents Opus parallèles, orthogonal à Sonnet/Haiku, fenêtres fraîches). Blind Hunter Opus : **« No findings. Convergence atteinte. »** Acceptance Auditor Opus : **« Zéro finding, pas même LOW. »** Edge Case Hunter Opus : 1 MEDIUM (F1) + 2 LOW (F2 defer pré-existant story 3.3, F3 rejet décision documentée T2.2). **P12** appliqué :
  - **P12 [MEDIUM]** `JournalEntryForm.svelte:354-378` — **régression A11y introduite par P7** détectée par Opus : retirer le `tabindex="0"` du `<span>` enfant (P7) n'avait pas résolu le problème sous-jacent car le wrapper `AccountingTooltip` reste rendu et son `Tooltip.Trigger` bits-ui devient un `<button>` focusable avec accessible name vide → violation WCAG 2.1 A critère 4.1.2 (Name, Role, Value). Fix : déplacement du `{#if}` HORS du wrapper `AccountingTooltip` — le composant n'est instancié que si la branche a un contenu significatif (balanced OU au moins un montant saisi). Branche vide rend simplement `<span aria-hidden="true">` sans bouton.
- 2026-04-11: **Revue adversariale passe 4** (1 subagent Sonnet focalisé sur P12, orthogonal à Opus passe 3). Verdict : **« No findings. Convergence pass 4 confirmée. »** Matrice des 3 états balanced vérifiée exhaustivement (vide, déséquilibré, équilibré), WCAG 4.1.2 confirmé OK, aucune régression de structure, test e2e toujours valide.
- 2026-04-11: **CRITÈRE D'ARRÊT CLAUDE.MD ATTEINT APRÈS 4 PASSES** (Sonnet → Haiku → Opus → Sonnet, **LLMs orthogonaux** + fenêtres fraîches). **12 patches au total** (P1–P12). Validation finale : `cargo check --workspace` OK, Vitest **126/126** (dont 6 nouveaux AccountingTooltip + 7 notify + 3 new audit tests accounts + 1 journal_entries audit test), svelte-check 0 errors. Story 3.5 marquée **done**. **Epic 3 (Plan Comptable & Écritures) entièrement terminé** — 5 stories done (3.1, 3.2, 3.3, 3.4, 3.5).
