# Story 36-15 : « compte analytique » au lieu de « projet analytique » (#1161)

Status: ready-for-dev

*Fiche rédigée le 2026-11-15 par l'agent de rédaction des fiches de l'Epic 36, phase 2 (Opus 5.5), worktree `kesh-36-15`,
branche `story/36-15-compte-analytique`, par le workflow `bmad-create-story`. Release **0.18.0**. **Aucune migration**,
aucune route, aucun champ JSON, aucun code d'erreur ne change. Code lu à **`0bf3c3b4b`** (`origin/epic/36-0.18.0`, le
kickoff de la 0.18.0). Registres : `e36-choix-autonomes-e360.md` (C-e360-15, qui fait foi) ;
`e36-choix-autonomes-36-15-compte-analytique.md` pour la story (C-36-15-1 à 6). Voie **1**. Validation non lancée.*

## Story

En tant que titulaire qui suit dans Kesh un immeuble, un véhicule, une activité ou un chantier, je veux que l'axe
analytique s'appelle partout **compte analytique** — écrans, messages, rapports, exports, manuels, dans les quatre
langues —, afin que le nom dise ce que l'objet permet de suivre, et non le seul cas du projet, sans que rien ne change de
ce que Kesh enregistre ni de ce que l'API reçoit et rend.

## Ce qui se lit au code (`0bf3c3b4b`)

**L'objet ne change pas.** La table `projects` (`20260702000001_projects_analytics.sql`), la route `/api/v1/projects`
(`crates/kesh-api/src/routes/projects.rs`), les champs `projectId` / `defaultProjectId`, les codes `PROJECT_ARCHIVED` /
`PROJECT_NOT_FOUND` (`crates/kesh-api/src/routes/reconciliation.rs:765`, `:770`), l'écran `/settings/projects`
(`frontend/src/routes/(app)/+layout.svelte:130` : `{ i18nKey: 'nav-projects', fallback: 'Projets analytiques', href:
'/settings/projects' }`) restent tels quels (C-e360-15). Seuls les **textes** changent.

**Le frontend n'a pas de catalogue propre.** `i18nMsg` lit `_messages[key] || fallback`
(`frontend/src/lib/shared/utils/i18n.svelte.ts:20` : `const raw = _messages[key] || fallback;`), servis par le backend
depuis `crates/kesh-i18n/locales/<locale>/messages.ftl`. Chaque site porte donc **deux** textes : la valeur au catalogue
et le repli français du site.

### Inventaire par le symptôme

Détecteur par lecture qui part du **mot**, non d'une liste de clés ; chaque occurrence classée « sens analytique » (dans
le périmètre) ou « sens courant » (hors périmètre, ligne citée). Sonde positive : chaque commande rend au moins un site
connu (`nav-projects`, `projects-title`), donc elle lit bien le fichier.

**1. Les quatre catalogues** — commande :

```sh
for l in de-CH:Projekt it-CH:progett en-CH:project; do loc=${l%%:*}; w=${l##*:}; f=crates/kesh-i18n/locales/$loc/messages.ftl
  /usr/bin/grep -niE "= .*$w|^#.*$w" $f | wc -l; done
git grep -n -iE "projet" -- crates/kesh-i18n/locales/fr-CH/messages.ftl | wc -l
```

Sortie : `de-CH 49`, `it-CH 49`, `en-CH 50`, `fr-CH 50`. Ventilation :
- **fr-CH, 50** = **49 valeurs** + le commentaire de convention `messages.ftl:3089`
  (`# « projet analytique » = `Projekt`/`progetto`/`project` (on ne traduit pas « analytique »),`).
- **de-CH, it-CH : 49 valeurs**, les mêmes 49 clés (relevé clé par clé, table de D2).
- **en-CH, 50** = les 49 valeurs + **un faux positif** : `en-CH/messages.ftl:2423`
  `dunning-example-heading = Projected schedule` — « projeté », sans rapport : **hors périmètre**.
- Contrôle de l'autre sens (clés `*project*` dont la valeur ne dit pas « projet ») :
  `/usr/bin/grep -nE '^[a-z0-9-]*project[a-z0-9-]* =' fr-CH/messages.ftl | /usr/bin/grep -viE '= .*projet'` rend 32
  lignes, toutes neutres (« — Aucun », « Compte », « Montant », « Sous-total », « Archiver », « Code »…) :
  `reports-project-expenses-col-account = Compte` désigne le **compte du plan** et reste « Compte ».
- Aucune valeur ne dit « analytique » sans « projet » (`/usr/bin/grep -niE 'analyti' fr-CH/messages.ftl | /usr/bin/grep -vi projet` vide).

Les 49 clés (fr-CH, ligne) : `journal-entry-form-col-project` :296, `journal-entry-project-archived` :297,
`reconciliation-failed-project-archived` :1554, `reconciliation-failed-project-not-found` :1555,
`reconciliation-manual-project-label` :1835, `reconciliation-rules-default-project-archived` :1837,
`reconciliation-rules-labels-default-project` :1839, `reconciliation-split-th-project` :1848, `reports-project-expenses`
:1944, `reports-project-expenses-empty` :1947, `reports-project-expenses-title` :1949, `reports-project-return` :1954,
`reports-project-return-col-project` :1957, `reports-project-return-empty` :1960, `reports-project-return-title` :1961,
`reports-project-selector-label` :1963, `reports-project-selector-placeholder` :1964, `reports-filename-project-expenses`
:1985, `reports-filename-project-return` :1986, `export-global-description` :2267, `export-global-content-includes` :2273,
`invoice-detail-project-label` :2659, `invoice-field-project` :2660, `invoice-project-archived` :2661,
`invoice-project-current` :2662, `supplier-invoices-detail-project` :2842, `supplier-invoices-field-project` :2855,
`nav-projects` :3116, `projects-archived` :3171, `projects-created` :3173, `projects-empty` :3175, `projects-err-code`
:3176, `projects-err-name` :3177, `projects-field-parent` :3182, `projects-form-create` :3184, `projects-form-edit` :3185,
`projects-load-error` :3186, `projects-new` :3187, `projects-parent-none` :3188, `projects-show-archived` :3191,
`projects-subtitle` :3192, `projects-title` :3193, `projects-unarchived` :3195, `projects-updated` :3196,
`audit-log-entity-project` :3259, `audit-log-action-project-created` :3362, `-updated` :3363, `-archived` :3364,
`-unarchived` :3365.

**2. Le frontend** — détecteur exécuté (script de lecture, commentaires masqués par `masquerCommentaires`
de `frontend/src/lib/shared/i18n-literal-reader.js:261`, qui masque aussi les `<!-- … -->` des `.svelte`), sur
`frontend/src/**/*.{svelte,ts,js}` hors `*.test.ts`, motif `/projet/i` :

