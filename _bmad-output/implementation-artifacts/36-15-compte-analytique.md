# Story 36-15 : « compte analytique » au lieu de « projet analytique » (#1161)

Status: ready-for-dev

> ⛔ **Développement suspendu à l'arbitrage de Guy sur les traductions (D2)** : fr-CH « compte analytique » est acquis ;
> de-CH, it-CH et en attendent la réponse à la page de décision de l'orchestrateur (option A recommandée).

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

- **D2 — Les traductions : fr-CH acquis, de-CH / it-CH / en en attente de l'arbitrage de Guy** (CR #1161,
  recommandation 3 ; C-36-15-1). ⛔ **Le développement ne commence qu'après la réponse de Guy**, soumise par
  l'orchestrateur sur une page de décision ; la fiche ne tranche pas.

  **Acquis** : fr-CH « compte analytique » (D1), et tout ce qui ne dépend que du français — les 49 valeurs fr-CH, les 46
  sites du frontend, les 15 chaînes Rust, les manuels, `api-external.md`, `README.md:47`.

  **En attente** — trois options, soumises à Guy :

  | Option | de-CH | it-CH | en-CH | |
  |---|---|---|---|---|
  | **A** | **analytisches Konto** (n.) | **conto analitico** (m.) | **analytical account** | **recommandée par l'orchestrateur** : suit le français (règle du glossaire `:29`, « En cas de doute sur une langue cible, suivre le FRANÇAIS ») ; *analytical* est l'adjectif que le catalogue anglais emploie déjà (`en-CH/messages.ftl:1478`, « The analytical project is archived. ») |
  | **B** | Kostenstelle (f.) | centro di costo (m.) | cost centre | le terme de gestion consacré dans les trois langues ; ne dit que les coûts, alors que le rendement compte des revenus |
  | **C** | Kostenstelle (f.) | conto analitico (m.) | analytical account | le mélange de la première version de cette fiche (`54b513a71`) |

  Formes dérivées par terme :

  | fr-CH | analytisches Konto | Kostenstelle | conto analitico | centro di costo | analytical account | cost centre |
  |---|---|---|---|---|---|---|
  | pluriel | analytische Konten | Kostenstellen | conti analitici | centri di costo | analytical accounts | cost centres |
  | … parent | übergeordnetes analytisches Konto | übergeordnete Kostenstelle | conto analitico superiore | centro di costo superiore | parent analytical account | parent cost centre |
  | sous-… | analytisches Unterkonto | Unterkostenstelle | sottoconto analitico | sottocentro di costo | analytical sub-account | sub-cost centre |
  | … racine | analytisches Hauptkonto | Hauptkostenstelle | conto analitico radice | centro di costo radice | root analytical account | root cost centre |

  *Source* : aucun précédent au catalogue (`git grep -n -iE "kostenst|kostentr|conto analitico|analytical account"` ne
  rend que la note du glossaire, `docs/i18n-glossaire.md:176-186`) ; les termes sont ceux que propose le CR #1161. **Le
  fondement est le CR de Guy**, non un changement de l'objet : la table `projects` porte toujours code, nom, deux niveaux
  et dates (`20260702000001_projects_analytics.sql`), ce qu'invoquait l'arbitrage du 2026-08-19 (glossaire `:172-189`).
  L'option B ou C remplace cet arbitrage ; l'option A le prolonge (« on ne traduit pas » devient « on suit le français »).
  *Écartées d'emblée* : `Kostenträger` et `commessa` (le « pour quoi ? » de 2026-08-19, trop étroit pour un immeuble ou un
  centre de coûts) ; garder `Projekt` / `progetto` / *project* (le défaut même que le CR corrige).

  **Valeurs, clé par clé.** fr-CH, puis une colonne par terme : la réponse de Guy choisit une colonne par langue
  (A = `analytisches Konto`, `conto analitico`, `analytical account` ; B = `Kostenstelle`, `centro di costo`,
  `cost centre` ; C = `Kostenstelle`, `conto analitico`, `analytical account`). Les deux clés d'export ne changent qu'un mot
  de leur liste (en gras).

  **fr-CH** (acquis) :

  | Clé | fr-CH |
  |---|---|
  | `journal-entry-form-col-project`, `reconciliation-split-th-project`, `reports-project-return-col-project`, `reports-project-selector-label`, `invoice-detail-project-label`, `supplier-invoices-detail-project`, `audit-log-entity-project` | Compte analytique |
  | `journal-entry-project-archived`, `reconciliation-rules-default-project-archived`, `invoice-project-archived`, `audit-log-action-project-archived` | Compte analytique archivé |
  | `reconciliation-failed-project-archived` | Le compte analytique est archivé. |
  | `reconciliation-failed-project-not-found` | Compte analytique introuvable. |
  | `reconciliation-manual-project-label`, `invoice-field-project`, `supplier-invoices-field-project` | Compte analytique (optionnel) |
  | `reconciliation-rules-labels-default-project` | Compte analytique par défaut |
  | `reports-project-expenses`, `reports-project-expenses-title` | Dépenses par compte analytique |
  | `reports-project-expenses-empty` | Aucune dépense taguée sur ce compte analytique pour la période. |
  | `reports-project-return`, `reports-project-return-title` | Rendement par compte analytique |
  | `reports-project-return-empty` | Aucun mouvement tagué sur ce compte analytique pour la période. |
  | `reports-project-selector-placeholder` | — Choisir un compte analytique |
  | `reports-filename-project-expenses` | depenses-par-compte-analytique |
  | `reports-filename-project-return` | rendement-par-compte-analytique |
  | `export-global-description` | … règlements, **comptes analytiques**, contacts … |
  | `export-global-content-includes` | … produits, **comptes analytiques**, factures … |
  | `invoice-project-current` | Compte analytique actuel |
  | `nav-projects`, `projects-title` | Comptes analytiques |
  | `projects-archived` | Compte analytique archivé. |
  | `projects-created` | Compte analytique créé. |
  | `projects-empty` | Aucun compte analytique. Créez votre premier compte analytique pour commencer. |
  | `projects-err-code` | Le code du compte analytique est requis. |
  | `projects-err-name` | Le nom du compte analytique est requis. |
  | `projects-field-parent` | Compte analytique parent (optionnel) |
  | `projects-form-create`, `projects-new` | Nouveau compte analytique |
  | `projects-form-edit` | Modifier le compte analytique |
  | `projects-load-error` | Impossible de charger les comptes analytiques. |
  | `projects-parent-none` | — Aucun (compte analytique racine) |
  | `projects-show-archived` | Afficher les comptes analytiques archivés |
  | `projects-subtitle` | Regroupez vos dépenses et revenus par compte analytique (un immeuble, un véhicule, une rénovation, une activité) pour les analyser isolément. |
  | `projects-unarchived` | Compte analytique désarchivé. |
  | `projects-updated` | Compte analytique mis à jour. |
  | `audit-log-action-project-created` | Compte analytique créé |
  | `audit-log-action-project-updated` | Compte analytique modifié |
  | `audit-log-action-project-unarchived` | Compte analytique réactivé |

  **de-CH** (en attente) :

  | Clé | analytisches Konto (A) | Kostenstelle (B, C) |
  |---|---|---|
  | `journal-entry-form-col-project`, `reconciliation-split-th-project`, `reports-project-return-col-project`, `reports-project-selector-label`, `invoice-detail-project-label`, `supplier-invoices-detail-project`, `audit-log-entity-project` | Analytisches Konto | Kostenstelle |
  | `journal-entry-project-archived`, `reconciliation-rules-default-project-archived`, `invoice-project-archived`, `audit-log-action-project-archived` | Analytisches Konto archiviert | Kostenstelle archiviert |
  | `reconciliation-failed-project-archived` | Das analytische Konto ist archiviert. | Die Kostenstelle ist archiviert. |
  | `reconciliation-failed-project-not-found` | Analytisches Konto nicht gefunden. | Kostenstelle nicht gefunden. |
  | `reconciliation-manual-project-label`, `invoice-field-project`, `supplier-invoices-field-project` | Analytisches Konto (optional) | Kostenstelle (optional) |
  | `reconciliation-rules-labels-default-project` | Analytisches Standardkonto | Standard-Kostenstelle |
  | `reports-project-expenses`, `-expenses-title` | Ausgaben pro analytischem Konto | Ausgaben pro Kostenstelle |
  | `reports-project-expenses-empty` | Für diesen Zeitraum sind diesem analytischen Konto keine Ausgaben zugeordnet. | Für diesen Zeitraum sind dieser Kostenstelle keine Ausgaben zugeordnet. |
  | `reports-project-return`, `-return-title` | Rendite pro analytischem Konto | Rendite pro Kostenstelle |
  | `reports-project-return-empty` | Für diesen Zeitraum sind diesem analytischen Konto keine Bewegungen zugeordnet. | Für diesen Zeitraum sind dieser Kostenstelle keine Bewegungen zugeordnet. |
  | `reports-project-selector-placeholder` | — Analytisches Konto wählen | — Kostenstelle wählen |
  | `reports-filename-project-expenses` | ausgaben-pro-analytischem-konto | ausgaben-pro-kostenstelle |
  | `reports-filename-project-return` | rendite-pro-analytischem-konto | rendite-pro-kostenstelle |
  | `export-global-description`, `export-global-content-includes` | … **analytische Konten** … | … **Kostenstellen** … |
  | `invoice-project-current` | Aktuelles analytisches Konto | Aktuelle Kostenstelle |
  | `nav-projects`, `projects-title` | Analytische Konten | Kostenstellen |
  | `projects-archived` | Analytisches Konto archiviert. | Kostenstelle archiviert. |
  | `projects-created`, `audit-log-action-project-created` | Analytisches Konto erstellt(.) | Kostenstelle erstellt(.) |
  | `projects-empty` | Kein analytisches Konto. Erstellen Sie Ihr erstes analytisches Konto, um zu beginnen. | Keine Kostenstelle. Erstellen Sie Ihre erste Kostenstelle, um zu beginnen. |
  | `projects-err-code` | Der Code des analytischen Kontos ist erforderlich. | Der Code der Kostenstelle ist erforderlich. |
  | `projects-err-name` | Der Name des analytischen Kontos ist erforderlich. | Der Name der Kostenstelle ist erforderlich. |
  | `projects-field-parent` | Übergeordnetes analytisches Konto (optional) | Übergeordnete Kostenstelle (optional) |
  | `projects-form-create`, `projects-new` | Neues analytisches Konto | Neue Kostenstelle |
  | `projects-form-edit` | Analytisches Konto bearbeiten | Kostenstelle bearbeiten |
  | `projects-load-error` | Analytische Konten können nicht geladen werden. | Kostenstellen können nicht geladen werden. |
  | `projects-parent-none` | — Keines (analytisches Hauptkonto) | — Keine (Hauptkostenstelle) |
  | `projects-show-archived` | Archivierte analytische Konten anzeigen | Archivierte Kostenstellen anzeigen |
  | `projects-subtitle` | Fassen Sie Ihre Ausgaben und Erträge nach analytischem Konto zusammen (eine Liegenschaft, ein Fahrzeug, eine Renovation, eine Tätigkeit), um sie separat auszuwerten. | Fassen Sie Ihre Ausgaben und Erträge nach Kostenstelle zusammen (eine Liegenschaft, ein Fahrzeug, eine Renovation, eine Tätigkeit), um sie separat auszuwerten. |
  | `projects-unarchived` | Analytisches Konto dearchiviert. | Kostenstelle dearchiviert. |
  | `projects-updated`, `audit-log-action-project-updated` | Analytisches Konto aktualisiert(.) | Kostenstelle aktualisiert(.) |
  | `audit-log-action-project-unarchived` | Analytisches Konto reaktiviert | Kostenstelle reaktiviert |

  **it-CH** (en attente) :

  | Clé | conto analitico (A, C) | centro di costo (B) |
  |---|---|---|
  | `journal-entry-form-col-project`, `reconciliation-split-th-project`, `reports-project-return-col-project`, `reports-project-selector-label`, `invoice-detail-project-label`, `supplier-invoices-detail-project`, `audit-log-entity-project` | Conto analitico | Centro di costo |
  | `journal-entry-project-archived`, `reconciliation-rules-default-project-archived`, `invoice-project-archived`, `audit-log-action-project-archived` | Conto analitico archiviato | Centro di costo archiviato |
  | `reconciliation-failed-project-archived` | Il conto analitico è archiviato. | Il centro di costo è archiviato. |
  | `reconciliation-failed-project-not-found` | Conto analitico introvabile. | Centro di costo introvabile. |
  | `reconciliation-manual-project-label`, `invoice-field-project`, `supplier-invoices-field-project` | Conto analitico (facoltativo) | Centro di costo (facoltativo) |
  | `reconciliation-rules-labels-default-project` | Conto analitico predefinito | Centro di costo predefinito |
  | `reports-project-expenses`, `-expenses-title` | Spese per conto analitico | Spese per centro di costo |
  | `reports-project-expenses-empty` | Nessuna spesa associata a questo conto analitico per il periodo. | Nessuna spesa associata a questo centro di costo per il periodo. |
  | `reports-project-return`, `-return-title` | Rendimento per conto analitico | Rendimento per centro di costo |
  | `reports-project-return-empty` | Nessun movimento associato a questo conto analitico per il periodo. | Nessun movimento associato a questo centro di costo per il periodo. |
  | `reports-project-selector-placeholder` | — Scegli un conto analitico | — Scegli un centro di costo |
  | `reports-filename-project-expenses` | spese-per-conto-analitico | spese-per-centro-di-costo |
  | `reports-filename-project-return` | rendimento-per-conto-analitico | rendimento-per-centro-di-costo |
  | `export-global-description`, `export-global-content-includes` | … **conti analitici** … | … **centri di costo** … |
  | `invoice-project-current` | Conto analitico attuale | Centro di costo attuale |
  | `nav-projects`, `projects-title` | Conti analitici | Centri di costo |
  | `projects-archived` | Conto analitico archiviato. | Centro di costo archiviato. |
  | `projects-created`, `audit-log-action-project-created` | Conto analitico creato(.) | Centro di costo creato(.) |
  | `projects-empty` | Nessun conto analitico. Crea il tuo primo conto analitico per iniziare. | Nessun centro di costo. Crea il tuo primo centro di costo per iniziare. |
  | `projects-err-code` | Il codice del conto analitico è obbligatorio. | Il codice del centro di costo è obbligatorio. |
  | `projects-err-name` | Il nome del conto analitico è obbligatorio. | Il nome del centro di costo è obbligatorio. |
  | `projects-field-parent` | Conto analitico superiore (facoltativo) | Centro di costo superiore (facoltativo) |
  | `projects-form-create`, `projects-new` | Nuovo conto analitico | Nuovo centro di costo |
  | `projects-form-edit` | Modifica il conto analitico | Modifica il centro di costo |
  | `projects-load-error` | Impossibile caricare i conti analitici. | Impossibile caricare i centri di costo. |
  | `projects-parent-none` | — Nessuno (conto analitico radice) | — Nessuno (centro di costo radice) |
  | `projects-show-archived` | Mostra i conti analitici archiviati | Mostra i centri di costo archiviati |
  | `projects-subtitle` | Raggruppa le tue spese e i tuoi ricavi per conto analitico (un immobile, un veicolo, una ristrutturazione, un'attività) per analizzarli separatamente. | Raggruppa le tue spese e i tuoi ricavi per centro di costo (un immobile, un veicolo, una ristrutturazione, un'attività) per analizzarli separatamente. |
  | `projects-unarchived` | Conto analitico ripristinato dall'archivio. | Centro di costo ripristinato dall'archivio. |
  | `projects-updated`, `audit-log-action-project-updated` | Conto analitico aggiornato(.) | Centro di costo aggiornato(.) |
  | `audit-log-action-project-unarchived` | Conto analitico riattivato | Centro di costo riattivato |

  **en-CH** (en attente) :

  | Clé | analytical account (A, C) | cost centre (B) |
  |---|---|---|
  | `journal-entry-form-col-project`, `reconciliation-split-th-project`, `reports-project-return-col-project`, `reports-project-selector-label`, `invoice-detail-project-label`, `supplier-invoices-detail-project`, `audit-log-entity-project` | Analytical account | Cost centre |
  | `journal-entry-project-archived`, `reconciliation-rules-default-project-archived`, `invoice-project-archived`, `audit-log-action-project-archived` | Analytical account archived | Cost centre archived |
  | `reconciliation-failed-project-archived` | The analytical account is archived. | The cost centre is archived. |
  | `reconciliation-failed-project-not-found` | Analytical account not found. | Cost centre not found. |
  | `reconciliation-manual-project-label`, `invoice-field-project`, `supplier-invoices-field-project` | Analytical account (optional) | Cost centre (optional) |
  | `reconciliation-rules-labels-default-project` | Default analytical account | Default cost centre |
  | `reports-project-expenses`, `-expenses-title` | Expenses by analytical account | Expenses by cost centre |
  | `reports-project-expenses-empty` | No expenses tagged on this analytical account for the period. | No expenses tagged on this cost centre for the period. |
  | `reports-project-return`, `-return-title` | Return by analytical account | Return by cost centre |
  | `reports-project-return-empty` | No movements tagged on this analytical account for the period. | No movements tagged on this cost centre for the period. |
  | `reports-project-selector-placeholder` | — Choose an analytical account | — Choose a cost centre |
  | `reports-filename-project-expenses` | expenses-by-analytical-account | expenses-by-cost-centre |
  | `reports-filename-project-return` | return-by-analytical-account | return-by-cost-centre |
  | `export-global-description`, `export-global-content-includes` | … **analytical accounts** … | … **cost centres** … |
  | `invoice-project-current` | Current analytical account | Current cost centre |
  | `nav-projects`, `projects-title` | Analytical accounts | Cost centres |
  | `projects-archived` | Analytical account archived. | Cost centre archived. |
  | `projects-created`, `audit-log-action-project-created` | Analytical account created(.) | Cost centre created(.) |
  | `projects-empty` | No analytical account. Create your first analytical account to get started. | No cost centre. Create your first cost centre to get started. |
  | `projects-err-code` | The analytical account code is required. | The cost centre code is required. |
  | `projects-err-name` | The analytical account name is required. | The cost centre name is required. |
  | `projects-field-parent` | Parent analytical account (optional) | Parent cost centre (optional) |
  | `projects-form-create`, `projects-new` | New analytical account | New cost centre |
  | `projects-form-edit` | Edit analytical account | Edit cost centre |
  | `projects-load-error` | Unable to load the analytical accounts. | Unable to load the cost centres. |
  | `projects-parent-none` | — None (root analytical account) | — None (root cost centre) |
  | `projects-show-archived` | Show archived analytical accounts | Show archived cost centres |
  | `projects-subtitle` | Group your expenses and revenue by analytical account (a building, a vehicle, a renovation, an activity) to analyse them separately. | Group your expenses and revenue by cost centre (a building, a vehicle, a renovation, an activity) to analyse them separately. |
  | `projects-unarchived` | Analytical account unarchived. | Cost centre unarchived. |
  | `projects-updated`, `audit-log-action-project-updated` | Analytical account updated(.) | Cost centre updated(.) |
  | `audit-log-action-project-unarchived` | Analytical account reactivated | Cost centre reactivated |

  « (.) » : le point final ne figure que dans la clé `projects-*` (toast), non dans la clé `audit-log-action-*`, comme
  aujourd'hui. Le commentaire `fr-CH/messages.ftl:3089` dit la convention retenue (`« compte analytique » = <terme de-CH> /
  <terme it-CH> / <terme en>`). Une phrase qu'un accord rendrait bancale se corrige au développement, en gardant le terme,
  et se déclare au Change Log. **Le glossaire** suit la réponse (D9) : `:105` (« projet parent ») et `:169` (« projet
  analytique ») passent aux termes arbitrés ; `:168` (« projet », attesté par `supplier-invoices-detail-project`) se
  **supprime**, plus aucune clé ne l'attestant après l'AC-1 ; la note `:172-189` reste, annotée « remplacée le <date de la
  réponse> par l'arbitrage de Guy sur le CR #1161 », suivie de la nouvelle note (date, terme, motif : le CR #1161) ; le
  compte de la partie A se recompte depuis le tableau et s'écrit avec son périmètre.

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
  `README.md:218`, carte E19 de `website/roadmap.html:249-252`, comptes rendus de stories), et les descriptions des
  migrations de l'Epic 19 dans `docs/migrations-idempotence-audit.md:62` et `:65` (« projets analytiques » : elles
  décrivent des fichiers `.sql` appliqués, que P8 interdit de toucher et qui disent eux-mêmes « projet analytique »).
  *Écartée* : réécrire les commentaires — des centaines de lignes, aucun lecteur concerné, et un historique faussé.

  **`failed[].details.message`, hors périmètre, et dit** : le lot de réconciliation y recopie le message interne
  « le projet analytique est archivé » (`reconciliation.rs:754` ; source `kesh-db/src/repositories/projects.rs:119`,
  `invoices.rs:2147`, `supplier_invoices.rs:506`). L'écran ne le lit pas (il affiche le code : `failed-proposal-label.ts:250`),
  mais un client de l'API le reçoit. Il reste, parce que le changer toucherait `kesh-db` et défairait l'appariement
  `reconciliation.rs:763` (`msg.contains("projet")`) qui produit `PROJECT_ARCHIVED`. La note de version et
  `api-external.md` (D9) le disent : seul le code fait foi.

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
  ne changent pas. Le texte libre `failed[].details.message` d'un refus `PROJECT_ARCHIVED` du lot de réconciliation dit
  encore « projet analytique » : seul le code fait foi. » Ses cinq lignes analytiques (§ 6) disent « compte analytique ».
  `docs/i18n-glossaire.md` : selon la réponse de Guy, comme écrit à la fin de D2 (`:105`, `:169` aux termes arbitrés,
  `:168` supprimée, note `:172-189` annotée et suivie de la nouvelle, avec date et motif — le CR #1161 —, partie A
  recomptée). `README.md:47` (Fonctionnalités) dit
  « **Comptabilité analytique** — dimension « compte analytique » … **Dépenses par compte analytique** … **Rendement par
  compte analytique** ». Le site n'a pas de mention actuelle à changer (§ 6).

## Critères d'acceptation

- **AC-1 — Catalogues.** Les 49 clés de § 1 portent, dans les quatre catalogues, les valeurs de D2 ; aucune autre clé
  ne change ; l'ensemble des clés est identique avant et après (`loader.rs::parity_between_locales` vert ; `git diff` des
  quatre `.ftl` ne touche que des lignes de valeur et le commentaire `:3089`). La garde G25 (AC-7) ne trouve, dans la
  **valeur** des messages (le texte après le premier ` = `, et les lignes de continuation) et dans les commentaires `#`
  des quatre catalogues, sans égard à la casse, aucune occurrence de `projet` (fr), `projekt` (de), `progett` (it),
  `\bprojects?\b` (en), hors `en-CH` `dunning-example-heading` (« Projected schedule »), seule exception écrite. Les
  **clés** (`nav-projects`, `projects-title`…) ne sont pas lues (D3).
