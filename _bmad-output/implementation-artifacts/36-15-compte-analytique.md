# Story 36-15 : « compte analytique » au lieu de « projet analytique » (#1161)

Status: done

> **Traductions arbitrées par Guy le 2026-11-16 (D2, option A)** : de-CH « analytisches Konto », it-CH « conto
> analitico », en « analytical account » ; fr-CH « compte analytique ».

*Fiche rédigée le 2026-11-15 par l'agent de rédaction des fiches de l'Epic 36, phase 2 (Opus 5.5), worktree `kesh-36-15`,
branche `story/36-15-compte-analytique`, par le workflow `bmad-create-story`. Release **0.18.0**. **Aucune migration**,
aucune route, aucun champ JSON, aucun code d'erreur ne change. Code lu à **`0bf3c3b4b`** (`origin/epic/36-0.18.0`, le
kickoff de la 0.18.0). Registres : `e36-choix-autonomes-e360.md` (C-e360-15, qui fait foi) ;
`e36-choix-autonomes-36-15-compte-analytique.md` pour la story (C-36-15-1 à 6 à la rédaction ; C-36-15-7 ajouté au développement, C-36-15-8 et 9 à la remédiation de
la revue P1 — annoté le 2026-11-16). Voie **1**. Validation : terminée en trois passes (Change Log).*

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
  `dunning-example-heading = Projected schedule` — « projeté », sans rapport : **hors périmètre**. (Faux positif du seul
  motif brut `project` ; le motif de G25, `\bprojects?\b` sur la valeur, ne l'atteint pas — `awk 'index($0," = ")>0 &&
  $0 ~ /^[a-zA-Z]/ {print substr($0, index($0," = ")+3)}' en-CH/messages.ftl | /usr/bin/grep -ciE '\bprojects?\b'`
  rend **49**.)
- Contrôle de l'autre sens (clés `*project*` dont la valeur ne dit pas « projet ») :
  `/usr/bin/grep -nE '^[a-z0-9-]*project[a-z0-9-]* =' fr-CH/messages.ftl | /usr/bin/grep -viE '= .*projet' | wc -l` rend
  **31** lignes, toutes neutres (« — Aucun », « Compte », « Montant », « Sous-total », « Archiver », « Code »…) :
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