```
46
src/lib/components/invoices/InvoiceForm.svelte:814: {i18nMsg('invoice-field-project', 'Projet analytique (optionnel)')}
src/lib/components/invoices/InvoiceForm.svelte:830: ? i18nMsg('invoice-project-archived', 'Projet archivé')
src/lib/components/invoices/InvoiceForm.svelte:831: : i18nMsg('invoice-project-current', 'Projet actuel')}
src/lib/features/journal-entries/JournalEntryForm.svelte:409: {i18nMsg('journal-entry-form-col-project', 'Projet')}
src/lib/features/journal-entries/JournalEntryForm.svelte:461: aria-label={i18nMsg('journal-entry-form-col-project', 'Projet')}
src/lib/features/journal-entries/JournalEntryForm.svelte:470: {i18nMsg('journal-entry-project-archived', 'Projet archivé')}
src/lib/features/reconciliation/ManualMatchModal.svelte:230: {i18nMsg('reconciliation-manual-project-label', 'Projet analytique (optionnel)')}
src/lib/features/reconciliation/TransactionSplitModal.svelte:389: {i18nMsg('reconciliation-split-th-project', 'Projet')}
src/lib/features/reconciliation/failed-proposal-label.ts:250: return i18nMsg('reconciliation-failed-project-archived', 'Le projet analytique est archivé.');
src/lib/features/reconciliation/failed-proposal-label.ts:252: return i18nMsg('reconciliation-failed-project-not-found', 'Projet analytique introuvable.');
src/lib/features/reconciliation/rules/RuleFormModal.svelte:241: {i18nMsg('reconciliation-rules-labels-default-project', 'Projet analytique par défaut')}
src/lib/features/reconciliation/rules/RuleFormModal.svelte:259: {i18nMsg('reconciliation-rules-default-project-archived', 'Projet archivé')}
src/lib/features/reports/ProjectExpensesView.svelte:29: {i18nMsg('reports-project-expenses-title', 'Dépenses par projet')} — {report.project.code}
src/lib/features/reports/ProjectExpensesView.svelte:38: {i18nMsg('reports-project-expenses-empty', 'Aucune dépense taguée sur ce projet pour la période.')}
src/lib/features/reports/ProjectReturnView.svelte:23: {i18nMsg('reports-project-return-title', 'Rendement par projet')} — {report.project.code}
src/lib/features/reports/ProjectReturnView.svelte:30: {i18nMsg('reports-project-return-empty', 'Aucun mouvement tagué sur ce projet pour la période.')}
src/lib/features/reports/ProjectReturnView.svelte:37: <th class="py-1">{i18nMsg('reports-project-return-col-project', 'Projet')}</th>
src/lib/features/supplier-invoices/SupplierInvoiceForm.svelte:759: {i18nMsg('supplier-invoices-field-project', 'Projet analytique (optionnel)')}
src/routes/(app)/+layout.svelte:130: { i18nKey: 'nav-projects', fallback: 'Projets analytiques', href: '/settings/projects' },
src/routes/(app)/export/+page.svelte:67: "Exportez toute votre comptabilité au format CSV dans un fichier ZIP : plan comptable, écritures, factures de
src/routes/(app)/export/+page.svelte:76: "L'export contient 30 tables : plan comptable, exercices, écritures et leurs lignes, contacts et personnes de
src/routes/(app)/invoices/[id]/+page.svelte:1282: {i18nMsg('invoice-detail-project-label', 'Projet analytique')}
src/routes/(app)/journal-entries/[id]/+page.svelte:611: <th class="py-2 pr-2">Projet</th>
src/routes/(app)/reports/+page.svelte:433: activeTab === 'project-expenses' ? 'depenses-par-projet' : 'rendement-par-projet';
src/routes/(app)/reports/+page.svelte:527: fallback: 'Dépenses par projet',
src/routes/(app)/reports/+page.svelte:532: fallback: 'Rendement par projet',
src/routes/(app)/reports/+page.svelte:714: {i18nMsg('reports-project-selector-label', 'Projet')}
src/routes/(app)/reports/+page.svelte:720: <option value={null}>{i18nMsg('reports-project-selector-placeholder', '— Choisir un projet')}</option>
src/routes/(app)/settings/projects/+page.svelte:55: … 'Impossible de charger les projets.'   (:97, :101, :116, :119, :136, :139, :159, :164,
                                                    :168, :173, :179, :193, :194, :210, :212, :252 : les 16 autres replis `projects-*`)
src/routes/(app)/supplier-invoices/[id]/+page.svelte:813: <dt …>{i18nMsg('supplier-invoices-detail-project', 'Projet analytique')}</dt>
```

**46 sites, tous au sens analytique** : 44 replis `i18nMsg`, le repli de nom de fichier `reports/+page.svelte:433`, et
**un libellé en dur** : `journal-entries/[id]/+page.svelte:611` (`<th class="py-2 pr-2">Projet</th>`, voisin de
`<th class="py-2 pr-2">Compte</th>` à `:609`, lui aussi en dur — la famille que porte #339). Les commentaires et
doc-comments qui disent « projet » (`InvoiceForm.svelte:165`, `projects.api.ts:2`, `reports.types.ts:186`…, 93 lignes de la
recherche brute `git grep -n -iE "projet" -- frontend/src ':!*.test.ts'`, qui en rend 139) et les **sens courants**
(« Pattern projet », « convention projet » : `accounts.api.ts:37`, `api-keys.api.ts:37`, `DeleteBankImportAction.svelte:124`,
`WriteOffDialog.svelte:121`, `clipboard.ts:8`) sont des commentaires : hors périmètre (D6).

**Tests et E2E qui lisent l'ancien texte** — `git grep -n -iE "projet|projekt|progett" -- 'frontend/src/**/*.test.ts' frontend/tests` :
aucun Vitest n'affirme un libellé (les assertions relevées ne portent que des **données** : `name: 'Projet un'`,
`journal-entry-page.test.ts:61`). **Deux sélecteurs E2E** lisent le libellé et rougiront :
`frontend/tests/e2e/reports.spec.ts:81` (`await page.getByRole('tab', { name: /dépenses par projet/i }).click();`) et
`:111` (`/rendement par projet/i`). `projects.spec.ts:30-37` ne vise le titre que par son rôle (KF-043) : inchangé. Les noms
de données des E2E (« Projet E2E 19-4 »…) sont des données, inchangés.