- **AC-2 — Frontend.** Les 46 sites de § 2 disent « compte analytique » : chaque repli `i18nMsg` est **égal** à la
  nouvelle valeur fr-CH de sa clé ; le repli de nom de fichier (`reports/+page.svelte:433`) égale
  `depenses-par-compte-analytique` / `rendement-par-compte-analytique` ; `journal-entries/[id]/+page.svelte:611` dit
  « Compte analytique ». La garde Vitest (AC-8) ne trouve plus aucun `projet` hors commentaires dans `frontend/src`
  (hors `*.test.ts`) et contrôle l'égalité repli = catalogue sur les 44 appels `i18nMsg` **et** sur les trois sites en
  objet (`+layout.svelte:130` `{ i18nKey, fallback }`, `reports/+page.svelte:527` et `:532` `{ labelKey, fallback }`) et le
  repli de nom de fichier (`:433`, contre `reports-filename-project-*`).
- **AC-3 — Backend.** Les 11 messages de `routes/projects.rs` et les 4 libellés de `ProjectExpensesPdfLabels` /
  `ProjectReturnPdfLabels::fr_ch_defaults()` disent « compte analytique » (D4, D1) ; le nom de fichier d'un export
  `GET /api/v1/reports/project-expenses/export` sur une société fr-CH contient `depenses-par-compte-analytique` (et
  `rendement-par-compte-analytique` pour `project-return`) ; les en-têtes CSV restent `Projet;SousProjet;…` (D5).