**46 sites, tous au sens analytique** : **41 appels `i18nMsg`** (38 dont le repli est sur la ligne de l'appel, 3 dont le
repli est sur la ligne suivante : `settings/projects/+page.svelte:168`, `export/+page.svelte:67`, `:76`), **3 replis en
objet** (`+layout.svelte:130` `{ i18nKey, fallback }`, `reports/+page.svelte:527`, `:532` `{ labelKey, fallback }`), **le
repli de nom de fichier** `reports/+page.svelte:433`, et **un libellé en dur** — 41 + 3 + 1 + 1 = 46. Recompté par le même
détecteur, en classant chaque ligne (`node <scratchpad>/3615/compte.mjs frontend`, 2026-11-15) :
`sites 46 ; appels i18nMsg 38 ; lignes 'fallback' hors i18nMsg 3 ; autres 5` — les 5 « autres » étant les trois replis
sur ligne suivante, `:433` et `:611`. Le libellé en dur : `journal-entries/[id]/+page.svelte:611` (`<th class="py-2 pr-2">Projet</th>`, voisin de
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
  *(Annoté le 2026-11-16 : faux au développement — `slugify` coupait le type à 20 caractères, d'où C-36-15-7,
  `util.rs` et deux sites de `reports.rs`. Le navigateur, lui, ne lit pas ce nom : il nomme le fichier depuis
  `reports/+page.svelte`, type entier — revue P1, E1-3.)*

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
  `:3229`, `:3253`, `:3756`, la section `\section{Comptabilité analytique par projet}` `:3767` à `:3804` (19 lignes, dont
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

- **D2 — Les traductions : option A, arbitrée par Guy le 2026-11-16** (CR #1161, recommandation 3 ; C-36-15-1) :
  de-CH **analytisches Konto** (n.), it-CH **conto analitico** (m.), en **analytical account**. *Motif* : suit le
  français (règle du glossaire `:29`, « En cas de doute sur une langue cible, suivre le FRANÇAIS ») ; *analytical* est
  l'adjectif que le catalogue anglais employait déjà (`en-CH/messages.ftl:1478`) ; l'option A **prolonge** l'arbitrage du
  2026-08-19 contre *Kostenstelle* (glossaire `:172-189` : « on ne traduit pas « analytique » » devient « on suit le
  français »), elle ne le défait pas. Le terme s'écrit **toujours en entier** : jamais « Konto », « conto » ni
  « account » seul, qui désignent le compte du plan. *Écartées* (soumises à Guy, non retenues) : B `Kostenstelle` /
  `centro di costo` / `cost centre` ; C `Kostenstelle` / `conto analitico` / `analytical account`.

  Formes dérivées :

  | fr-CH | de-CH | it-CH | en-CH |
  |---|---|---|---|
  | pluriel | analytische Konten | conti analitici | analytical accounts |
  | … parent | übergeordnetes analytisches Konto | conto analitico superiore | parent analytical account |
  | sous-… | analytisches Unterkonto | sottoconto analitico | analytical sub-account |
  | … racine | analytisches Hauptkonto | conto analitico radice | root analytical account |

  *Source* : aucun précédent au catalogue (`git grep -n -iE "kostenst|kostentr|conto analitico|analytical account"` ne
  rendait que la note du glossaire, `docs/i18n-glossaire.md:176-186`) ; les termes sont ceux que propose le CR #1161. **Le
  fondement est le CR de Guy**, non un changement de l'objet : la table `projects` porte toujours code, nom, deux niveaux
  et dates (`20260702000001_projects_analytics.sql`).

  **Valeurs, clé par clé.** Les deux clés d'export ne changent qu'un mot de leur liste (en gras).

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

  **de-CH** (option A) :

  | Clé | de-CH |
  |---|---|
  | `journal-entry-form-col-project`, `reconciliation-split-th-project`, `reports-project-return-col-project`, `reports-project-selector-label`, `invoice-detail-project-label`, `supplier-invoices-detail-project`, `audit-log-entity-project` | Analytisches Konto |
  | `journal-entry-project-archived`, `reconciliation-rules-default-project-archived`, `invoice-project-archived`, `audit-log-action-project-archived` | Analytisches Konto archiviert |
  | `reconciliation-failed-project-archived` | Das analytische Konto ist archiviert. |
  | `reconciliation-failed-project-not-found` | Analytisches Konto nicht gefunden. |
  | `reconciliation-manual-project-label`, `invoice-field-project`, `supplier-invoices-field-project` | Analytisches Konto (optional) |
  | `reconciliation-rules-labels-default-project` | Analytisches Standardkonto |
  | `reports-project-expenses`, `-expenses-title` | Ausgaben pro analytisches Konto *(accusatif après* pro *: arbitrage de Guy du 2026-11-16, revue P1 B1-4 ; « analytischem » jusque-là)* |
  | `reports-project-expenses-empty` | Für diesen Zeitraum sind diesem analytischen Konto keine Ausgaben zugeordnet. |
  | `reports-project-return`, `-return-title` | Rendite pro analytisches Konto *(idem, B1-4)* |
  | `reports-project-return-empty` | Für diesen Zeitraum sind diesem analytischen Konto keine Bewegungen zugeordnet. |
  | `reports-project-selector-placeholder` | — Analytisches Konto wählen |
  | `reports-filename-project-expenses` | ausgaben-pro-analytisches-konto *(B1-4)* |
  | `reports-filename-project-return` | rendite-pro-analytisches-konto *(B1-4)* |
  | `export-global-description`, `export-global-content-includes` | … **analytische Konten** … |
  | `invoice-project-current` | Aktuelles analytisches Konto |
  | `nav-projects`, `projects-title` | Analytische Konten |
  | `projects-archived` | Analytisches Konto archiviert. |
  | `projects-created`, `audit-log-action-project-created` | Analytisches Konto erstellt(.) |
  | `projects-empty` | Kein analytisches Konto. Erstellen Sie Ihr erstes analytisches Konto, um zu beginnen. |
  | `projects-err-code` | Der Code des analytischen Kontos ist erforderlich. |
  | `projects-err-name` | Der Name des analytischen Kontos ist erforderlich. |
  | `projects-field-parent` | Übergeordnetes analytisches Konto (optional) |
  | `projects-form-create`, `projects-new` | Neues analytisches Konto |
  | `projects-form-edit` | Analytisches Konto bearbeiten |
  | `projects-load-error` | Analytische Konten können nicht geladen werden. |
  | `projects-parent-none` | — Keines (analytisches Hauptkonto) |
  | `projects-show-archived` | Archivierte analytische Konten anzeigen |
  | `projects-subtitle` | Fassen Sie Ihre Ausgaben und Erträge nach analytischem Konto zusammen (eine Liegenschaft, ein Fahrzeug, eine Renovation, eine Tätigkeit), um sie separat auszuwerten. |
  | `projects-unarchived` | Analytisches Konto dearchiviert. |
  | `projects-updated`, `audit-log-action-project-updated` | Analytisches Konto aktualisiert(.) |
  | `audit-log-action-project-unarchived` | Analytisches Konto reaktiviert |

  **it-CH** (option A) :

  | Clé | it-CH |
  |---|---|
  | `journal-entry-form-col-project`, `reconciliation-split-th-project`, `reports-project-return-col-project`, `reports-project-selector-label`, `invoice-detail-project-label`, `supplier-invoices-detail-project`, `audit-log-entity-project` | Conto analitico |
  | `journal-entry-project-archived`, `reconciliation-rules-default-project-archived`, `invoice-project-archived`, `audit-log-action-project-archived` | Conto analitico archiviato |
  | `reconciliation-failed-project-archived` | Il conto analitico è archiviato. |
  | `reconciliation-failed-project-not-found` | Conto analitico introvabile. |
  | `reconciliation-manual-project-label`, `invoice-field-project`, `supplier-invoices-field-project` | Conto analitico (facoltativo) |
  | `reconciliation-rules-labels-default-project` | Conto analitico predefinito |
  | `reports-project-expenses`, `-expenses-title` | Spese per conto analitico |
  | `reports-project-expenses-empty` | Nessuna spesa associata a questo conto analitico per il periodo. |
  | `reports-project-return`, `-return-title` | Rendimento per conto analitico |
  | `reports-project-return-empty` | Nessun movimento associato a questo conto analitico per il periodo. |
  | `reports-project-selector-placeholder` | — Scegli un conto analitico |
  | `reports-filename-project-expenses` | spese-per-conto-analitico |
  | `reports-filename-project-return` | rendimento-per-conto-analitico |
  | `export-global-description`, `export-global-content-includes` | … **conti analitici** … |
  | `invoice-project-current` | Conto analitico attuale |
  | `nav-projects`, `projects-title` | Conti analitici |
  | `projects-archived` | Conto analitico archiviato. |
  | `projects-created`, `audit-log-action-project-created` | Conto analitico creato(.) |
  | `projects-empty` | Nessun conto analitico. Crea il tuo primo conto analitico per iniziare. |
  | `projects-err-code` | Il codice del conto analitico è obbligatorio. |
  | `projects-err-name` | Il nome del conto analitico è obbligatorio. |
  | `projects-field-parent` | Conto analitico superiore (facoltativo) |
  | `projects-form-create`, `projects-new` | Nuovo conto analitico |
  | `projects-form-edit` | Modifica il conto analitico |
  | `projects-load-error` | Impossibile caricare i conti analitici. |
  | `projects-parent-none` | — Nessuno (conto analitico radice) |
  | `projects-show-archived` | Mostra i conti analitici archiviati |
  | `projects-subtitle` | Raggruppa le tue spese e i tuoi ricavi per conto analitico (un immobile, un veicolo, una ristrutturazione, un'attività) per analizzarli separatamente. |
  | `projects-unarchived` | Conto analitico ripristinato dall'archivio. |
  | `projects-updated`, `audit-log-action-project-updated` | Conto analitico aggiornato(.) |
  | `audit-log-action-project-unarchived` | Conto analitico riattivato |

  **en-CH** (option A) :

  | Clé | en-CH |
  |---|---|
  | `journal-entry-form-col-project`, `reconciliation-split-th-project`, `reports-project-return-col-project`, `reports-project-selector-label`, `invoice-detail-project-label`, `supplier-invoices-detail-project`, `audit-log-entity-project` | Analytical account |
  | `journal-entry-project-archived`, `reconciliation-rules-default-project-archived`, `invoice-project-archived`, `audit-log-action-project-archived` | Analytical account archived |
  | `reconciliation-failed-project-archived` | The analytical account is archived. |
  | `reconciliation-failed-project-not-found` | Analytical account not found. |
  | `reconciliation-manual-project-label`, `invoice-field-project`, `supplier-invoices-field-project` | Analytical account (optional) |
  | `reconciliation-rules-labels-default-project` | Default analytical account |
  | `reports-project-expenses`, `-expenses-title` | Expenses by analytical account |
  | `reports-project-expenses-empty` | No expenses tagged on this analytical account for the period. |
  | `reports-project-return`, `-return-title` | Return by analytical account |
  | `reports-project-return-empty` | No movements tagged on this analytical account for the period. |
  | `reports-project-selector-placeholder` | — Choose an analytical account |
  | `reports-filename-project-expenses` | expenses-by-analytical-account |
  | `reports-filename-project-return` | return-by-analytical-account |
  | `export-global-description`, `export-global-content-includes` | … **analytical accounts** … |
  | `invoice-project-current` | Current analytical account |
  | `nav-projects`, `projects-title` | Analytical accounts |
  | `projects-archived` | Analytical account archived. |
  | `projects-created`, `audit-log-action-project-created` | Analytical account created(.) |
  | `projects-empty` | No analytical account. Create your first analytical account to get started. |
  | `projects-err-code` | The analytical account code is required. |
  | `projects-err-name` | The analytical account name is required. |
  | `projects-field-parent` | Parent analytical account (optional) |
  | `projects-form-create`, `projects-new` | New analytical account |
  | `projects-form-edit` | Edit analytical account |
  | `projects-load-error` | Unable to load the analytical accounts. |
  | `projects-parent-none` | — None (root analytical account) |
  | `projects-show-archived` | Show archived analytical accounts |
  | `projects-subtitle` | Group your expenses and revenue by analytical account (a building, a vehicle, a renovation, an activity) to analyse them separately. |
  | `projects-unarchived` | Analytical account unarchived. |
  | `projects-updated`, `audit-log-action-project-updated` | Analytical account updated(.) |
  | `audit-log-action-project-unarchived` | Analytical account reactivated |

  « (.) » : le point final ne figure que dans la clé `projects-*` (toast), non dans la clé `audit-log-action-*`, comme
  aujourd'hui. Le commentaire `fr-CH/messages.ftl:3089` dit la convention retenue (`« compte analytique » = analytisches Konto /
  conto analitico / analytical account`). Une phrase qu'un accord rendrait bancale se corrige au développement, en gardant le terme,
  et se déclare au Change Log. **Le glossaire** suit l'arbitrage (D9) : `:105` (« projet parent ») et `:169` (« projet
  analytique ») passent aux termes arbitrés ; `:168` (« projet », attesté par `supplier-invoices-detail-project`) se
  **supprime**, plus aucune clé ne l'attestant après l'AC-1 ; la note `:172-189` reste, annotée « prolongée le 2026-11-16 par
  l'arbitrage de Guy sur le CR #1161 », suivie de la nouvelle note (date, terme, motif : le CR #1161) ; le
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
  ne changent pas. Le texte libre `failed[].details.message` d'un refus `PROJECT_ARCHIVED` du lot de réconciliation
  reprend le libellé interne de Kesh, en français et non revu : seul le code fait foi. » La phrase ne contient pas le mot
  « projet » : G25 n'a besoin d'aucune exception pour `api-external.md`. Ses cinq lignes analytiques (§ 6) disent « compte analytique ».
  `docs/i18n-glossaire.md` : selon l'arbitrage de Guy (option A), comme écrit à la fin de D2 (`:105`, `:169` aux termes arbitrés,
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
  `\bprojects?\b` (en), **sans aucune exception** (« Projected schedule » n'est pas atteint par `\b`, § 1). Les
  **clés** (`nav-projects`, `projects-title`…) ne sont pas lues (D3).
- **AC-2 — Frontend.** Les 46 sites de § 2 disent « compte analytique » : chaque repli `i18nMsg` est **égal** à la
  nouvelle valeur fr-CH de sa clé ; le repli de nom de fichier (`reports/+page.svelte:433`) égale
  `depenses-par-compte-analytique` / `rendement-par-compte-analytique` ; `journal-entries/[id]/+page.svelte:611` dit
  « Compte analytique ». La garde Vitest (AC-8) ne trouve plus aucun `projet` hors commentaires dans `frontend/src`
  (hors `*.test.ts`) et contrôle l'égalité repli = catalogue sur les 41 appels `i18nMsg` **et** sur les trois sites en
  objet (`+layout.svelte:130` `{ i18nKey, fallback }`, `reports/+page.svelte:527` et `:532` `{ labelKey, fallback }`) et le
  repli de nom de fichier (`:433`, contre `reports-filename-project-*`).
- **AC-3 — Backend.** Les 11 messages de `routes/projects.rs` et les 4 libellés de `ProjectExpensesPdfLabels` /
  `ProjectReturnPdfLabels::fr_ch_defaults()` disent « compte analytique » (D4, D1) ; le nom de fichier d'un export
  `GET /api/v1/reports/project-expenses/export` sur une société fr-CH contient `depenses-par-compte-analytique` (et
  `rendement-par-compte-analytique` pour `project-return`) ; les en-têtes CSV restent `Projet;SousProjet;…` (D5).
- **AC-4 — Rien d'autre ne bouge.** `git diff origin/epic/36-0.18.0 --stat` ne nomme aucun fichier sous
  `crates/kesh-db/`, aucune migration, ni `routes/reconciliation.rs`, ni `kesh-report/src/csv.rs`, ni
  `i18n-keys.test.ts` ; `sitesTotal` reste **3050** (`i18n-keys.test.ts:1020`) ; `npm run lint-i18n-ownership` vert sans
  changement de son registre. Le couplage `reconciliation.rs:763` est tenu, de bout en bout, par **la clause « aucun
  fichier sous `crates/kesh-db/` »** ci-dessus (les trois textes de production y vivent) et par le test existant
  `crates/kesh-api/tests/reconciliation_rules_e2e.rs:1810`
  (`assert_eq!(failed[0]["errorCode"].as_str(), Some("PROJECT_ARCHIVED"));`, compte analytique réellement archivé), vert
  sans modification. `other_errors_keep_their_mapping` (`reconciliation.rs:6948`, littéral `:6955`) ne tient que la
  fonction d'appariement : il fournit son propre littéral et resterait vert si `kesh-db` changeait de texte.
- **AC-5 — E2E.** `reports.spec.ts` ouvre les onglets **par leurs nouveaux noms** (`/dépenses par compte analytique/i`,
  `/rendement par compte analytique/i`) et y vérifie, en plus, que le titre de la vue rendue commence par ce nom ; les deux
  tests rougissent sur les catalogues d'avant (AC de preuve, § Tests).
- **AC-6 — Manuels.** Les 33 lignes analytiques de § 5 disent « compte analytique » (`user-manual.tex` : section
  `\section{Comptabilité analytique}`, chemin « \emph{Administration} → \emph{Comptes analytiques} », les deux rapports
  sous leurs nouveaux noms, une phrase qui dit ce qu'un compte analytique peut suivre — un immeuble, un véhicule, une
  activité, un chantier) ; les 16 lignes au sens courant restent ; sur le PDF compilé localement (D8),
  `pdftotext user-manual.pdf - | tr '\n' ' ' | tr -s ' ' | /usr/bin/grep -oi 'projet' | wc -l` rend 0 et `… | /usr/bin/grep
  -c 'compte analytique'` rend au moins 1, relevés au Change Log avec leur commit.
- **AC-7 — Garde de texte backend et documentation (G25).** Un test de `crates/kesh-api/tests/textes_coherents.rs`
  lit, et rougit en **nommant fichier et ligne** à toute occurrence de l'ancien mot, sans égard à la casse :
  - **catalogues** : la valeur seule, comme l'écrit l'AC-1 ;
  - **trois `.tex` français**, après `manuel::sans_commentaires` (`crates/kesh-api/tests/common/manuel.rs:145`, qui
    retire les commentaires LaTeX), `docs/api-external.md` et `README.md` : le texte entier, motif `projet` ;
  - **`routes/projects.rs` et `kesh-report/src/pdf.rs`** : après `sans_modules_de_test` (`textes_coherents.rs:1076`),
    les **seuls littéraux de chaîne** (`"…"`, échappements compris), motif `projet` — les commentaires `//`, `///`,
    `//!` qui disent « projet » (`projects.rs:1`, `:138`, `:141`, `:142`, `:150`… ; `pdf.rs:1170`, `:1173`, `:1233`…)
    restent hors de la garde (D6). L'extracteur de littéraux a son propre test, sur des sources synthétiques qui
    reprennent les formes présentes dans les deux fichiers :
    - sondes positives : `"Le code du projet est requis."` est rendu ; un littéral **continué** par `\` en fin de ligne
      (forme de `pdf.rs:603-604`, `"… dans la section \` / `equity (…)"`), dont la seconde ligne dit « projet », est
      rendu en entier ;
    - sondes négatives : rien n'est rendu de `// projet`, `/// projet`, ni d'un **doc-comment à guillemets**
      `/// renvoie "projet"` (forme de `pdf.rs:61` ``/// Code locale BCP-47 (e.g. `"fr-CH"`)``, `:67`, `:242-244`,
      `:885` `// Section vide : on note "—"`, `:1062`) — l'extracteur saute les commentaires `//` avant d'apparier les
      guillemets ;
    - sur les fichiers d'avant, il rend les 15 littéraux de § 3 (11 + 4).

    **Angles morts assumés** : chaînes brutes (`r"…"`, `r#"…"#`), caractère `'"'`, commentaires `/* … */`. Absents des
    deux fichiers aujourd'hui : `/usr/bin/grep -nE '\br#*"' crates/kesh-api/src/routes/projects.rs
    crates/kesh-report/src/pdf.rs`, `/usr/bin/grep -nF "'\"'" …` et `/usr/bin/grep -nF '/*' …`, sur les fichiers entiers,
    sont vides (relevé du 2026-11-15 à `c513cc0a1`, recoupant la validation P2) ; le développement rejoue ces trois commandes et les cite au Change Log.

  Les exceptions forment une **liste fermée, par extrait de texte et non par numéro de ligne** (pour survivre aux
  unions de la 0.18.0), comparés sans égard à la casse, chacun avec son motif et **attaché à son seul fichier** : un
  extrait de l'admin n'excuse rien dans `user-manual.tex` ni ailleurs. Aucune exception pour les catalogues (AC-1), pour
  `api-external.md` (D9), ni pour les deux fichiers Rust. Proposés, relus contre les lignes de § 5 : pour l'admin, le projet Docker Compose ou Container Manager — « Container Manager » (`:602`), « onglet
  \textbf{Projet} » (`:608`), « nom du projet » / « nom de projet » (`:612`, `:1676`, `:1766`, `:1975`), « Démarrer le
  projet » (`:617`), « <projet> » (`:1535`, `:2087`, `:2509`, `:2921`), « configuration des projets » (`:2001`),
  « projet compose » (`:2167`), « projets Docker » (`:2198`) — et le `.mcp.json` de Claude Code, « racine du projet »
  (`:3616`) ; « documentation projet » (brochure) ; « Structure du projet », « Le projet suit » (README) ; la ligne historique de l'E19 du tableau
  des versions (extrait « E19 Comptabilité analytique par projet »). Une occurrence du motif n'est excusée que si **sa
  ligne**, dans **son fichier**, contient un extrait attaché à ce fichier. « Doit encore se rencontrer » veut dire :
  chaque extrait **excuse au moins une occurrence du motif** dans son fichier — un extrait qui n'excuse plus rien (la
  ligne a disparu, ou ne dit plus « projet ») fait rougir en le nommant, ce qui tient la liste fermée et à jour. La liste exacte se fixe au développement contre les 15 + 1 + 3 + 1 (admin, brochure, README courant, README historique)
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
  PDF commités se contrôlent aplatis, chaque contrôle étant **non vide aujourd'hui** (relevé à `c513cc0a1`) :
  - `user-manual.pdf` : `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' | /usr/bin/grep -oi 'projet' | wc -l`
    → **0** (40 aujourd'hui). Le manuel utilisateur n'a **aucune** ligne au sens courant (§ 5 : ses 30 lignes sont
    analytiques), d'où le motif nu, casse ignorée, qui voit aussi « Projets analytiques » et « sous-projets » ;
  - `admin-manual.pdf` : `… | /usr/bin/grep -c 'projet compris'` (`:2787`) → **0** et `… | /usr/bin/grep -c 'des projets,
    les règlements'` (`:3509`) → **0** (1 et 1 aujourd'hui). Les 15 lignes au sens courant (Docker Compose, Container
    Manager, `.mcp.json`) gardent leur « projet » : elles ne sont pas contrôlées, G25 les tient sur le `.tex` ;
  - `marketing-brochure.pdf` : `… | /usr/bin/grep -c 'projets analytiques y seront'` (`:195`) → **0** (1 aujourd'hui) ;
    « documentation projet » (`:498`, sens courant) reste, non contrôlée ;
  - et `… | /usr/bin/grep -c 'compte analytique'` ≥ 1 sur `user-manual.pdf` (0 aujourd'hui).

  La fiche le transmet à l'orchestrateur ; la story ne commite aucun PDF.

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
| G25 — mutation 3 : ajouter à l'admin un extrait qui n'excuse aucune occurrence (« projet fictif ») | rouge « exception périmée » nommant l'extrait | `mut/g25-perimee.log` |
| G25 — mutation 3-bis : écrire « nom du projet » dans une ligne analytique de `user-manual.tex` | rouge (l'extrait de l'admin ne vaut pas pour ce fichier) | `mut/g25-portee.log` |
| `vocabulaire-analytique.test.ts` | 46 sites sur les sources d'avant | `rouges.log` |
| idem — mutation : rendre un repli `'Projet archivé'` (`InvoiceForm.svelte:830`) | rouge sur ce site **et** sur l'écart repli ≠ catalogue | `mut/vitest-repli.log` |
| `project_reports_e2e.rs` : `export_filenames_say_compte_analytique` (neuf) | `Content-Disposition` contient `depenses-par-projet` | `rouges.log` |
| `reports.spec.ts` (deux tests modifiés) | onglet introuvable sur les catalogues d'avant (spec seule, voie 1, backend lancé pour la preuve) | `rouges-e2e.log` |

Restauration après mutation : `git restore --worktree -- <fichier>` puis `touch <fichier>` ; scripts de mutation dans
`/home/gcorbaz/devel/kesh-gate-logs/e36-15/mut/`.

**Pourquoi pas de test unitaire de `fr_ch_defaults()`** : G25 lit `pdf.rs` et rougit sur ses quatre textes ; un test qui
recopierait la valeur attendue ne prouverait rien de plus.

## Tâches

0. ~~Attendre l'arbitrage de Guy sur D2~~ — reçu le 2026-11-16 (option A) ; l'écrire au Change Log (date, option, terme par
   langue) et au registre (C-36-15-1), puis ne garder de D2 que les colonnes retenues.
1. **Étape 0** : relire C-e360-15, ce que dit le CR, et `git log origin/epic/36-0.18.0..HEAD` ; vérifier que
   `crates/kesh-db/` reste hors du diff prévu. Base `kesh_e3615` et `kesh_e2e_e3615` créées à neuf (voie 1).
2. **Gardes d'abord** (AC-7, AC-8, test neuf de `project_reports_e2e.rs`, sélecteurs de `reports.spec.ts`), commit ;
   rouges journalisés.
3. **Catalogues** (AC-1) par l'outil Edit, **par blocs contigus** (`projects-*` :3171-3196, `reports-project-*`
   :1944-1964, `audit-log-action-project-*` :3362-3365, `invoice-*` :2659-2662…) pour borner le nombre d'éditions ; fr-CH,
   puis de-CH, it-CH, en-CH, clé par clé contre la table de D2.
4. **Frontend** (AC-2) : 46 sites.
5. **Backend** (AC-3) : `routes/projects.rs` (11), `kesh-report/src/pdf.rs` (4) ; `cargo fmt`. *(Annoté le
   2026-11-16 : s'y sont ajoutés au développement `crates/kesh-api/src/util.rs` (`slugify_max`) et deux sites de
   `crates/kesh-api/src/routes/reports.rs` — C-36-15-7.)*
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
journal d'audit et les manuels, dans les quatre langues (allemand *analytisches Konto*, italien *conto analitico*,
anglais *analytical account*). Rien ne change de ce qui est enregistré : routes,
champs et codes de l'API gardent leur nom `project` ; les colonnes `Projet;SousProjet` des exports CSV restent. Le nom
d'un fichier exporté de ces deux rapports change donc (`kesh-depenses-par-compte-analytique-…` au lieu de
`kesh-depenses-par-projet-…`). Pour un client de l'API seulement, le nom que donne l'en-tête `Content-Disposition`
d'un export de rapport ne coupe plus le type du rapport à vingt caractères, mais à quarante (le nom de la société reste
coupé à vingt) ; le fichier téléchargé depuis l'écran portait déjà le type entier.
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
- `sitesTotal` 3050 inchangé ; les 31 clés `*project*` neutres (« — Aucun », « Compte »…) inchangées.
- Pas de migration (C-e360-3) : la story n'en demande aucune.

### Références

- CR #1161 ; C-e360-15 ; fichier d'epic, phase 2, rang 14 et « Phase 2 — décisions de Guy du 2026-11-15 ».
- `docs/i18n-glossaire.md:168-189` (l'arbitrage remplacé) ; `docs/testing.md` § « Les échecs attendus ».
- `frontend/src/lib/shared/i18n-repli-catalogue.ts` (`ecartsAuCatalogue`, angle mort #758) ;
  `frontend/src/lib/shared/i18n-literal-reader.js:261` (`masquerCommentaires`).

## Dev Agent Record

### Agent Model Used

Opus 5.5 (agent de développement de l'Epic 36, phase 2), 2026-11-16, en autonomie, worktree `kesh-36-15`, voie 1.

### Debug Log References

Journaux sous `/home/gcorbaz/devel/kesh-gate-logs/e36-15/` :
- `rouges.log` — G25 et `export_filenames_say_compte_analytique` rouges au commit des gardes `f5f6ff88b` (catalogues
  d'avant) : G25 **256 occurrences** = 197 lignes de catalogue (196 valeurs + le commentaire fr-CH `:3089`) + 5 preuves
  de lecture (`projects-title` ×4, « Comptes analytiques » du manuel) + 39 lignes de documents (30 du manuel utilisateur,
  2 de l'admin, 1 de la brochure, 5 d'`api-external.md`, 1 du README) + 15 littéraux Rust (11 + 4) ; puis Vitest
  `vocabulaire-analytique` rouge, **46 sites** (1 test rouge sur 5 : l'égalité repli = catalogue était encore vraie sur
  les deux côtés d'avant).
- `rouges-e2e.log` — `reports.spec.ts` sur les catalogues d'avant (`f5f6ff88b`, backend voie 1 port 3615) : les deux
  tests modifiés rouges (onglet `/dépenses par compte analytique/i`, `/rendement par compte analytique/i` introuvable),
  12 autres verts.
- `cible-1.log` (arbre sur `f5f6ff88b`, correctif non commité) : 318/319, `export_filenames_say_compte_analytique` rouge
  — le nom rendu était `kesh-depenses-par-compte-…` : la coupe à 20 caractères de `slugify` (C-36-15-7).
- `cible-2.log` (même arbre, C-36-15-7 appliqué) : gate ciblé `binary(textes_coherents) | binary(project_reports_e2e) |
  package(kesh-i18n) | package(kesh-report) | binary(projects_e2e) | binary(reconciliation_rules_e2e) | (package(kesh-api)
  & kind(lib))` **1000/1000**, 6 ignorés ; `clippy-dev-2.log` vert ; `cargo fmt --check` vert.
- `vitest-cible-1.log` : `src/lib/shared/` **389/389** (30 fichiers).
- `mut/` — mutations au commit `894267d82` (code commité avant, E5) : `g25-commentaire` (commentaire `// projet` ajouté à
  `projects.rs`) **reste vert** ; `g25-catalogue` (de-CH `projects-title = Projekte`) rouge nommant
  `de-CH/messages.ftl:3115` ; `g25-casse` (en-CH `PROJECTS`) rouge nommant `en-CH/messages.ftl:3121` ; `g25-exception`
  (extrait « Container Manager » retiré) rouge nommant `admin-manual.tex:602` ; `g25-perimee` (extrait « projet fictif »
  ajouté à l'admin) rouge « exception périmée » ; `g25-portee` (« nom du projet » dans une ligne du manuel utilisateur)
  rouge nommant `user-manual.tex:3253` ; `vitest-repli` (`'Projet archivé'` remis dans `InvoiceForm.svelte`) rouge sur le
  site **et** sur l'écart repli ≠ catalogue. Restauration par `git restore --worktree` + `touch`, constatée dans chaque
  journal. Scripts : `mut/g25-mutations.sh`, `mut/vitest-repli.sh`.
- `make-fr.log`, `pdf-local.log` — `make fr` au commit `894267d82`, PDF **non commités** (D8, rendus par `git restore`) :
  `user-manual.pdf` `grep -oi 'projet' | wc -l` **0** (40 avant), `grep -c 'compte analytique'` **1** (ligne unique
  aplatie ; 30 occurrences), `grep -c 'Comptes analytiques'` 1 ; admin `projet compris` 0, `des projets, les règlements` 0,
  `compte analytique compris` 1 ; brochure `projets analytiques y seront` 0, `comptes analytiques y seront` 1.
- `test-fast-ci-review.log` — `scripts/test-fast.sh --ci`, base `kesh_e3615` remise à neuf, au commit `894267d82` :
  **5456 passés, 13 ignorés**, 1 lent (`recette_montee_script`, 100 s).
- `gate-ci-review.log` — premier `scripts/gate-ci.sh kesh_ci_e3615` au commit `894267d82`, **interrompu** (panne du
  classificateur de sécurité de la session, arrêt de l'agent ; aucun échec au moment de l'arrêt) ; relancé sous les mêmes
  verrous : `gate-ci-review-2.log` — **5463 passés, 0 échoué, 17 ignorés**, `rc=0`, commit `894267d82`.
- `front-{check,lint-i18n-ownership,test-unit,build}-review.log` — frontend complet au commit `894267d82` : `npm run
  check` 0 erreur (29 avertissements, 5 fichiers) ; `lint-i18n-ownership` PASS ; `test:unit` **3714/3714** (239
  fichiers) ; `build` vert.
- `captures.log`, `captures/` — captures (base jetable `kesh_e2e_e3615`, preset `with-data`, thème clair).

### Completion Notes List

- **Option A appliquée** (arbitrage de Guy du 2026-11-16) : 49 valeurs × 4 catalogues + le commentaire `fr-CH:3089`
  (197 lignes de valeur, aucune clé ajoutée, retirée ni renommée ; `git diff` : 197 insertions, 197 suppressions) ; 46
  sites du frontend ; 11 refus de `/api/v1/projects` ; 4 libellés du PDF ; documentation (trois `.tex`,
  `api-external.md` avec la phrase de D9, glossaire, `README.md:47`).
- **C-36-15-7, hors fiche** : le type du nom de fichier exporté n'est plus coupé à 20 caractères
  (`slugify_max`, `LONGUEUR_MAX_SLUG_DE_TYPE = 40`, `crates/kesh-api/src/util.rs` ; deux sites de `routes/reports.rs`).
  Coupé, il disait `depenses-par-compte` — « compte » seul. Aucun autre type ne dépasse 20 caractères dans les quatre
  langues : aucun autre nom de fichier ne change. Le fichier `routes/reports.rs` n'est pas dans la liste d'exclusion de
  l'AC-4.
- **G25 côté Rust plus large que la fiche** : l'extracteur `litteraux_de_chaine` lit aussi les chaînes brutes, saute les
  commentaires `/* */` et les caractères `'"'` ; les trois angles morts de l'AC-7 sont donc couverts (sondes dans
  `g25_l_extracteur_de_litteraux_de_chaine`) ; reste l'angle mort des commentaires de bloc imbriqués (doc-comment de la
  fonction). Les trois commandes de l'AC-7 (`\br#*"`, `'"'`, `/*`) rejouées sur les deux fichiers au commit `894267d82` :
  vides.
- **Exceptions de G25** : 14 extraits (10 admin, 1 brochure, 3 README), chacun attaché à son fichier ; les 15 lignes
  courantes de l'admin, la ligne de la brochure et les 4 du README sont toutes couvertes (le test le prouve : aucune
  occurrence non excusée, aucune exception périmée).
- **Glossaire** : partie A **82** lignes de tableau (83 avant ; ligne « projet » retirée), recomptées par la commande
  écrite dans la note du glossaire ; le « 76 » de la note de la 23-4 est laissé (relevé daté).
- **Manuels** : les PDF ne sont pas commités (D8) ; l'AC-11 reste à l'intégration.
- **Captures** (`/home/gcorbaz/devel/kesh-gate-logs/e36-15/captures/`) : `00-menu-lateral-{fr,de}`,
  `01-comptes-analytiques-liste-{fr,de}` (allemand : `KESH_LANG=de-CH`), `02-…-formulaire-fr`,
  `03-…-refus-sous-comptes-actifs-fr` (toast du refus d'archiver une racine qui a des sous-comptes actifs),
  `04-ecriture-formulaire-colonne-fr`, `05-ecriture-detail-colonne-fr` (`<th>` en dur, D7), `06-facture-client-champ-fr`,
  `07-facture-fournisseur-champ-fr`, `08-reconciliation-regle-compte-par-defaut-fr`, `09-rapport-depenses-fr`,
  `10-rapport-rendement-fr`, les deux PDF exportés et leur page 1 (`09-…-export.pdf`, `10-…-export.pdf`,
  `*-export-page-1.png`), `11-journal-audit-fr`. **Non capturés** : les dialogues de réconciliation *écriture manuelle* et
  *ventilation* (ils demandent un relevé importé ; leurs textes sont tenus par G25 et la garde Vitest) ; le refus
  « parent archivé » (l'écran n'offre pas un parent archivé ; le refus capturé est celui de l'archivage, du même fichier).
  Remarque : les onglets des rapports passent sur deux lignes (« Dépenses par compte / analytique »), lisibles.
- **Pas d'E2E complet** au passage en `review` (consigne) : il tourne à la clôture, avec `gate-ci`. Seule la spec
  `reports.spec.ts` a tourné, pour la preuve du rouge.

### File List

- `crates/kesh-i18n/locales/{fr-CH,de-CH,it-CH,en-CH}/messages.ftl`
- `crates/kesh-api/src/routes/projects.rs`, `crates/kesh-api/src/routes/reports.rs`, `crates/kesh-api/src/util.rs`
- `crates/kesh-report/src/pdf.rs`
- `crates/kesh-api/tests/textes_coherents.rs` (G25), `crates/kesh-api/tests/project_reports_e2e.rs`
- `frontend/src/lib/shared/vocabulaire-analytique.test.ts` (neuf)
- `frontend/src/lib/components/invoices/InvoiceForm.svelte`,
  `frontend/src/lib/features/journal-entries/JournalEntryForm.svelte`,
  `frontend/src/lib/features/reconciliation/{ManualMatchModal.svelte,TransactionSplitModal.svelte,failed-proposal-label.ts}`,
  `frontend/src/lib/features/reconciliation/rules/RuleFormModal.svelte`,
  `frontend/src/lib/features/reports/{ProjectExpensesView.svelte,ProjectReturnView.svelte}`,
  `frontend/src/lib/features/supplier-invoices/SupplierInvoiceForm.svelte`,
  `frontend/src/routes/(app)/{+layout.svelte,export/+page.svelte,invoices/[id]/+page.svelte,journal-entries/[id]/+page.svelte,reports/+page.svelte,settings/projects/+page.svelte,supplier-invoices/[id]/+page.svelte}`
- `frontend/tests/e2e/reports.spec.ts`
- `docs/manual/fr/{user-manual,admin-manual,marketing-brochure}.tex`, `docs/api-external.md`, `docs/i18n-glossaire.md`,
  `README.md`
- `_bmad-output/implementation-artifacts/36-15-compte-analytique.md`, `…-review-prompt-p1.md` (neuf),
  `e36-choix-autonomes-36-15-compte-analytique.md` (C-36-15-7), `sprint-status.yaml`

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
- **2026-11-15 — remédiation de la validation P2** (Opus 5.5 ; rapport `kesh-gate-logs/e36-15/validation-p2.md` :
  0 CRITICAL, 0 HIGH, 2 MEDIUM, 5 LOW ; prémisses lues à `c513cc0a1`). Traductions toujours en attente de Guy (D2), non
  tranchées. Toutes les corrections par l'outil Edit. *Annotation de l'entrée P1* : la substitution `head`/`tail`/`cp`
  du bloc D2 était un écart à la consigne Edit, relevé par l'orchestrateur ; elle n'est pas reconduite.
  - V2-1 (MEDIUM, corrigé) : la phrase de D9 pour `api-external.md` ne contient plus le mot « projet » (« reprend le
    libellé interne de Kesh, en français et non revu ») ; aucune exception ajoutée.
  - V2-2 (MEDIUM, corrigé) : recompté à la source par le détecteur à commentaires masqués, en classant les lignes :
    `sites 46 ; appels i18nMsg 38 ; lignes 'fallback' hors i18nMsg 3 ; autres 5` ; 38 + 3 appels à repli sur la ligne
    suivante (`settings/projects/+page.svelte:168`, `export/+page.svelte:67`, `:76`) = **41** appels, + 3 objets + 1 nom
    de fichier + 1 en dur = 46. § 2 et AC-2 alignés (« 44 » retiré ; `\b44\b` grepé : plus aucune occurrence).
  - V2-3 (LOW, corrigé) : prémisse confirmée (`awk … | /usr/bin/grep -ciE '\bprojects?\b'` sur les valeurs en-CH rend
    49) ; « Projected schedule » retiré des exceptions (AC-1 : sans exception) ; chaque extrait est attaché à son seul
    fichier ; « doit encore se rencontrer » = excuser au moins une occurrence du motif dans son fichier ; mutations 3
    et 3-bis récrites.
  - V2-4 (LOW, corrigé) : formes lues `pdf.rs:61`, `:67`, `:603-604`, `:885` ; sondes du littéral continué et du
    doc-comment à guillemets ajoutées ; chaînes brutes, `'"'` et `/* */` écrits en angle mort, absents des deux fichiers
    entiers (trois `grep` vides, relancés ce jour).
  - V2-5 (LOW, corrigé) : AC-4 nomme `reconciliation_rules_e2e.rs:1810` (ligne lue) et la clause « aucun fichier sous
    `crates/kesh-db/` » comme gardes réelles ; `other_errors_keep_their_mapping` ramené à la fonction d'appariement.
  - V2-6 (LOW, corrigé) : AC-11 et AC-6 — `user-manual.pdf` par `grep -oi 'projet' | wc -l` → 0 (40 aujourd'hui) ;
    admin `projet compris` (`:2787`) et `des projets, les règlements` (`:3509`), brochure `projets analytiques y
    seront` : 1 chacun aujourd'hui (relevé ce jour), → 0 ; contrôles vides retirés ; sens courants laissés hors
    contrôle PDF, tenus par G25 sur le `.tex`.
  - V2-7 (LOW, corrigé) : 31 clés neutres (`… | wc -l` rend 31 ; § 1 et Dev Notes), 19 lignes pour `:3767-3804`,
    en-tête « Validation non lancée » remplacé ; `\b32\b` et `\b20\b` grepés (restent les « 32 caractères » et « 150
    caractères » de D4, légitimes).
- **2026-11-15 — validation terminée** en trois passes : P1 (Sonnet) 0 CRITICAL / 0 HIGH / 3 MEDIUM / 7 LOW → P2 (Opus)
  0 / 0 / 2 / 5 → P3 ciblée (Sonnet, lentille unique sur la remédiation `ddd1aa3ec`,
  `kesh-gate-logs/e36-15/validation-p3-ciblee.md`) 0 / 0 / 0 / 0. Close sur une passe sans finding au-dessus de LOW,
  dont la remédiation ne touchait aucune ligne de production. Aucun LOW au dernier rapport : rien à traiter à l'étape 0.
- **2026-11-16 — arbitrage de Guy sur D2 : option A** (transmis par l'orchestrateur dans le prompt de développement).
  de-CH **analytisches Konto**, it-CH **conto analitico**, en **analytical account** (fr-CH « compte analytique »).
  Motif : suit le français (règle du glossaire `docs/i18n-glossaire.md:29`), sans défaire l'arbitrage du 2026-08-19
  contre *Kostenstelle* ; le terme s'écrit toujours en entier (« Konto », « conto », « account » seuls désignent le compte
  du plan). D2 ne garde que les colonnes de l'option A ; les marques `<terme arbitré>` de la note de version et du
  commentaire `:3089` sont remplacées ; bandeau d'en-tête et tâche 0 mis à jour ; registre C-36-15-1 annoté.
  *Écart à la consigne Edit, déclaré* : sept remplacements de phrases de cette étape (bandeau, `:3089`, glossaire,
  tâche 0, note de version) ont été faits par un court script Python sur la fiche (artefact `_bmad-output`, chaque motif
  vérifié unique avant remplacement) ; la réduction des tables de D2, par l'outil Edit. La suite, par l'outil Edit.
- **2026-11-16 — développement** (Opus 5.5). Commits : `e5352ee82` (étape 0), `f5f6ff88b` (gardes d'abord, rouges
  journalisés), `894267d82` (correctif : catalogues, frontend, backend, documentation ; dernier commit de code), puis la
  fiche à `review`. Tout le code, les catalogues et la documentation par l'outil Edit, les catalogues par blocs contigus.
  Décision neuve **C-36-15-7** (type du nom de fichier non coupé à 20 caractères), née du rouge du test
  neuf de l'AC-3 après le correctif. Détail et journaux au Dev Agent Record.
  - Recontrôles de la fiche : trois commandes d'angle mort de l'AC-7 rejouées au commit `894267d82` (vides) ; AC-4 :
    `git diff --name-only $(git merge-base origin/epic/36-0.18.0 HEAD)..HEAD | /usr/bin/grep -E
    "kesh-db|reconciliation.rs|csv.rs|i18n-keys.test|CHANGELOG|migrations"` vide ; AC-10 : contrôle des mots-clés de
    fermeture vide.
  - *Écarts aux consignes, déclarés* : (1) l'en-tête du journal `rouges-e2e.log` a été ajouté par `sed -i` (journal hors
    du dépôt, non du code) ; (2) les mutations et les captures s'appliquent par de courts scripts Python sur des copies de
    travail restaurées par `git restore --worktree` (forme permise pour les mutations) ; (3) le commentaire
    `fr-CH/messages.ftl:3089` reste sur une ligne, comme la fiche l'exige (aucune ligne ajoutée au catalogue).
  - Gates au passage en `review`, tous au commit `894267d82` : `test-fast --ci` **5456/5456** (13 ignorés) ; `gate-ci`
    **5463 passés, 17 ignorés** (relancé après une interruption) ; frontend complet vert (Vitest 3714/3714). **Pas d'E2E
    complet** : il tourne à la clôture, avec `gate-ci`.
- **2026-11-16 — revue de code P1** (trois lentilles, Opus 5.5 ; rapports `kesh-gate-logs/e36-15/review-p1-{A,B,E}.md`) :
  A 0/0/0/3, B 0/0/0/5, E 0/0/0/6 — rien au-dessus de LOW. Règle « tarir la source » : les LOW qui touchent la
  production se corrigent dans la story, puis une passe ciblée.
- **2026-11-16 — remédiation de la revue P1** (Opus 5.5). Commit de code **`62a0b8083`**, puis ce compte rendu. Chaque
  prémisse lue au code avant correction. **Arbitrages de Guy du 2026-11-16**, transmis par l'orchestrateur :
  **B1-4** de-CH « pro **analytisches** Konto » (accusatif) partout — quatre libellés, deux noms de fichier, test
  `slugify_max_keeps_the_analytical_report_types_whole`, doc-comment `util.rs:46`, D2 et glossaire (C-36-15-8) ;
  **B1-5** « Analytisches Standardkonto » et « analytisches Hauptkonto » gardés, inscrits en partie A du glossaire avec
  leur sens (compte analytique par défaut, compte analytique racine — pas un compte du plan) ; partie A **84** lignes
  (C-36-15-9).
  - **B1-1 = E1-1** (confirmé : `i += 2` posait l'indice sur la quote échappée de `'\''`) : `litteraux_de_chaine` saute
    `'`, `\` et l'octet échappé (`i += 3`) ; doc-comment juste ; sondes positives `['\'','"']` puis « projet » et
    `'\\'`. `fin_du_bloc` portait le même défaut (antérieur), trivial : corrigé de même, sonde `['\'','{']` dans
    `sans_modules_de_test_apparie_les_accolades`. Mutation `mut/rem-extracteur.log` (`i += 3` → `i += 2`) : rouge sur la
    sonde (`left: []`, `right: [(1, "projet")]`).
  - **B1-2 = V-A1** (confirmé ; recompte à la source : **39** clés relevées en appel littéral, non 36 comme le disait le
    commentaire) : liste **fermée** des dix clés hors relevé (trois sites en objet, deux noms de fichier, cinq clés
    d'audit assemblées) et `toHaveLength(39)`.
  - **B1-3 = E1-4** (confirmé) : `ExceptionG25 { fichier, extrait, lignes, ligne_entiere, motif }` et la fonction pure
    `g25_non_excuses` : l'extrait (qui contient désormais son « projet » — « le projet dans Container Manager » au lieu
    de « Container Manager ») est retiré de la ligne et le reste relu ; chaque extrait porte le nombre **exact** de lignes
    attendu ; la ligne entière n'est excusée que pour `README.md:218` (ligne historique). Le nouveau régime a fait
    apparaître `admin-manual.tex:2087` (« préfixe du projet », sens Compose, jusque-là excusé par « <projet> » sur la même
    ligne) : seizième extrait ajouté, et « structure-du-projet » (ancre `README.md:19`). Test
    `g25_une_exception_n_excuse_que_son_extrait` (cinq cas). Mutations : `mut/rem-borne.log` (une troisième ligne porte
    « nom du projet » : « 3 ligne(s) la portent, 2 attendue(s) ») et `mut/rem-extrait.log` (« projet analytique » ajouté
    sur une ligne excusée : `admin-manual.tex:608` rouge). Les trois mutations : restauration constatée.
  - **E1-2** (confirmé, latent) : motif anglais `(?i)project(s|'s)?\b`, `\b` à droite seulement ; test
    `g25_le_motif_anglais_voit_les_composes_et_epargne_projected`.
  - **E1-6** (confirmé) : `projects_e2e.rs` affirme le texte de quatre refus, une par famille (champs, hiérarchie,
    archivage, parent archivé), par l'aide `message_du_refus` (`error.message`).
  - **E1-3** (confirmé) : `triggerDownload` (`reports.api.ts:413`) nomme le blob ; le nom vient de
    `reports/+page.svelte` (`kesh-${typeSlug}-${companySlug}.${format}`) avec le type **entier** du catalogue, valeur
    arbitrée : rien à corriger dans le nom. Corrigés : le commentaire `+page.svelte:425-430` (ce nom est le seul employé)
    et la note de version (« quarante », client de l'API seulement). L'écart nom navigateur ≠ `Content-Disposition`
    (préexistant) est donné à l'orchestrateur pour une issue.
  - **E1-5** (confirmé, préexistant, non corrigé) : compte analytique de 150 caractères, rendement exporté en PDF — le
    titre déborde de la page et est coupé au bord (`captures/e15-project-return-page-1.png`, journal
    `e15-titre-long.log`) ; issue donnée à l'orchestrateur.
  - **V-A2, V-A3** (confirmés) : en-tête (C-36-15-1 à 9), § 3 et tâche 5 annotés ; note de version précisée ;
    commentaire de `sprint-status.yaml` récrit dans l'ordre.
  - Gates de la remédiation, au commit de code `62a0b8083` : gate ciblé `cible-rem2.log` 55/55 (le premier,
    `cible-rem1.log`, rouge sur `admin-manual.tex:2087` avant le seizième extrait) ; `clippy-rem1.log` vert ;
    `test-fast-ci-rem1.log` (base remise à neuf) **5458 passés, 13 ignorés** ; `front-test-unit-rem1.log` Vitest
    **3714/3714** (239 fichiers) ; `front-check-rem1.log` 0 erreur (29 avertissements, inchangés). Ni `gate-ci` ni E2E
    (consigne de remédiation). `reports.spec.ts` non rejoué : les noms fr-CH ne changent pas. Prompt de la passe P2 ciblée :
    `36-15-compte-analytique-review-prompt-p2.md`.
- **2026-11-16 — revue P2 ciblée** (Sonnet, une lentille sur `62a0b8083` ; rapport
  `kesh-gate-logs/e36-15/review-p2-ciblee.md`) : 0/0/0/2, deux LOW de code de test, corrigés dans la story (commit
  **`a9778aea3`**, dernier commit de code) :
  - **P2-1** (confirmé : `replace` retirait tous les exemplaires de l'extrait d'une ligne) : `g25_non_excuses` ne retire
    plus qu'un exemplaire (`replacen(…, 1)`) ; sonde « nom du projet, puis nom du projet » → un écart.
  - **P2-2** (confirmé) : mutations de code jouées, journaux `mut/rem-fin-du-bloc.log` (`fin_du_bloc` `i += 3` → `i += 2` :
    `sans_modules_de_test_apparie_les_accolades` rouge), `mut/rem-retrait.log` (retrait de l'extrait supprimé :
    `g25_une_exception_n_excuse_que_son_extrait` et `le_vocabulaire_analytique_est_compte_analytique` rouges),
    `mut/rem-replacen.log` (`replacen` → `replace` : `g25_une_exception_n_excuse_que_son_extrait` rouge) ; fichier
    restauré à chaque fois (script `mut/rem-p2-mutations.sh`). Gate ciblé `cible-rem-p2.log` 30/30, `clippy-rem2.log`
    vert. Boucle de revue close (passe sans finding au-dessus de LOW, remédiation hors production).
- **2026-11-16 — synthèse des boucles.** Validation : P1 Sonnet 0/0/3/7 → P2 Opus 0/0/2/5 → P3 ciblée Sonnet 0/0/0/0.
  Revue de code : P1 — A Sonnet 0/0/0/3, B Opus 0/0/0/5, E Opus 0/0/0/6 (14 LOW, tous corrigés sauf E1-5, préexistant,
  tracé en issue avec E1-3) → P2 ciblée Sonnet 0/0/0/2 (corrigés). Arbitrages de Guy : D2 option A ; B1-4 ; B1-5.
- **2026-11-16 — clôture**, au dernier commit de code `a9778aea3` :
  - `scripts/gate-ci.sh kesh_ci_e3615` (voie 1, sous ses verrous) : **5465 passés, 0 échoué, 17 ignorés**, `rc=0`
    (`kesh-gate-logs/e36-15/gate-ci-cloture.log`).
  - `npm run build` (`front-build-cloture.log`) vert, puis **E2E complet** sur `kesh_e2e_e3615` recréée vide, port 3615,
    montage de `docs/testing.md` (`e2e-cloture.log`, script `e2e-spec-36-15.sh`) : **426 passés, 7 échecs, 18 ignorés**
    (18,8 min). Les sept échecs, jugés un à un contre `docs/testing.md` § « Les échecs attendus » : les sept de la
    **KF-029 (#97)** — `mode-expert.spec.ts:26`, `:41`, `onboarding-path-b.spec.ts:65`, `:92`, `onboarding.spec.ts:57`,
    `:77`, `:150`. Aucun autre échec, aucune pollution ; `reports.spec.ts` vert (onglets et titres « compte analytique »).
  - Backend arrêté par son PID, port 3615 libéré. Bases `kesh_e3615`, `kesh_e2e_e3615`, `kesh_ci_e3615` gardées (consigne
    de l'orchestrateur).
  - Story **`done`**.