**3. Le backend — textes visibles.** `git grep -n -iE '"[^"]*projet[^"]*"' -- 'crates/*.rs' ':!crates/*/tests/*'` :
- `crates/kesh-api/src/routes/projects.rs` — **11 messages `AppError::Validation`**, servis en `error.message` et
  **affichés tels quels** par l'écran (`settings/projects/+page.svelte:124-125` : `formError = isApiError(err)` / `? err.message` ;
  `:144` pour l'archivage : `isApiError(err) ? err.message : …`) : `:107` « Le code du projet est requis. », `:111` « … dépasse 32 caractères. », `:115`,
  `:119`, `:154` « Un projet ne peut pas être son propre parent. », `:159` « Le projet parent est introuvable. », `:162`
  « Un sous-projet ne peut pas être parent (hiérarchie limitée à 2 niveaux). », `:169` et `:344` « Le projet parent est
  archivé — désarchivez-le d'abord. », `:176` « Ce projet a des sous-projets : … », `:335` « Ce projet a des sous-projets
  actifs — archivez-les d'abord. » — **dans le périmètre** (D4).
- `crates/kesh-report/src/pdf.rs:1188` (`title: "Dépenses par projet".into(),`), `:1192` (`empty_message: "Aucune dépense
  taguée sur ce projet pour la période."`), `:1309` (`title: "Rendement par projet".into(),`), `:1315` — libellés du PDF,
  en dur en fr-CH (`:1173-1174` : « labels dédiés en dur FR-CH, i18n PDF déférée v0.2 »), passés par
  `crates/kesh-api/src/routes/reports.rs:628` et `:723` (`&ProjectExpensesPdfLabels::fr_ch_defaults()`) — **dans le
  périmètre** : c'est le titre du rapport imprimé.
- **Le nom du fichier exporté** se résout par la clé `reports-filename-{report_type}` (`reports.rs:955` :
  `let key = format!("reports-filename-{report_type}");`) : il suit le catalogue, rien à toucher dans le code.

**4. Le backend — textes non visibles, hors périmètre (D6)** :
- `crates/kesh-report/src/csv.rs:547-548` et `:616-617` : en-têtes CSV `"Projet"`, `"SousProjet"` (à côté de
  `"NumeroCompte"`, `"CoutInvesti"`) — des **noms de colonnes** sans espace ni accent, lus par les tableurs et les macros ;
  affirmés par `crates/kesh-api/tests/project_reports_e2e.rs:218` (`text.contains("Projet;SousProjet;NumeroCompte")`) et
  `:350`. Gardés (D5).
- `DbError::IllegalStateTransition("le projet analytique est archivé")` : `crates/kesh-db/src/repositories/projects.rs:119`,
  `invoices.rs:2147`, `supplier_invoices.rs:506`. Le mappage HTTP ne le montre pas (`crates/kesh-api/src/errors.rs:4676-4682` :
  `tracing::warn!("illegal state: {m}")`, puis le texte `t("error-illegal-state", …)`) ; il ne sort qu'en
  `failed[].details.message` du lot de réconciliation (`reconciliation.rs:754` : `obj.insert("message".to_string(), serde_json::json!(msg));`), et **la réconciliation le reconnaît
  par son texte** : `reconciliation.rs:763` (`DbError::IllegalStateTransition(msg) if msg.contains("projet") =>`). Le
  changer toucherait `kesh-db` et ce couplage : **gardé**.
- `crates/kesh-report/src/errors.rs:23` (`#[error("projet analytique introuvable (project_id={project_id})")]`) : mappé
  404 sans son texte (`errors.rs:1319-1321`). Gardé.
- `crates/kesh-api/src/exports/global.rs:529` (`push_csv!("projects.csv", …)`) : nom de fichier du ZIP, identifiant. Gardé.
- Le serveur MCP rend `projet_id` (`crates/kesh-api/src/mcp/outils/ecritures.rs:128`, `factures.rs:265`) : nom de champ
  d'un texte lu par un agent, contrat du prompt. Gardé.
- Données de test (`"Projet A"`, `"Loyer projet"`…) : données. Gardées.

**5. Les manuels** — `git grep -n -iE 'projet' -- docs/manual/fr/*.tex` (sans `\includegraphics` d'un écran de projets :
`git grep -n -iE "includegraphics.*(proj|analyt)" -- docs/manual` vide ; `docs/manual/{de,en,it}/` ne portent qu'un
`README.md`) :
- `user-manual.tex`, **30 lignes, toutes analytiques** : `:748`, `:1755`, `:1880`, `:1882`, `:1883`, `:1890`, `:3170`,
  `:3229`, `:3253`, `:3756`, la section `\section{Comptabilité analytique par projet}` `:3767` à `:3804` (20 lignes, dont
  le chemin « \emph{Administration} → \emph{Projets analytiques} » `:3775` et les deux rapports `:3800`, `:3804`), `:3826`.