- **AC-4 — Rien d'autre ne bouge.** `git diff origin/epic/36-0.18.0 --stat` ne nomme aucun fichier sous
  `crates/kesh-db/`, aucune migration, ni `routes/reconciliation.rs`, ni `kesh-report/src/csv.rs`, ni
  `i18n-keys.test.ts` ; `sitesTotal` reste **3050** (`i18n-keys.test.ts:1020`) ; `npm run lint-i18n-ownership` vert sans
  changement de son registre. Le couplage `reconciliation.rs:763` reste gardé par le test existant
  `other_errors_keep_their_mapping` (`reconciliation.rs:6948`, littéral `:6955` →
  `assert_eq!(f.error_code, "PROJECT_ARCHIVED")`), vert sans modification.
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
  lit, et rougit en **nommant fichier et ligne** à toute occurrence de l'ancien mot, sans égard à la casse :
  - **catalogues** : la valeur seule, comme l'écrit l'AC-1 ;
  - **trois `.tex` français**, après `manuel::sans_commentaires` (`crates/kesh-api/tests/common/manuel.rs:145`, qui
    retire les commentaires LaTeX), `docs/api-external.md` et `README.md` : le texte entier, motif `projet` ;
  - **`routes/projects.rs` et `kesh-report/src/pdf.rs`** : après `sans_modules_de_test` (`textes_coherents.rs:1076`),
    les **seuls littéraux de chaîne** (`"…"`, échappements compris), motif `projet` — les commentaires `//`, `///`,
    `//!` qui disent « projet » (`projects.rs:1`, `:138`, `:141`, `:142`, `:150`… ; `pdf.rs:1170`, `:1173`, `:1233`…)
    restent hors de la garde (D6). L'extracteur de littéraux a son propre test : sur une source synthétique, il rend
    `"Le code du projet est requis."` et ne rend rien de `// projet` ni de `/// projet` (sonde positive et négative) ;
    sur les fichiers d'avant, il rend les 15 littéraux de § 3 (11 + 4).

  Les exceptions forment une **liste fermée, par extrait de texte et non par numéro de ligne** (pour survivre aux
  unions de la 0.18.0), comparés sans égard à la casse, chacun avec son motif. Proposés, relus contre les lignes de
  § 5 : pour l'admin, le projet Docker Compose ou Container Manager — « Container Manager » (`:602`), « onglet
  \textbf{Projet} » (`:608`), « nom du projet » / « nom de projet » (`:612`, `:1676`, `:1766`, `:1975`), « Démarrer le
  projet » (`:617`), « <projet> » (`:1535`, `:2087`, `:2509`, `:2921`), « configuration des projets » (`:2001`),
  « projet compose » (`:2167`), « projets Docker » (`:2198`) — et le `.mcp.json` de Claude Code, « racine du projet »
  (`:3616`) ; « documentation projet » (brochure) ; « Structure du projet », « Le projet suit » (README) ; la ligne historique de l'E19 du tableau
  des versions (extrait « E19 Comptabilité analytique par projet ») ; « Projected schedule ». Chaque extrait doit
  **encore** se rencontrer au moins une fois (un extrait qui ne rencontre plus rien fait rougir) ; une occurrence n'est
  excusée que si sa ligne contient un extrait. La liste exacte se fixe au développement contre les 15 + 1 + 3 + 1 + 1
  lignes de § 5 et § 6, chacune devant être couverte par un extrait, ligne lue citée au Change Log. Une assertion
  positive par fichier prouve qu'il a été lu : dans chaque catalogue, la valeur de `projects-title` contient le terme
  arbitré de sa langue (D2) ; dans `user-manual.tex`, « Comptes analytiques ».
- **AC-8 — Garde Vitest du frontend.** `frontend/src/lib/shared/vocabulaire-analytique.test.ts` parcourt
  `frontend/src/**/*.{svelte,ts,js}` hors `*.test.ts`, masque les commentaires par `masquerCommentaires`, et attend
  **zéro** site `/projet/i` ; il vérifie aussi que chaque repli littéral des 49 clés égale la valeur fr-CH du catalogue
  (`releveDesReplis` / `catalogueDe` de `i18n-repli-catalogue.ts:59`, `:64` — qui ne lisent que les appels `i18nMsg`),
  et, par une lecture dédiée, les quatre sites hors `i18nMsg` de l'AC-2 : `nav-projects` (`i18nKey`/`fallback`),
  `reports-project-expenses` et `reports-project-return` (`labelKey`/`fallback`), et le couple `slugKey` / `slugFallback`
  de `reports/+page.svelte:428-433` ; chacun des quatre doit être **trouvé** (sinon rouge), puis égal à sa valeur fr-CH.
  Sondes positives : une source synthétique `i18nMsg('x', 'Projet archivé')` est détectée ; un objet synthétique
  `{ i18nKey: 'nav-projects', fallback: 'Compte analytique' }` (singulier) rougit à l'égalité.
- **AC-9 — Documentation.** `docs/api-external.md` porte la phrase de D9 et ne dit plus « projet » ; le glossaire porte
  les termes de D2 et le compte recompté de sa partie A ; `README.md:47` dit « compte analytique ».