- `admin-manual.tex`, **17 lignes, 2 analytiques** : `:2787` (« (projet compris ; … ») et `:3509` (« Les rapports de TVA,
  des journaux et des projets … ne sont pas servis à l'agent »). Les **15 autres** sont le **projet Docker Compose** ou
  Container Manager (`:602`, `:608`, `:612`, `:617`, `:1535`, `:1676`, `:1766`, `:1975`, `:2001`, `:2087`, `:2167`,
  `:2198`, `:2509`, `:2921`) et `--scope project` de Claude Code (`:3616`) : sens courant, hors périmètre.
- `marketing-brochure.tex`, **2 lignes** : `:195` (« les achats, les avoirs et les projets analytiques y seront
  ajoutés ») analytique ; `:498` (« la documentation projet ») courant.

**6. Documentation et site** :
- `docs/api-external.md`, 5 lignes : `:108` (« rapports de projet », « projets »), `:325` (« Projet inconnu / archivé (un
  projet **déjà présent** … ») , `:633` (deux), `:691`, `:901` — toutes analytiques ; aucune phrase ne dit que `project`
  désigne l'axe analytique (C-e360-15 la demande).
- `docs/i18n-glossaire.md` : `:105` (« projet parent »), `:168-169` (« projet », « projet analytique »), la note
  `:172-189` (l'arbitrage du 2026-08-19 : « on ne traduit pas « analytique » », `Kostenstelle` écarté), `:224` (historique).
- `README.md`, 5 lignes : `:47` (**Fonctionnalités** : « **Comptabilité analytique par projet** — dimension « projet » … »)
  analytique et courant ; `:218` (tableau des versions, ligne de l'E19, historique) ; `:19`, `:106` (« Structure du
  projet »), `:201` (« Le projet suit … BMAD ») : sens courant.
- `website/` : `git grep -n -iE 'projet analytique|projets analytiques|par projet|analytique' -- website` vide ; en
  anglais, la seule mention est la carte historique de l'E19 (`website/roadmap.html:249-252`, « E19 — Project cost
  accounting », « Expenses by project »).

## Décisions

- **D1 — Le terme français : « compte analytique », toujours en entier** (CR #1161, recommandation 2). Le champ, la
  colonne, l'option et le titre disent « Compte analytique », jamais « Compte » seul — « Compte » reste le compte du plan
  (`reports-project-expenses-col-account = Compte`, `journal-entries/[id]/+page.svelte:609`). Hiérarchie : **compte
  analytique parent**, **sous-compte analytique**, **compte analytique racine**. Les rapports : **Dépenses par compte
  analytique**, **Rendement par compte analytique** ; la section du manuel : **Comptabilité analytique**. Le genre ne
  change pas (« projet » et « compte » sont masculins) : les accords (« archivé », « créé », « actuel ») restent.
  *Écartée* : « axe analytique » ou « centre de coûts » (le CR a tranché).

- **D2 — Les traductions, fixées une fois** (CR, recommandation 3 ; C-36-15-1) :

  | fr-CH | de-CH | it-CH | en-CH |
  |---|---|---|---|
  | compte analytique | **Kostenstelle** (f.) | **conto analitico** | **analytical account** |
  | comptes analytiques | Kostenstellen | conti analitici | analytical accounts |
  | compte analytique parent | übergeordnete Kostenstelle | conto analitico superiore | parent analytical account |
  | sous-compte analytique | Unterkostenstelle | sottoconto analitico | analytical sub-account |
  | compte analytique racine | Hauptkostenstelle | conto analitico radice | root analytical account |

  *Source* : aucun précédent au catalogue (`git grep -n -iE "kostenst|kostentr|conto analitico|analytical account"` ne
  rend que la note du glossaire, `docs/i18n-glossaire.md:176-186`). `Kostenstelle` / `conto analitico` / *analytical
  account* sont les termes que le CR propose et ceux des logiciels de comptabilité suisses et européens pour l'axe
  analytique générique ; l'anglais garde l'adjectif *analytical* que le catalogue emploie déjà (`en-CH:1478`, « The
  analytical project is archived. »). **Cette décision remplace l'arbitrage du 2026-08-19** (glossaire `:172-189`), qui
  écartait `Kostenstelle` parce qu'un *projet* répond à « pour quoi ? » et non à « où ? » : la prémisse a changé — l'objet
  sert désormais à un immeuble, un véhicule, une activité, un centre de coûts (CR #1161), c'est-à-dire aux deux.
  *Écartées* : `Kostenträger` (le « pour quoi ? » de 2026-08-19 : trop étroit à son tour pour un centre de coûts ou un
  immeuble) ; `Analysekonto` (calque, et collision avec `Konto` du plan sans le bénéfice d'un terme consacré) ;
  `centro di costo` / *cost centre* (ne portent que des coûts, alors que le rendement compte des revenus) ; garder
  `Projekt` / `progetto` / *project* (le défaut même que le CR corrige). ⚠️ Point remis à l'orchestrateur : la décision
  défait un arbitrage de Guy, sur la base du CR de Guy qui la demande ; elle se lit au rapport.

  **Valeurs, clé par clé** (fr | de | it | en). Les deux clés d'export ne changent qu'un mot de leur liste.

  | Clé | fr-CH | de-CH | it-CH | en-CH |
  |---|---|---|---|---|
  | `journal-entry-form-col-project` | Compte analytique | Kostenstelle | Conto analitico | Analytical account |
  | `journal-entry-project-archived` | Compte analytique archivé | Kostenstelle archiviert | Conto analitico archiviato | Analytical account archived |
  | `reconciliation-failed-project-archived` | Le compte analytique est archivé. | Die Kostenstelle ist archiviert. | Il conto analitico è archiviato. | The analytical account is archived. |
  | `reconciliation-failed-project-not-found` | Compte analytique introuvable. | Kostenstelle nicht gefunden. | Conto analitico introvabile. | Analytical account not found. |
  | `reconciliation-manual-project-label` | Compte analytique (optionnel) | Kostenstelle (optional) | Conto analitico (facoltativo) | Analytical account (optional) |
  | `reconciliation-rules-default-project-archived` | Compte analytique archivé | Kostenstelle archiviert | Conto analitico archiviato | Analytical account archived |
  | `reconciliation-rules-labels-default-project` | Compte analytique par défaut | Standard-Kostenstelle | Conto analitico predefinito | Default analytical account |
  | `reconciliation-split-th-project` | Compte analytique | Kostenstelle | Conto analitico | Analytical account |
  | `reports-project-expenses`, `-expenses-title` | Dépenses par compte analytique | Ausgaben pro Kostenstelle | Spese per conto analitico | Expenses by analytical account |
  | `reports-project-expenses-empty` | Aucune dépense taguée sur ce compte analytique pour la période. | Für diesen Zeitraum sind dieser Kostenstelle keine Ausgaben zugeordnet. | Nessuna spesa associata a questo conto analitico per il periodo. | No expenses tagged on this analytical account for the period. |
  | `reports-project-return`, `-return-title` | Rendement par compte analytique | Rendite pro Kostenstelle | Rendimento per conto analitico | Return by analytical account |
  | `reports-project-return-col-project` | Compte analytique | Kostenstelle | Conto analitico | Analytical account |
  | `reports-project-return-empty` | Aucun mouvement tagué sur ce compte analytique pour la période. | Für diesen Zeitraum sind dieser Kostenstelle keine Bewegungen zugeordnet. | Nessun movimento associato a questo conto analitico per il periodo. | No movements tagged on this analytical account for the period. |
  | `reports-project-selector-label` | Compte analytique | Kostenstelle | Conto analitico | Analytical account |
  | `reports-project-selector-placeholder` | — Choisir un compte analytique | — Kostenstelle wählen | — Scegli un conto analitico | — Choose an analytical account |
  | `reports-filename-project-expenses` | depenses-par-compte-analytique | ausgaben-pro-kostenstelle | spese-per-conto-analitico | expenses-by-analytical-account |
  | `reports-filename-project-return` | rendement-par-compte-analytique | rendite-pro-kostenstelle | rendimento-per-conto-analitico | return-by-analytical-account |
  | `export-global-description` | … règlements, **comptes analytiques**, contacts … | … Zahlungen, **Kostenstellen**, Kontakte … | … pagamenti, **conti analitici**, contatti … | … settlements, **analytical accounts**, contacts … |
  | `export-global-content-includes` | … produits, **comptes analytiques**, factures … | … Produkte, **Kostenstellen**, Ausgangsrechnungen … | … prodotti, **conti analitici**, fatture … | … products, **analytical accounts**, sales invoices … |
  | `invoice-detail-project-label` | Compte analytique | Kostenstelle | Conto analitico | Analytical account |
  | `invoice-field-project` | Compte analytique (optionnel) | Kostenstelle (optional) | Conto analitico (facoltativo) | Analytical account (optional) |
  | `invoice-project-archived` | Compte analytique archivé | Kostenstelle archiviert | Conto analitico archiviato | Analytical account archived |
  | `invoice-project-current` | Compte analytique actuel | Aktuelle Kostenstelle | Conto analitico attuale | Current analytical account |
  | `supplier-invoices-detail-project` | Compte analytique | Kostenstelle | Conto analitico | Analytical account |
  | `supplier-invoices-field-project` | Compte analytique (optionnel) | Kostenstelle (optional) | Conto analitico (facoltativo) | Analytical account (optional) |
  | `nav-projects`, `projects-title` | Comptes analytiques | Kostenstellen | Conti analitici | Analytical accounts |
  | `projects-archived` | Compte analytique archivé. | Kostenstelle archiviert. | Conto analitico archiviato. | Analytical account archived. |
  | `projects-created` | Compte analytique créé. | Kostenstelle erstellt. | Conto analitico creato. | Analytical account created. |
  | `projects-empty` | Aucun compte analytique. Créez votre premier compte analytique pour commencer. | Keine Kostenstelle. Erstellen Sie Ihre erste Kostenstelle, um zu beginnen. | Nessun conto analitico. Crea il tuo primo conto analitico per iniziare. | No analytical account. Create your first analytical account to get started. |
  | `projects-err-code` | Le code du compte analytique est requis. | Der Code der Kostenstelle ist erforderlich. | Il codice del conto analitico è obbligatorio. | The analytical account code is required. |
  | `projects-err-name` | Le nom du compte analytique est requis. | Der Name der Kostenstelle ist erforderlich. | Il nome del conto analitico è obbligatorio. | The analytical account name is required. |
  | `projects-field-parent` | Compte analytique parent (optionnel) | Übergeordnete Kostenstelle (optional) | Conto analitico superiore (facoltativo) | Parent analytical account (optional) |
  | `projects-form-create`, `projects-new` | Nouveau compte analytique | Neue Kostenstelle | Nuovo conto analitico | New analytical account |
  | `projects-form-edit` | Modifier le compte analytique | Kostenstelle bearbeiten | Modifica il conto analitico | Edit analytical account |
  | `projects-load-error` | Impossible de charger les comptes analytiques. | Kostenstellen können nicht geladen werden. | Impossibile caricare i conti analitici. | Unable to load the analytical accounts. |
  | `projects-parent-none` | — Aucun (compte analytique racine) | — Keine (Hauptkostenstelle) | — Nessuno (conto analitico radice) | — None (root analytical account) |
  | `projects-show-archived` | Afficher les comptes analytiques archivés | Archivierte Kostenstellen anzeigen | Mostra i conti analitici archiviati | Show archived analytical accounts |
  | `projects-subtitle` | Regroupez vos dépenses et revenus par compte analytique (un immeuble, un véhicule, une rénovation, une activité) pour les analyser isolément. | Fassen Sie Ihre Ausgaben und Erträge nach Kostenstelle zusammen (eine Liegenschaft, ein Fahrzeug, eine Renovation, eine Tätigkeit), um sie separat auszuwerten. | Raggruppa le tue spese e i tuoi ricavi per conto analitico (un immobile, un veicolo, una ristrutturazione, un'attività) per analizzarli separatamente. | Group your expenses and revenue by analytical account (a building, a vehicle, a renovation, an activity) to analyse them separately. |
  | `projects-unarchived` | Compte analytique désarchivé. | Kostenstelle dearchiviert. | Conto analitico ripristinato dall'archivio. | Analytical account unarchived. |
  | `projects-updated` | Compte analytique mis à jour. | Kostenstelle aktualisiert. | Conto analitico aggiornato. | Analytical account updated. |
  | `audit-log-entity-project` | Compte analytique | Kostenstelle | Conto analitico | Analytical account |
  | `audit-log-action-project-created` | Compte analytique créé | Kostenstelle erstellt | Conto analitico creato | Analytical account created |
  | `audit-log-action-project-updated` | Compte analytique modifié | Kostenstelle aktualisiert | Conto analitico aggiornato | Analytical account updated |
  | `audit-log-action-project-archived` | Compte analytique archivé | Kostenstelle archiviert | Conto analitico archiviato | Analytical account archived |
  | `audit-log-action-project-unarchived` | Compte analytique réactivé | Kostenstelle reaktiviert | Conto analitico riattivato | Analytical account reactivated |

  Le commentaire `fr-CH/messages.ftl:3089` dit la nouvelle convention (`« compte analytique » = Kostenstelle / conto
  analitico / analytical account`). Une phrase allemande ou italienne qu'un accord rendrait bancale se corrige au
  développement, en gardant le terme, et se déclare au Change Log.

- **D3 — Identifiants internes inchangés** (C-e360-15) : clés i18n (`*project*` gardées : les renommer changerait
  `sitesTotal` et l'ownership sans rien montrer à l'utilisateur), table, colonnes, routes (`/api/v1/projects`,
  `/settings/projects`), champs JSON, codes d'erreur, `data-testid` (`project-*`), noms de composants
  (`ProjectExpensesView`…), types et fonctions. *Écartée* : renommer les clés en `analytic-account-*` — 196 lignes de plus
  pour un gain nul à l'écran.

- **D4 — Les messages de refus de `/api/v1/projects` changent de mot** (C-36-15-2). Les 11 textes de `routes/projects.rs`
  (§ 3) s'affichent tels quels sur l'écran « Comptes analytiques » ; les laisser dire « projet » serait le renommage à
  moitié fait que le CR refuse. Les **codes** (`VALIDATION_ERROR`) et le statut HTTP ne changent pas ; ce n'est pas un
  changement de contrat au sens de C-e360-15, mais un texte d'erreur de l'API change : la note de version le dit
  (précédent : `INVOICE_NOT_PDF_READY` à la 0.17.7). Textes :
  « Le code du compte analytique est requis. » ; « Le code du compte analytique dépasse 32 caractères. » ; « Le nom du
  compte analytique est requis. » ; « Le nom du compte analytique dépasse 150 caractères. » ; « Un compte analytique ne
  peut pas être son propre parent. » ; « Le compte analytique parent est introuvable. » ; « Un sous-compte analytique ne
  peut pas être parent (hiérarchie limitée à 2 niveaux). » ; « Le compte analytique parent est archivé — désarchivez-le
  d'abord. » (×2) ; « Ce compte analytique a des sous-comptes analytiques : il ne peut pas devenir lui-même sous-compte
  analytique. » ; « Ce compte analytique a des sous-comptes analytiques actifs — archivez-les d'abord. »
  *Écartée* : les laisser à #710 (traduction des refus du backend) — #710 traduira ce qui sera écrit ici.

- **D5 — Les en-têtes CSV `Projet;SousProjet` restent** (C-36-15-3). Ce sont des noms de colonnes en forme
  d'identifiant (`NumeroCompte`, `CoutInvesti`), lus par des tableurs et des macros ; les changer casserait un usage sans
  rien dire de mieux. Le **titre** du PDF, lui, change (§ 3). *Écartée* : `CompteAnalytique;SousCompteAnalytique` —
  réversible si Guy le demande (une ligne par rapport et deux assertions de `project_reports_e2e.rs`).

- **D6 — Hors du texte visible, rien ne bouge** : commentaires et doc-comments (Rust, TS, Svelte, LaTeX), noms de tests,
  données de test, messages internes `DbError` (dont le couplage `reconciliation.rs:763`), `errors.rs:23` de
  `kesh-report`, `projects.csv`, `projet_id` du MCP, l'historique (CHANGELOG publié, ligne E19 du tableau des versions
  `README.md:218`, carte E19 de `website/roadmap.html:249-252`, comptes rendus de stories). *Écartée* : réécrire les
  commentaires — des centaines de lignes, aucun lecteur concerné, et un historique faussé.

- **D7 — Le libellé en dur `journal-entries/[id]/+page.svelte:611` change de texte et reste en dur** (C-36-15-4) :
  `<th class="py-2 pr-2">Compte analytique</th>`. Le passer à `i18nMsg` ajouterait une clé ×4 et un site (`sitesTotal`),
  hors du périmètre de vocabulaire ; c'est la dette de #339 (libellés en dur du balisage), qui reste ouverte.

- **D8 — Manuels : le `.tex` dans la story, le PDF à l'intégration** (C-36-15-5). `user-manual.tex` est commun à
  d'autres stories de la 0.18.0 (36-16, 36-17, 36-18 probables) : les trois PDF se régénèrent une fois, après l'union, et
  se contrôlent aplatis (E6). Le développement compile localement (`make fr`, voie frontend et manuels) pour vérifier que
  le `.tex` compile et pour relever les comptes de l'AC-6 sur le PDF local, puis rend les PDF à leur état
  (`git restore --worktree -- docs/manual/fr/*.pdf`) : aucun PDF dans les commits de la story.

- **D9 — La documentation dit le lien entre le mot et l'identifiant.** `docs/api-external.md` reçoit une phrase, dans
  sa vue d'ensemble (§ 1) : « Ce que l'interface appelle **compte analytique** s'appelle `project` dans l'API — route
  `/api/v1/projects`, champs `projectId` et `defaultProjectId`, codes `PROJECT_ARCHIVED` et `PROJECT_NOT_FOUND` ; ces noms
  ne changent pas. » Ses cinq lignes analytiques (§ 6) disent « compte analytique ». `docs/i18n-glossaire.md` : les lignes
  `:105`, `:168`, `:169` passent aux termes de D2 (le compte de la partie A se recompte et s'écrit), la note `:172-189`
  reste comme historique, annotée et suivie de l'arbitrage de 36-15. `README.md:47` (Fonctionnalités) dit
  « **Comptabilité analytique** — dimension « compte analytique » … **Dépenses par compte analytique** … **Rendement par
  compte analytique** ». Le site n'a pas de mention actuelle à changer (§ 6).

## Critères d'acceptation

- **AC-1 — Catalogues.** Les 49 clés de § 1 portent, dans les quatre catalogues, les valeurs de D2 ; aucune autre clé
  ne change ; l'ensemble des clés est identique avant et après (`loader.rs::parity_between_locales` vert ; `git diff` des
  quatre `.ftl` ne touche que des lignes de valeur et le commentaire `:3089`). La garde G25 (AC-7) ne trouve, dans les
  quatre catalogues, aucune occurrence de `projet` (fr), `Projekt` (de), `progett` (it), `\bprojects?\b` (en), hors
  `en-CH` `dunning-example-heading` (« Projected »), seule exception écrite.
- **AC-2 — Frontend.** Les 46 sites de § 2 disent « compte analytique » : chaque repli `i18nMsg` est **égal** à la
  nouvelle valeur fr-CH de sa clé ; le repli de nom de fichier (`reports/+page.svelte:433`) égale
  `depenses-par-compte-analytique` / `rendement-par-compte-analytique` ; `journal-entries/[id]/+page.svelte:611` dit
  « Compte analytique ». La garde Vitest (AC-8) ne trouve plus aucun `projet` hors commentaires dans `frontend/src`
  (hors `*.test.ts`).
- **AC-3 — Backend.** Les 11 messages de `routes/projects.rs` et les 4 libellés de `ProjectExpensesPdfLabels` /
  `ProjectReturnPdfLabels::fr_ch_defaults()` disent « compte analytique » (D4, D1) ; le nom de fichier d'un export
  `GET /api/v1/reports/project-expenses/export` sur une société fr-CH contient `depenses-par-compte-analytique` (et
  `rendement-par-compte-analytique` pour `project-return`) ; les en-têtes CSV restent `Projet;SousProjet;…` (D5).
- **AC-4 — Rien d'autre ne bouge.** `git diff origin/epic/36-0.18.0 --stat` ne nomme aucun fichier sous
  `crates/kesh-db/`, aucune migration, ni `routes/reconciliation.rs`, ni `kesh-report/src/csv.rs`, ni
  `i18n-keys.test.ts` ; `sitesTotal` reste **3050** (`i18n-keys.test.ts:1020`) ; `npm run lint-i18n-ownership` vert sans
  changement de son registre.
- **AC-5 — E2E.** `reports.spec.ts` ouvre les onglets **par leurs nouveaux noms** (`/dépenses par compte analytique/i`,
  `/rendement par compte analytique/i`) et y vérifie, en plus, que le titre de la vue rendue commence par ce nom ; les deux
  tests rougissent sur les catalogues d'avant (AC de preuve, § Tests).
- **AC-6 — Manuels.** Les 33 lignes analytiques de § 5 disent « compte analytique » (`user-manual.tex` : section
  `\section{Comptabilité analytique}`, chemin « \emph{Administration} → \emph{Comptes analytiques} », les deux rapports
  sous leurs nouveaux noms, une phrase qui dit ce qu'un compte analytique peut suivre — un immeuble, un véhicule, une
  activité, un chantier) ; les 16 lignes au sens courant restent ; sur le PDF compilé localement (D8),
  `pdftotext user-manual.pdf - | tr '\n' ' ' | tr -s ' ' | /usr/bin/grep -c 'projet analytique'` rend 0 et `… -c 'compte
  analytique'` rend au moins 1, relevés au Change Log avec leur commit.
- **AC-7 — Garde de texte backend et documentation (G25).** Un test de `crates/kesh-api/tests/textes_coherents.rs`
  lit les quatre catalogues, les trois `.tex` français, `docs/api-external.md`, `README.md`, et les sources hors modules
  de test (`sans_modules_de_test`) de `routes/projects.rs` et `kesh-report/src/pdf.rs`, et rougit en **nommant fichier et
  ligne** à toute occurrence du mot de l'ancienne langue hors d'une **liste fermée d'exceptions** écrites avec leur motif
  (les 15 lignes Docker et Claude Code de l'admin, `marketing-brochure.tex:498`, les trois « projet » courants du README,
  la ligne historique `README.md:218`, « Projected »). Chaque exception doit **encore** se trouver (une exception qui ne
  rencontre plus rien fait rougir) ; une assertion positive par fichier prouve qu'il a été lu (« Comptes analytiques »
  dans chaque catalogue fr-CH… et son équivalent par langue).
- **AC-8 — Garde Vitest du frontend.** `frontend/src/lib/shared/vocabulaire-analytique.test.ts` parcourt
  `frontend/src/**/*.{svelte,ts,js}` hors `*.test.ts`, masque les commentaires par `masquerCommentaires`, et attend
  **zéro** site `/projet/i` ; il vérifie aussi que chaque repli littéral des 49 clés égale la valeur fr-CH du catalogue
  (`releveDesReplis` / `catalogueDe` de `i18n-repli-catalogue.ts`), avec sa sonde positive (une source synthétique
  `i18nMsg('x', 'Projet archivé')` est détectée).
- **AC-9 — Documentation.** `docs/api-external.md` porte la phrase de D9 et ne dit plus « projet » ; le glossaire porte
  les termes de D2 et le compte recompté de sa partie A ; `README.md:47` dit « compte analytique ».
- **AC-10 — Aucun mot-clé de fermeture** dans les commits
  (`git log origin/epic/36-0.18.0..HEAD --format=%B | /usr/bin/grep -niE '\b(close[sd]?|fix(e[sd])?|resolve[sd]?) #[0-9]'`
  vide) ; aucune ligne de `CHANGELOG.md` (C-e360-11).

## Tests à écrire, et la preuve du rouge

Ordre imposé (E5) : les gardes et tests neufs se commitent **avant** le correctif, se jouent sur les textes d'avant
(rouge), puis le correctif les met au vert. Journaux dans `/home/gcorbaz/devel/kesh-gate-logs/e36-15/`.

| Test | Rouge attendu sans le correctif | Journal |
|---|---|---|
| G25 `le_vocabulaire_analytique_est_compte_analytique` (`textes_coherents.rs`) | sur les fichiers d'avant : liste des sites, ~196 lignes de catalogue, 33 de manuel, 5 de `api-external.md`, 1 du README, 15 de Rust | `rouges.log` |
| G25 — mutation 1 : réintroduire `projects-title = Projekte` en de-CH | rouge nommant `de-CH/messages.ftl:<ligne>` | `mut/g25-catalogue.log` |
| G25 — mutation 2 : retirer une exception (une ligne Docker de l'admin) de la liste | rouge nommant `admin-manual.tex:<ligne>` | `mut/g25-exception.log` |
| G25 — mutation 3 : ajouter une exception qui ne rencontre rien | rouge « exception périmée » | `mut/g25-perimee.log` |
| `vocabulaire-analytique.test.ts` | 46 sites sur les sources d'avant | `rouges.log` |
| idem — mutation : rendre un repli `'Projet archivé'` (`InvoiceForm.svelte:830`) | rouge sur ce site **et** sur l'écart repli ≠ catalogue | `mut/vitest-repli.log` |
| `project_reports_e2e.rs` : `export_filenames_say_compte_analytique` (neuf) | `Content-Disposition` contient `depenses-par-projet` | `rouges.log` |
| `reports.spec.ts` (deux tests modifiés) | onglet introuvable sur les catalogues d'avant (spec seule, voie 1, backend lancé pour la preuve) | `rouges-e2e.log` |

Restauration après mutation : `git restore --worktree -- <fichier>` puis `touch <fichier>` ; scripts de mutation dans
`/home/gcorbaz/devel/kesh-gate-logs/e36-15/mut/`.

**Pourquoi pas de test unitaire de `fr_ch_defaults()`** : G25 lit `pdf.rs` et rougit sur ses quatre textes ; un test qui
recopierait la valeur attendue ne prouverait rien de plus.

## Tâches

1. **Étape 0** : relire C-e360-15, ce que dit le CR, et `git log origin/epic/36-0.18.0..HEAD` ; vérifier que
   `crates/kesh-db/` reste hors du diff prévu. Base `kesh_e3615` et `kesh_e2e_e3615` créées à neuf (voie 1).
2. **Gardes d'abord** (AC-7, AC-8, test neuf de `project_reports_e2e.rs`, sélecteurs de `reports.spec.ts`), commit ;
   rouges journalisés.
3. **Catalogues** (AC-1) par l'outil Edit, **par blocs contigus** (`projects-*` :3171-3196, `reports-project-*`
   :1944-1964, `audit-log-action-project-*` :3362-3365, `invoice-*` :2659-2662…) pour borner le nombre d'éditions ; fr-CH,
   puis de-CH, it-CH, en-CH, clé par clé contre la table de D2.
4. **Frontend** (AC-2) : 46 sites.
5. **Backend** (AC-3) : `routes/projects.rs` (11), `kesh-report/src/pdf.rs` (4) ; `cargo fmt`.
6. **Documentation** (AC-6, AC-9) : trois `.tex`, `api-external.md`, glossaire, `README.md:47` ; `make fr` local, comptes
   de l'AC-6, PDF rendus (D8).
7. **Gates** (voie 1, verrous `flock /tmp/station-heavy.lock flock /home/gcorbaz/devel/kesh-gate-logs/.gate.lock
   scripts/mem-guard.sh …`, une commande par prise, `free -g` avant) :
   - en cours : `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`,
     `cargo nextest run -E 'binary(textes_coherents) | binary(project_reports_e2e) | package(kesh-i18n) | package(kesh-report)'`
     (pas de `kesh-db` touché : le gate complet n'est pas exigé en boucle) ; Vitest ciblé
     (`vocabulaire-analytique`, `i18n-*`) par `.gate-front.lock` ;
   - passage en `review` : bases remises à neuf, `scripts/test-fast.sh --ci`, `scripts/gate-ci.sh kesh_ci_e3615`,
     frontend complet (`npm run check`, `lint-i18n-ownership`, `test:unit`, `build`) ; **pas d'E2E** ;
   - remédiations : `test-fast --ci` (+ frontend complet si touché) ;
   - clôture : `gate-ci.sh kesh_ci_e3615` **et** E2E complet sur `kesh_e2e_e3615` recréée, au dernier commit de code ;
     échecs nommés contre `docs/testing.md` § « Les échecs attendus ».
8. **Captures** (thème clair, base de démonstration jetable, `/home/gcorbaz/devel/kesh-gate-logs/e36-15/captures/`) :
   menu latéral, écran « Comptes analytiques » (liste, formulaire, refus « parent archivé »), colonne d'une écriture
   (formulaire et détail), champ d'une facture client et d'une facture fournisseur, dialogues de réconciliation (écriture
   manuelle, ventilation, règle), les deux onglets de rapport et leurs PDF exportés, le journal d'audit ; une capture en
   allemand de l'écran « Kostenstellen » (langue de la société passée en de-CH sur la base jetable).
9. Fiche à `review`, prompt de la revue P1 à côté de la fiche (lentilles B, E, A ; manuels, PDF local et captures nommés ;
   relire la table de D2 dans les trois langues).

## Note de version

*Modifié* — Kesh parle de **compte analytique** et non plus de projet analytique : le menu, l'écran *Comptes
analytiques*, les champs des écritures, des factures et de la réconciliation, les rapports **Dépenses par compte
analytique** et **Rendement par compte analytique** (titres à l'écran et dans le PDF, noms des fichiers exportés), le
journal d'audit et les manuels, dans les quatre langues (allemand *Kostenstelle*, italien *conto analitico*, anglais
*analytical account*). Rien ne change de ce qui est enregistré : routes, champs et codes de l'API gardent leur nom
`project` ; les colonnes `Projet;SousProjet` des exports CSV restent. ⚠️ Les textes des refus de `/api/v1/projects`
disent désormais « compte analytique » (codes inchangés). (refs #1161)

## Dev Notes

### Hors périmètre

- #339 (libellés en dur du balisage, dont `journal-entries/[id]/+page.svelte:609-611`) ; #710 (refus du backend en
  français en dur, dont ceux de `projects.rs`) ; #405 (un objet « immeuble ») et #220 (plus de deux niveaux) : le terme
  générique y répond en partie, sans les fermer ; #1167 (36-16, la réimputation), qui part de 36-15 fusionnée.
- Ce que D5 et D6 gardent. Aucune issue n'est à ouvrir pour eux : ce sont des décisions, réversibles (registre).

### Fichiers communs (0.18.0)

| Fichier | Autres stories | Règle |
|---|---|---|
| `messages.ftl` ×4 | 36-17 (assistant), 36-10 (journal d'audit), 36-18 (montants) possibles | 36-15 ne modifie que des lignes existantes, sans en ajouter ; une union ne peut se heurter qu'à une ligne `audit-log-*` que 36-10 changerait aussi |
| `docs/manual/fr/user-manual.tex` | 36-16, 36-17, 36-18 possibles | paragraphes distincts ; 36-16 réemploie le vocabulaire de 36-15 (C-e360-16) ; PDF après l'union (D8) |
| `crates/kesh-api/tests/textes_coherents.rs` | toute story qui ajoute une garde | G25 est le numéro libre à `0bf3c3b4b` (`git grep -n "G25" -- crates docs` vide) ; renuméroter à l'union si une autre branche l'a pris |
| `docs/i18n-glossaire.md`, `docs/api-external.md` | 36-16 possible (`api-external.md`) | sections distinctes |

### Modules (règle de découpage)

Quatre modules de code — `kesh-i18n` (catalogues), le frontend, `kesh-api` (`routes/projects.rs` et deux fichiers de
tests), `kesh-report` (`pdf.rs`) — et la documentation (manuels, `docs/`, README) : **cinq**, à la limite, sans la
franchir. Le volume est mécanique (≈ 196 lignes de catalogue, 46 sites, 15 chaînes Rust, ≈ 40 lignes de documentation).
Si la validation le juge trop long, le découpage proposé est 36-15a (application : catalogues, frontend, backend,
gardes, E2E) puis 36-15b (documentation), fusionnées toutes deux avant le tag — l'application seule ne laisse aucun écran
dire « projet », et le CR n'exige le geste unique que de ce que l'utilisateur voit.

### Ce qui doit rester vrai

- Les quatre `IllegalStateTransition("le projet analytique est archivé")` restent **à l'octet** : `reconciliation.rs:763`
  les reconnaît à `msg.contains("projet")` pour rendre `PROJECT_ARCHIVED`.
- `project_reports_e2e.rs:218` et `:350` (en-têtes CSV) restent verts sans modification.
- `sitesTotal` 3050 inchangé ; les 32 clés `*project*` neutres (« — Aucun », « Compte »…) inchangées.
- Pas de migration (C-e360-3) : la story n'en demande aucune.

### Références

- CR #1161 ; C-e360-15 ; fichier d'epic, phase 2, rang 14 et « Phase 2 — décisions de Guy du 2026-11-15 ».
- `docs/i18n-glossaire.md:168-189` (l'arbitrage remplacé) ; `docs/testing.md` § « Les échecs attendus ».
- `frontend/src/lib/shared/i18n-repli-catalogue.ts` (`ecartsAuCatalogue`, angle mort #758) ;
  `frontend/src/lib/shared/i18n-literal-reader.js:261` (`masquerCommentaires`).

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

## Change Log

- **2026-11-15 — création de la fiche** (Opus 5.5, `bmad-create-story`, code lu à `0bf3c3b4b`). Inventaire par le
  symptôme dans les quatre langues : 49 clés ×4 + un commentaire ; 46 sites frontend (détecteur exécuté, commentaires
  masqués) ; 15 chaînes Rust visibles ; 33 lignes de manuel analytiques sur 49 lignes « projet » ; 5 lignes de
  `api-external.md`. Choix C-36-15-1 à 6 au registre de la story. Gates : aucun (fiche seule).