- **AC-10 — Aucun mot-clé de fermeture** dans les commits
  (`git log origin/epic/36-0.18.0..HEAD --format=%B | /usr/bin/grep -niE '\b(close[sd]?|fix(e[sd])?|resolve[sd]?) #[0-9]'`
  vide) ; aucune ligne de `CHANGELOG.md` (C-e360-11).
- **AC-11 — PDF des manuels, à l'intégration** (condition de la PR de release, D8) : après l'union et `make fr`, les trois
  PDF commités se contrôlent aplatis — `pdftotext docs/manual/fr/<f>.pdf - | tr '\n' ' ' | tr -s ' ' | /usr/bin/grep -c
  '<phrase>'` — avec, pour `user-manual.pdf`, `projet analytique` → 0, `Dépenses par projet` → 0, `Rendement par projet`
  → 0, `compte analytique` ≥ 1 ; pour `admin-manual.pdf` et `marketing-brochure.pdf`, `projets analytiques` → 0 et
  `des projets, les règlements` → 0. La fiche le transmet à l'orchestrateur ; la story ne commite aucun PDF.

## Tests à écrire, et la preuve du rouge

Ordre imposé (E5) : les gardes et tests neufs se commitent **avant** le correctif, se jouent sur les textes d'avant
(rouge), puis le correctif les met au vert. Journaux dans `/home/gcorbaz/devel/kesh-gate-logs/e36-15/`.

| Test | Rouge attendu sans le correctif | Journal |
|---|---|---|
| G25 `le_vocabulaire_analytique_est_compte_analytique` (`textes_coherents.rs`) | sur les fichiers d'avant : liste des sites — 196 valeurs de catalogue et le commentaire `:3089`, 33 lignes de manuel, 5 de `api-external.md`, 1 du README (`:47`), 15 littéraux Rust | `rouges.log` |
| G25 — extracteur de littéraux Rust (test propre) | sonde positive : rend le littéral « projet » ; sonde négative : ne rend rien de `// projet`, `/// projet` | `rouges.log` |
| G25 — mutation 0 : ajouter `// projet` à un commentaire de `projects.rs` | **reste vert** (commentaire non lu) | `mut/g25-commentaire.log` |
| G25 — mutation 1 : réintroduire `projects-title = Projekte` en de-CH | rouge nommant `de-CH/messages.ftl:<ligne>` | `mut/g25-catalogue.log` |
| G25 — mutation 2 : retirer un extrait d'exception (« Container Manager ») de la liste | rouge nommant `admin-manual.tex:<ligne>` | `mut/g25-exception.log` |
| G25 — mutation 2-bis : mettre `projects-title` de en-CH en majuscules « PROJECTS » | rouge (casse ignorée) | `mut/g25-casse.log` |
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

0. ⛔ **Attendre l'arbitrage de Guy sur D2** (de-CH, it-CH, en) ; l'écrire au Change Log (date, option, terme par
   langue) et au registre (C-36-15-1), puis ne garder de D2 que les colonnes retenues.
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
   allemand de l'écran des comptes analytiques sous le terme arbitré (langue de la société passée en de-CH sur la base
   jetable).
9. Fiche à `review`, prompt de la revue P1 à côté de la fiche (lentilles B, E, A ; manuels, PDF local et captures nommés ;
   relire la table de D2 dans les trois langues).

## Note de version

*Modifié* — Kesh parle de **compte analytique** et non plus de projet analytique : le menu, l'écran *Comptes
analytiques*, les champs des écritures, des factures et de la réconciliation, les rapports **Dépenses par compte
analytique** et **Rendement par compte analytique** (titres à l'écran et dans le PDF, noms des fichiers exportés), le
journal d'audit et les manuels, dans les quatre langues (allemand *<terme arbitré>*, italien *<terme arbitré>*,
anglais *<terme arbitré>* — D2, à compléter selon la réponse de Guy). Rien ne change de ce qui est enregistré : routes,
champs et codes de l'API gardent leur nom `project` ; les colonnes `Projet;SousProjet` des exports CSV restent.
⚠️ Les textes des refus de `/api/v1/projects` disent désormais « compte analytique » (codes inchangés) ; le texte libre
`failed[].details.message` d'un refus `PROJECT_ARCHIVED` de la réconciliation dit encore « projet analytique » — seul le
code fait foi. (refs #1161)

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

- Les **trois** `IllegalStateTransition("le projet analytique est archivé")` de production (`kesh-db/src/repositories/
  projects.rs:119`, `invoices.rs:2147`, `supplier_invoices.rs:506`) restent **à l'octet**, ainsi que les deux littéraux de
  test qui les miment (`routes/journal_entries.rs:897`, dans `mod tests` ouvert à `:756` ; `routes/reconciliation.rs:6955`,
  dans `other_errors_keep_their_mapping`) : `reconciliation.rs:763` les reconnaît à `msg.contains("projet")` pour rendre
  `PROJECT_ARCHIVED`.
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
- **2026-11-15 — remédiation de la validation P1** (Opus 5.5 ; rapport `kesh-gate-logs/e36-15/validation-p1.md` :
  0 CRITICAL, 0 HIGH, 3 MEDIUM, 7 LOW ; chaque prémisse lue au code à `54b513a71`).
  - V1-1 (MEDIUM, corrigé) : `projects.rs:1`, `:138`, `:141`, `:142`, `:150` et `pdf.rs:1170`, `:1173`, `:1233` sont des
    commentaires qui disent « projet » (lu) ; G25 ne lit plus côté Rust que les littéraux de chaîne, extracteur testé
    (sondes positive et négative), mutation 0 « un commentaire reste vert » (AC-7, tableau du rouge).
  - V1-2 (MEDIUM, corrigé) : `en-CH/messages.ftl:3044` `nav-projects = …` rougirait sous `\bprojects?\b` (lu) ; G25 ne
    lit que la valeur et les commentaires, casse ignorée, clés exclues (AC-1) ; mutation 2-bis (majuscules).
  - V1-3 (MEDIUM, corrigé sans trancher) : D2 récrite — fr-CH acquis, de-CH / it-CH / en **en attente de l'arbitrage de
    Guy** (options A, B, C ; A recommandée par l'orchestrateur), table clé par clé donnée pour chaque terme ; fondement
    ramené au CR #1161 (la table `projects` n'a pas changé) ; sort des lignes `:105`, `:168` (supprimée), `:169` et de la
    note `:172-189` du glossaire écrit. ⛔ **Le développement ne commence qu'après la réponse de Guy** (tâche 0,
    bandeau d'en-tête).
  - V1-4 (LOW, corrigé) : exceptions de G25 par extrait de texte, chacune devant encore se rencontrer ; les 15 lignes
    courantes de l'admin vérifiées une à une couvertes par un extrait proposé (y compris `:3616`, « racine du projet »,
    que la première version attribuait à tort à `--scope project`).
  - V1-5 (LOW, corrigé) : AC-2 et AC-8 couvrent les trois sites en objet (`i18nKey` / `labelKey` + `fallback`) et le
    couple `slugKey` / `slugFallback`.
  - V1-6 (LOW, corrigé) : `docs/migrations-idempotence-audit.md:62` et `:65` classés hors périmètre (D6), motif P8.
  - V1-7 (LOW, corrigé) : `failed[].details.message` dit hors périmètre avec sa raison (D6), dans la note de version et
    dans la phrase d'`api-external.md` (D9) ; « quatre » `IllegalStateTransition` → trois de production et deux
    littéraux de test (`journal_entries.rs:897`, `mod tests` `:756` ; `reconciliation.rs:6955`).
  - V1-8 (LOW, corrigé) : AC-4 nomme `other_errors_keep_their_mapping` (`reconciliation.rs:6948`).
  - V1-9 (LOW, corrigé) : AC-11, contrôle aplati des trois PDF commités à l'intégration.
  - V1-10 (LOW, **réfuté**) : la fiche cite `en-CH:1478` pour « The analytical project is archived. », et c'est bien
    cette ligne (`sed -n 1478p` : `reconciliation-failed-project-archived = The analytical project is archived.`) ;
    `:1479` est `not-found`, que la fiche ne cite pas. Les autres recoupements du finding confirment la fiche.
  - Édition : le bloc D2 (≈ 75 lignes, tableaux) a été remplacé par une substitution de lignes en shell (`head`/`tail`
    vers un fichier du scratchpad, puis `cp`), artefact `_bmad-output` seulement ; source du bloc versionnée dans
    `kesh-gate-logs/e36-15/d2-v2.md`. Les autres corrections, par l'outil Edit. Gates : aucun (fiche seule).
