# Story 55-2 : le lien de compte partout — Journaux, Grand livre, TVA, Compte de résultat

Status: ready-for-dev

Epic : `_bmad-output/planning-artifacts/epic-55-navigation-interface.md` — D6, D7 (Q3, acceptée par le mainteneur le
2026-11-25), D8. Issues : #À-VENIR (`enhancement`,
P4, point a) ; #À-VENIR (`bug`, P3, point d : sélecteur d'exercice du Grand livre, AC-11 à AC-15). Release : la prochaine planifiée après la 0.20.0. **Frontend seulement, aucune migration. Suit la 55-1a**
(l'URL du grand livre et le Retour).

## ⚠️ Points renvoyés au développement (validation P6)

La validation de l'ajout (sélecteur d'exercice du Grand livre) s'est arrêtée à la **sixième passe** sur un MEDIUM
(`kesh-gate-logs/s55-kickoff/55-2-validation-p6.md`), **renvoyé au développement** par la règle de la sixième passe : le T0
le tranche avant toute ligne de code, ligne lue et décision citées au Change Log.

- **R-1 (V6-1) — caduc le 2026-11-26** : l'entrée « dates libres » est retirée par D15 (décision de l'utilisateur et avis
  UX, fichier d'epic) ; le menu se déduit des dates et montre « Plusieurs » ou « Hors exercice », options désactivées
  (AC-11). **R-3** (passer l'AC-11 au composant `ui/select`, passe ciblée du 2026-11-26) est **retiré** de même : le menu
  reste natif.
- **R-4 (D15, 2026-11-26)** — Le sélecteur du Grand livre, récrit au-delà du plafond de huit passes (une passe ciblée
  seulement ; ses MEDIUM s'ajoutent ici en R-5 et suivants) : au T0, relire l'AC-11 à l'AC-15 contre la 55-1a (AC-14 à
  AC-17) telle que le développement de la 55-1a l'aura livrée, ligne lue citée au Change Log.
- **R-5 (passe ciblée D15, SE-3, MEDIUM)** — Le sort de `selectedFiscalYearId` avec le bandeau `ReportSelector` (lié en
  `$bindable`, alors que le Grand livre ne doit jamais l'écrire) et le préremplissage de l'AC-12 qui en dépend : à trancher
  au T0 avec R-11 de la 55-1a. LOW : SE-9 (*Modules touchés* à compléter : `ReportSelector`, bandeau de la page).
  Rapport : `kesh-gate-logs/s55-selecteur/selecteur-2026-11-26-validation.md`.
- **LOW à reprendre** : V6-2 (préremplissage à l'**entrée** sur l'onglet, une fois, jamais à la saisie ; test « deux dates
  vidées à la main restent vides »), V6-3 (exception notée aux lignes `:146` et `:175` de la 55-1a ; `reports.spec.ts` cité
  pour la 55-2 au tableau des fichiers communs), V6-4 (second exercice créé pour l'E2E, `api-fixtures.ts:445` ; mutation
  « écriture de `selectedFiscalYearId` » avec un exercice différent et un rapport en mémoire ; tests nommés pour AC-13 et
  AC-14), V6-5 (fichier d'epic : la phrase « Justificatif et non pièce » à remettre au point 3 de l'*Objectif*), V6-6
  (modules de `fiscalYearMatching` et de la fonction de libellé : `features/reports` ; taille de la story déclarée ;
  caduc pour ces deux fonctions, remplacées par `fiscal-year-menu.ts` de la 55-1a, D15).
- **R-2 (P7, V7-1, MEDIUM) — Gardes Rust des textes. Tranché le 2026-11-26** par le mainteneur (resserrement de la
  dérogation Q6, fichier d'epic § *Dérogation*) dans le sens du défaut ci-dessous : § *Gates* et T5 le portent. La story, « purement écran », modifie `messages.ftl` et le manuel,
  gardés par des tests Rust (`parity_between_locales`, `textes_coherents`) qui ne tourneraient qu'en fin d'epic. Défaut :
  la précision de la *Dérogation* proposée au mainteneur (fichier d'epic, question *Gardes Rust des textes*) — à sa
  clôture, `cargo nextest run -p kesh-i18n` et `-p kesh-api -E 'binary(textes_coherents)'` ; l'AC-9 nomme la garde A.
- **LOW de la P7** (`kesh-gate-logs/s55-decisions/55-2-validation-p7.md`, aucune remédiation) : V7-2 (preuve du grep du
  fichier d'epic : remédiée là, Pv2p5 de la 55-3a), V7-3 (**tranché le 2026-11-26** : la ligne surlignée revient à l'epic à part des liens
  de compte, 56 proposé, et n'est pas un livrable de la 55-2 ; C-55-2-4), V7-4 (§ *Gates* et T5 : nommer les specs de la *Dérogation*),
  V7-5 (« Paramètres → Facturation, section Comptes TVA » ; le compte de décompte n'est pas lu, sans lien).

## Story

En tant que comptable,
je veux que chaque numéro de compte d'un rapport ouvre le grand livre de ce compte sur la période du rapport, de la même
façon partout,
afin de passer d'une écriture du journal, d'une contrepartie ou d'une ligne de TVA à l'extrait du compte, et d'en revenir
par *Retour*.

## Modules touchés (règle de splitting préventif)

Quatre : `frontend/src/lib/features/reports` (composant neuf, sept vues), `frontend/src/routes/(app)/reports` (données
passées aux vues), `kesh-i18n` (quatre locales), `docs/manual/fr/user-manual.tex` ; plus `frontend/tests/e2e`. Aucun Rust.

## Ce qui se lit au code

Relevé à `origin/main` `1e23edb55` et `pr/46-rapports-groupes` `511e61d54` ; lignes de la PR 46 relevées au T0.

| Vue | Numéro de compte affiché | Lien aujourd'hui | Période du lien |
|---|---|---|---|
| `BalanceSheetView.svelte` (actifs `:140`, passifs `:179`, fonds propres `:234`) | oui | `<a href={ledgerHref(…)}>` recopié **trois fois** (`:143-149`, `:182-188`, `:237-243`) | `dto.period.startDate`/`endDate` |
| `TrialBalanceView.svelte:67-72` | oui | même `<a>` recopié | idem |
| `GroupedReportRows.svelte` (PR 46, `:150-154`) | oui | même `<a>`, lignes `account` et `own` ; ligne `group` : bouton d'accordéon, pas de lien (à garder) | `period` (prop) |
| `IncomeStatementView.svelte:60`, `:90` (à plat) | oui | **aucun** (texte) — ni à `main` ni sur la PR 46 | — |
| `JournalReportView.svelte:68` | oui, `l.accountNumber`, avec `l.accountId` | **aucun** | `dto.period.startDate`/`endDate` |
| `GeneralLedgerView.svelte:220` (titre de section) | oui, `s.accountNumber`, `s.accountId` | **aucun** | `dto.period.from`/`to` (`LedgerPeriod`) |
| `GeneralLedgerView.svelte:289` (contrepartie) | oui, **numéros seuls** (`counterpart: string[]`, `reports.types.ts:342`) | **aucun** | idem |
| `VatReportView.svelte` | **aucun** (lignes par taux, totaux) | — | `dto.period` |

- Le titre `i18nMsg('reports-ledger-open-from-balance', 'Voir le détail dans le grand livre')` est repris tel quel (une
  clé, quatre locales déjà traduites).
- La page charge déjà la liste des comptes, archivés compris (`fetchAccounts(true)`, `+page.svelte:129-133`), pour le grand
  livre et le lettrage ; le numéro est unique par société (`uq_accounts_company_number`,
  `migrations/20260411000001_accounts.sql:19`) : il désigne le compte sans ambiguïté.
- Les comptes de TVA du décompte (`crates/kesh-report/src/vat_report.rs:177-189`) : `default_vat_recoverable_account_id`
  donne `total_vat_recoverable` (solde du compte au grand livre sur la période) ; `default_vat_payable_account_id` ne sert
  qu'à l'**alerte d'écart** (périmètre ventes). **La TVA due du décompte vient des factures validées** (`invoice_lines`,
  `vat_report.rs:3-9`), non du grand livre ; le manuel le dit (`user-manual.tex:4322`) et avertit que décompte et grand
  livre divergent après un avoir (`keshwarning`, `:4337-4346`). Côté écran, `getInvoiceSettings()` (`invoices.api.ts:99`, `GET
  /api/v1/company/invoice-settings`, **tout rôle authentifié**, `kesh-api/src/lib.rs:1309`) rend
  `defaultVatPayableAccountId` et `defaultVatRecoverableAccountId` (`invoices.types.ts:144-145`).
- `ledgerHref` (55-1a) : sortie inchangée, écrite sur la fonction d'URL de la 55-1a.
- Le décompte vide (`isReportEmpty('vat')`, `reports.api.ts:198-206` : aucune vente, TVA récupérable nulle, aucun solde)
  ne rend pas le tableau (`VatReportView.svelte:11`, `:40-44`).
- La page passe aujourd'hui aux vues : `dto` seul, et `letterableAccountIds` au grand livre (`+page.svelte:853-870`).

### Sites qui restent en texte (inventaire clos)

| Site | Raison |
|---|---|
| `GroupedReportRows.svelte` ligne `group` (PR 46, `:142-145`) | le numéro jouxte le bouton d'accordéon ; le solde propre d'un groupe a sa ligne `own`, liée ; la 46-2 (AC-4) l'a voulu ainsi |
| `ProjectExpensesView.svelte:70` | le numéro est **dans** le bouton qui déplie la ligne : un lien dans un bouton n'est pas du HTML valide ; le rapport par compte analytique n'est pas un rapport de comptes |
| `MirrorOfNote.svelte` (Epic 47, `epic/47-comptes-miroirs`, absent de `main` et de la PR 46) | la mention « Solde de 2270 … » est **un** message Fluent dont les sources sont jointes en une chaîne ; la lier demande de refaire le message (Epic 47) : hors périmètre ; issue `enhancement` **ouverte dans tous les cas** (section *Issues à ouvrir*), et signalée au tableau des fichiers communs de l'epic pour le second arrivé |
| `data-testid` formés sur `accountNumber` (`AccountCurrencyBalance.svelte:18`, `TrialBalanceView.svelte:88`, …) | attributs de test, non des affichages |

## Ajout du 2026-11-25 — le Grand livre a son sélecteur d'exercice (défaut d, `bug` P3)

*Remarque de l'utilisateur, transmise le 2026-11-25 ; l'epic étant à six stories, elle entre dans la 55-2 (la story qui
traite le Grand livre comme destination des liens), et la validation de la fiche reprend.*

**Le défaut** : le bandeau du Grand livre n'a pas de sélecteur d'exercice (`+page.svelte:638-714` à `1e23edb55` :
`ledgerFrom`, `ledgerTo`, compte, « comptes sans mouvement », *Générer* `ledger-generate` désactivé tant que « Du » ou « Au »
est vide). En arrivant d'un autre onglet où un exercice était choisi, le sélecteur d'exercice disparaît avec le bandeau des
rapports classiques (`ReportSelector`) et *Générer* reste inutilisable : comportement faux visible. Issue à ouvrir (texte :
`kesh-gate-logs/s55-kickoff/issues-a-ouvrir.md`).

**Le remède** (AC-11 à AC-15, numérotés à la suite des AC de la story) :

11. **AC-11 — Le sélecteur de la 55-1a, valeur déduite des dates** (*récrit le 2026-11-26 par D15*). Le bandeau du Grand
    livre devient le composant **`ReportSelector`** de la 55-1a (AC-14 à AC-17) — libellé « Exercice » par `<label for>`,
    menu natif en gras, « Du », « Au », « Exercice entier », *Générer* et exports —, lié à `ledgerFrom` et `ledgerTo`, avec
    `unSeulExercice={false}` ; le compte et « comptes sans mouvement » lui sont passés par le *snippet* des contrôles propres
    (`data-testid` d'aujourd'hui gardés : `ledger-from`, `ledger-to`, `ledger-account`, `ledger-include-zero`,
    `ledger-generate`, exports). Le menu se **déduit** des dates par `etatDuMenu` de la 55-1a (aucune fonction
    `fiscalYearMatching` propre au Grand livre) : l'exercice qui contient les deux dates, **même pour une période
    partielle** (et alors « Exercice entier ») ; « Plusieurs » pour deux exercices ; « Hors exercice » pour une date hors de
    tout exercice ; début après fin : inchangé. **Différence avec les autres onglets** : le Grand livre franchit la borne,
    donc *Générer* reste **actif** avec « Plusieurs » et « Hors exercice », sans message. Choisir un exercice remplit « Du »
    et « Au » avec ses bornes ; le Grand livre **n'écrit jamais** `selectedFiscalYearId` (P17 ne se déclenche pas). Société
    sans exercice : le menu est vide et désactivé, les dates restent libres et *Générer* utilisable dès qu'elles sont
    remplies. **Retiré** : l'entrée « — (dates libres) » et sa clé `reports-nav-ledger-free-dates`.
12. **AC-12 — Dates vides pré-remplies.** Quand l'onglet *Grand livre* s'affiche **avec ses deux dates vides** — clic ou
    clavier sur l'onglet (`selectTab`), URL incomplète `?tab=general-ledger`, rechargement, Retour —, « Du » et « Au »
    prennent les bornes de l'exercice choisi des rapports classiques (`selectedFiscalYearId`, par défaut le premier) ;
    *Générer* est **visible et utilisable** (aucune requête n'est faite, 55-1a). Dates déjà remplies : elles restent (garde
    de non-régression, mutation : préremplissage inconditionnel → test rouge). Arrivée depuis la balance âgée ou un rapport
    par compte analytique : même règle (l'exercice choisi des rapports classiques). **Amende la 55-1a** : c'est la seule
    exception à « clic sur un onglet : les champs ne changent pas » et à « URL incomplète : aucun champ n'est modifié », et
    elle ne touche que des champs vides.
13. **AC-13 — Arrivée par un lien de compte.** `ledgerHref(…, from, to)` (sortie inchangée) : « Du » et « Au » sont ceux du
    lien ; le sélecteur en dérive (AC-11).
14. **AC-14 — URL.** Le sélecteur **se déduit** des dates de l'URL (55-1a : `from`, `to`) — aucun paramètre neuf, ordre
    canonique et `ledgerHref` inchangés.
15. **AC-15 — Manuel et commentaires.** § *Grand livre*, l'item « Période libre, sans exercice »
    (`user-manual.tex:4198-4205` à `1e23edb55`) : le titre et la première phrase deviennent « l'exercice comme raccourci
    pour la période ; des dates choisies à la main pour franchir la borne, le menu indiquant alors « Plusieurs » ou « Hors
    exercice » », avec renvoi au paragraphe « Choisir la période » de la 55-1a (AC-19) ; la suite de l'item (ruptures, solde
    d'ouverture) reste. Les commentaires rendus faux sont corrigés : `+page.svelte:103-106`, `:639-641` (« PAS de sélecteur
    d'exercice »), `reports.spec.ts:385-386` (« AUCUN exercice à choisir »).

**Tests** (*récrits le 2026-11-26*) : les règles du menu sont testées une fois, par `fiscal-year-menu.test.ts` et
`ReportSelector.test.ts` de la 55-1a ; ici, Vitest de la page (`reports-page-url.test.ts` de la 55-1a — le bandeau est dans
`+page.svelte`) : le bandeau du Grand livre est `ReportSelector` (`getByRole('combobox', { name: 'Exercice' })` dans
`ledger-controls`) ; choix d'un exercice → dates aux bornes ; dates à cheval sur deux exercices → « Plusieurs » sélectionnée
et désactivée, *Générer* **actif** (mutation : `unSeulExercice` vrai → test rouge) ; arrivée avec dates vides (clic, URL
incomplète) → bornes de l'exercice choisi des rapports classiques, *Générer* actif ; dates remplies gardées (garde et
mutation) ; `selectedFiscalYearId` jamais écrit (mutation : écriture → P17 périme la génération et vide les rapports par
compte analytique, test rouge) ; E2E `reports-navigation.spec.ts` : Bilan de E généré, clic sur *Grand livre* → menu sur E,
dates aux bornes, *Générer* actif, génération ; « Du » reculé dans l'exercice précédent → « Plusieurs », génération, en-tête
« (exercices … et …) » ; rouges sans le correctif (aucun menu dans `ledger-controls`). **Captures** : bandeau du Grand livre
sur un exercice entier, sur une partie (« Exercice entier »), et sur « Plusieurs » avec son en-tête.

## Critères d'acceptation

1. **AC-1 — Un composant.** `frontend/src/lib/features/reports/AccountLink.svelte` (neuf), propriétés `{ accountId: number
   | null; number: string; from: string; to: string; linked?: boolean; testId?: string }`, rend le numéro en lien
   `ledgerHref(accountId, from, to)`, avec le titre d'aujourd'hui, la classe d'aujourd'hui (`text-indigo-700
   hover:underline`), le `data-testid` passé par la vue ; `accountId` nul ou `linked={false}` : le numéro en texte. Tout
   numéro de compte des vues des Rapports passe par lui, liés ou non (le titre d'une section unique du grand livre compris). Les
   `<a href={ledgerHref…}>` d'aujourd'hui — **quatre** à `1e23edb55` (Bilan ×3, Balance), **cinq** sur la base PR 46
   (`GroupedReportRows` en plus) — sont remplacés par lui : après
   la story, `grep -rn "ledgerHref(" frontend/src/lib/features/reports/*.svelte` ne rend **que** `AccountLink.svelte`.
   L'accueil et la réévaluation gardent leurs liens (hors Rapports, hors périmètre). **Second contrôle**, qui voit un
   numéro neuf rendu en nœud de texte : `grep -rnE "(^|[>[:space:]])\{[a-z]+\.accountNumber\}"
   frontend/src/lib/features/reports/*.svelte` (les attributs `number={…}` n'y correspondent pas). Sortie à `1e23edb55` :
   `BalanceSheetView.svelte:147,186,241`, `TrialBalanceView.svelte:71`, `IncomeStatementView.svelte:60,90`,
   `JournalReportView.svelte:68`, `GeneralLedgerView.svelte:220`, `ProjectExpensesView.svelte:70` ; **attendue après** :
   `ProjectExpensesView.svelte:70` seulement, et la ligne `group` de `GroupedReportRows.svelte` sur la base PR 46. Et `grep -n
   "counterpart.join" frontend/src/lib/features/reports/GeneralLedgerView.svelte` : vide après. Les sorties au Change Log.
   Les imports de `ledgerHref` devenus inutiles dans les vues sont retirés (`npm run check` et le lint le vérifient).
2. **AC-2 — Compte de résultat à plat.** Produits et charges : le numéro devient `AccountLink`, période du rapport.
3. **AC-3 — Journaux.** Chaque ligne d'écriture : `AccountLink` sur `l.accountNumber` / `l.accountId`, période du journal.
4. **AC-4 — Grand livre, contreparties (D8).** Chaque numéro de `counterpart` devient un lien vers le grand livre de ce
   compte, **même période** (`dto.period.from`/`to`) ; le numéro se résout en compte par la liste des comptes de la page
   (`Map` numéro → identifiant, construite une fois) ; numéro absent de la liste (liste en échec) : texte, sans erreur.
   Les numéros restent séparés par « , » comme aujourd'hui.
5. **AC-5 — Grand livre, titre (D8).** Le numéro du titre d'une section est un lien **seulement** quand le rapport rend
   plusieurs sections (`dto.sections.length > 1`) ; une seule section — compte filtré, ou « tous les comptes » qui n'en
   rend qu'une — reste du texte. Tests des deux cas.
6. **AC-6 — TVA (D7, Q3 acceptée : les comptes désignés des paramètres de la TVA ; sans compte désigné, aucun lien).** Une ligne `data-testid="vat-accounts"`, **après** le contenu du décompte (tableau, ou message
   « aucune écriture » d'un décompte vide — elle s'affiche dans les deux cas) : « Comptes au grand livre : » suivie de deux
   paires étiquette et numéro — `reports-vat-column-vat-due` puis `AccountLink` du compte TVA due, `reports-vat-recoverable`
   puis `AccountLink` du compte d'impôt préalable —, sur la période du décompte. Les comptes viennent de
   `getInvoiceSettings()` (une requête, **au premier affichage du décompte** : clic, URL, Retour ; aucune tant qu'aucun
   décompte n'est affiché) et de la liste des comptes ; un compte
   non désigné, ou des réglages en échec : sa paire est omise ; les deux : la ligne est omise. L'étiquette ne promet aucune
   égalité de montant (C-55-2-2). Aucun changement du DTO de la TVA, du PDF ni du CSV. *Écart à D7 du fichier d'epic,
   amendé en conséquence* : une ligne à part plutôt qu'un lien dans les lignes « TVA due » et « TVA récupérable », parce que
   la TVA due du tableau n'est pas le solde du compte.
7. **AC-7 — Retour.** Depuis chacun des liens ci-dessus, *Retour* ramène au rapport d'origine (55-1a) : un E2E le constate
   pour le Journal, une contrepartie du Grand livre et la TVA.
8. **AC-8 — Rien d'autre ne bouge.** Exports PDF et CSV, ordre et texte des lignes, montants, accessibilité (axe à zéro
   violation sur les états peuplés de `reports-navigation.spec.ts`, ci-dessous ; celui de `reports.spec.ts` ne voit que le
   Bilan vide), libellés d'aujourd'hui : inchangés ; les Vitest existants des vues passent sans perte
   d'assertion (ceux qui lisaient un `<td>` de texte sont adaptés au lien, adaptation nommée au Change Log).
9. **AC-9 — Textes.** Une clé neuve, `reports-nav-vat-accounts` (« Comptes au grand livre : ») — `reports-nav-ledger-free-dates`
   est retirée avec « dates libres » (D15) ; les textes du sélecteur sont ceux de la 55-1a (AC-18) —, dans un bloc
   `reports-nav-*` des quatre locales (`crates/kesh-i18n/locales/{fr,de,it,en}-CH/messages.ftl`) ; les étiquettes
   réutilisent `reports-vat-column-vat-due` et `reports-vat-recoverable` (quatre locales déjà traduites, aucune phrase
   assemblée par concaténation) ; `i18n-keys` (`sitesTotal` recompté par la garde) et `i18n-libelle-en-dur` verts.
10. **AC-10 — Manuel.** `docs/manual/fr/user-manual.tex` : § *Grand livre*, l'item « Accès direct depuis le bilan et la
    balance » (`:4220-4221`, présent dans le PDF) devient « depuis tous les rapports » (Compte de résultat, Journaux,
    contreparties du grand livre, comptes de TVA) ; § *Journal des écritures* (`:4316`) : une phrase ; § *Décompte TVA*
    (`:4320`) : la ligne des comptes ouvre leur grand livre, **le total des mouvements de la période du compte d'impôt
    préalable (débit moins crédit) égale la TVA récupérable — son solde de clôture y ajoute l'ouverture —, la TVA due
    venant des factures** (`:4322`), avec renvoi à l'avertissement `:4337-4346` ; la phrase `:4326` (« lue du solde du
    compte d'impôt préalable ») est corrigée dans le même sens. `.tex` seul.

## Tests à écrire, et la preuve du rouge

- **Vitest** `AccountLink.test.ts` (neuf) : lien et `href` exacts, titre, texte sans `accountId`.
- **Vitest des vues** : `IncomeStatementView.test.ts`, `JournalReportView.test.ts`, `GeneralLedgerView.test.ts` (contrepartie
  résolue, non résolue, titre lié à plusieurs sections et non lié à une), `VatReportView.test.ts` (deux comptes, un seul,
  aucun, compte désigné absent de la liste des comptes, décompte vide), `BalanceSheetView.test.ts`,
  `TrialBalanceView.test.ts` et `GroupedReportRows.test.ts` (PR 46) : **tests neufs** du lien inchangé (`href`, `title`,
  classe exacts — aucun Vitest actuel ne vérifie ces liens). Chacun rougit sans le correctif (journal
  `kesh-gate-logs/s55-2/vitest-rouge-avant.log`), sauf les trois derniers, filets de non-régression déclarés tels.
- **Vitest de la page** (fichier `reports-page-url.test.ts` de la 55-1a) : réglages de TVA lus **une fois**, au premier
  affichage du décompte (y compris par une URL), **aucune** tant que le décompte n'est pas affiché (onglet ouvert sans
  génération) ; échec silencieux ; carte numéro → compte recalculée quand la liste des
  comptes arrive après le rapport.
- **E2E** `frontend/tests/e2e/reports-navigation.spec.ts` (fichier de la 55-1a, bloc `55-2`) : Journal → lien d'une ligne →
  grand livre du compte → *Retour* → Journal ; Grand livre « tous les comptes » → contrepartie → grand livre de la
  contrepartie → *Retour* ; TVA → compte d'impôt préalable → *Retour*. **Données** : le preset `with-company` désigne déjà les
  deux comptes de TVA (`seed_accounting_company`, `crates/kesh-db/src/test_fixtures.rs:179-195` : TVA due `2000`, impôt
  préalable `1000`) — **aucun `PUT`** ; identifiants lus par `GET /api/v1/company/invoice-settings`, numéros par `GET
  /api/v1/accounts` (modèle `invoice-write-off.spec.ts:131-136`), jamais codés en dur. Le preset n'a aucune écriture : une
  écriture à deux comptes est créée par `POST /api/v1/journal-entries` (modèle `reports.spec.ts`, vers `:364`), sur des
  comptes **autres** que `1000` et `2000` pour le Journal et les contreparties, et une seconde sur un compte tiers pour
  que le grand livre « tous les comptes » rende plusieurs sections. Le décompte vide est couvert par le Vitest de
  `VatReportView`, non par l'E2E. Axe à zéro violation sur le Journal et le grand livre **peuplés** (`AxeBuilder`,
  `withTags(['wcag2a','wcag2aa'])`). Le test rougit sans le correctif (aucun `vat-accounts`).
- La garde DRY de l'AC-1 se vérifie par la commande `grep` de l'AC, journal au Change Log.

## Recette visuelle

Captures (thème clair, `kesh-gate-logs/s55-2/`) : Journal avec les liens ; Grand livre, contreparties en liens ; titre de
section lié et non lié ; TVA avec sa ligne de comptes ; Compte de résultat à plat.

## Tâches

- [ ] **T0** — Base : `epic/55-navigation` avec la 55-1a et la 55-1b ; relever les lignes ; choisir les clés i18n (AC-9).
- [ ] **T1** — Vitest rouges des vues neuves.
- [ ] **T2** — `AccountLink.svelte` ; remplacement des sites (cinq sur la base PR 46).
- [ ] **T3** — Compte de résultat, Journaux, Grand livre, TVA. Propriétés neuves : `GeneralLedgerView` reçoit
      `accountIdByNumber: ReadonlyMap<string, number>` (un `$derived` de `accounts`, comme `letterableAccountIds`
      `+page.svelte:137` : la liste arrive de façon asynchrone) ; `VatReportView` reçoit `vatAccounts: { payable:
      AccountRef | null; recoverable: AccountRef | null }` (`AccountRef = { id, number }`), lus par la page **au premier
      affichage du décompte** (clic, URL, Retour), échec silencieux ; un compte désigné absent de la liste des comptes :
      sa paire est omise.
- [ ] **T3bis** — Grand livre : le bandeau passe à `ReportSelector` de la 55-1a (`unSeulExercice={false}`, contrôles
      propres par *snippet*), préremplissage des dates vides (AC-11 à AC-15).
- [ ] **T4** — i18n, manuel, E2E, captures.
- [ ] **T5** — Gates frontend ; E2E de la story et des Rapports ; à la clôture, `nextest -p kesh-i18n` et
      `binary(textes_coherents)` (§ *Gates*, R-2).

## Gates

`cd frontend && npm run check && npm run lint-i18n-ownership && npm run test:unit && npm run build` ; E2E des Rapports et
de la navigation au dernier commit de code ; suite complète et `scripts/gate-ci.sh` en fin d'epic, story **purement écran** (fichier d'epic, § *Gates*,
*Dérogation*, Q6 tranchée par le mainteneur le 2026-11-25). **À la clôture de la story**, comme elle modifie le manuel et `messages.ftl`, aussi `cargo nextest run -p kesh-i18n` et
`cargo nextest run -p kesh-api -E 'binary(textes_coherents)'` (resserrement de la dérogation, décision du mainteneur du
2026-11-26, fichier d'epic § *Dérogation*).

## Note de version

*Rapports* : le numéro de compte ouvre le grand livre de ce compte partout — Compte de résultat, Journaux, contreparties du
Grand livre et, pour la TVA, les comptes de TVA due et d'impôt préalable. Le Grand livre a le même sélecteur d'exercice que les
autres rapports ; des dates choisies à la main franchissent la borne (« Plusieurs », « Hors exercice ») ; ses dates sont
pré-remplies, et *Générer* utilisable, en arrivant d'un autre
onglet. (refs #À-VENIR, #À-VENIR)

## Dev Notes

### Choix et écarts

- **C-55-2-1** — Un composant plutôt qu'une fonction : le titre, la classe et le repli en texte font partie de ce qui doit
  être identique partout (DRY) ; `ledgerHref` reste la fonction sous-jacente.
- **C-55-2-2** — TVA par les réglages (D7). *Écarté* : un champ de comptes dans le DTO de la TVA (backend). Ce que le lien
  montre : le grand livre des comptes désignés **du moment**, ceux que lit le décompte (`vat_report.rs:177-189`). Le total des
  mouvements de la période du compte d'impôt préalable **égale** la TVA récupérable (`recoverable_balance`,
  `vat_report.rs:370-376` : mouvements de la période, sans ouverture), son solde de clôture non ; celui du compte TVA due **ne l'égale pas** en
  général — la TVA due vient des factures, et avoirs, soldes ou écritures manuelles font diverger les deux (manuel
  `:4322`, `:4337-4346`). D'où une ligne de comptes à part, non un lien sur le montant de TVA due.
- **C-55-2-3** — Les numéros de contrepartie se résolvent côté écran (liste déjà chargée) plutôt que d'ajouter les
  identifiants au DTO du grand livre : aucune requête de plus.

- **C-55-2-4** — `AccountLink` est réutilisable hors des Rapports (décision Q9 du mainteneur : les liens de compte du
  reste de l'application font un epic à part, qui dépend de l'Epic 55) : ses propriétés ne lisent rien de la page des
  Rapports, l'appelant passe les bornes de la période. La **ligne surlignée** du grand livre n'est pas posée ici : elle revient à
  l'epic à part des liens de compte (56 proposé), et seulement depuis une écriture, une facture ou une transaction
  (décision du mainteneur du 2026-11-26 ; fichier d'epic, § *Hors périmètre*).

### References

- Fiche 55-1a ; fiche 46-2 AC-4 (`ledgerHref` sur `account` et `own` seulement).

## Dev Agent Record

### Agent Model Used

### Completion Notes List

### Issues à ouvrir

(Points de revue non corrigés : sévérité, site, texte proposé ; source de la story de solde 55-4.)

- (LOW, `enhancement`) La mention « Solde de 2270 … » des comptes miroirs (`MirrorOfNote.svelte`, Epic 47) reste en
  texte quand les numéros des rapports deviennent des liens ; la lier demande de refaire le message Fluent.

### File List

## Change Log

- 2026-11-25 — Fiche écrite au kickoff de l'Epic 55 (`drafted`).
- 2026-11-25 — Validation P1 (Sonnet) : 0/0/4/6 ; remédiée : TVA due venue des factures, C-55-2-2 et manuel (V1-1) ;
  inventaire clos des sites restés en texte et second contrôle de l'AC-1 (V1-2) ; emplacement et décompte vide, D7
  amendée (V1-3) ; fixture de l'E2E de la TVA (V1-4) ; LOW V1-5 à V1-10.
- 2026-11-25 — Validation P2 (Opus) : 0/0/3/5 ; remédiée : le seed désigne déjà les comptes de TVA, aucun `PUT`, données
  de l'E2E décrites (V2-1, V2-5) ; total des mouvements, non solde, égal à la TVA récupérable, manuel `:4326` (V2-2) ;
  propriétés d'`AccountLink` fixées, second contrôle sur les nœuds de texte avec sorties attendues, contreparties (V2-3) ;
  LOW V2-4, V2-6, V2-7, V2-8.
- 2026-11-25 — Validation P3 ciblée (Haiku) : 0/0/1/1 ; remédiée : déclencheur unique de la lecture des réglages de TVA,
  au premier affichage du décompte (V3-1) ; quatre liens à `1e23edb55`, cinq sur la PR 46 (V3-2).
- 2026-11-25 — Validation P4 ciblée (Sonnet) : 0/0/0/4 — **validation terminée**. Tendance : P1 Sonnet 0/0/4/6 → P2 Opus
  0/0/3/5 → P3 ciblée Haiku 0/0/1/1 → P4 ciblée Sonnet 0/0/0/4. Les quatre LOW de la P4 ne sont pas remédiés dans la fiche
  (une remédiation après la dernière passe ne serait relue par personne) : ils se reprennent au **T0 du développement**,
  ligne lue citée au Change Log — V4-1 (déclencheur de la lecture des réglages : « clic sur *Générer* », décompte en
  mémoire montré par un clic d'onglet, retentative après échec), V4-2 (le cas « aucune lecture avant affichage » est une
  garde, rouge sous une mutation `onMount`), V4-3 (dépendance à la 55-1b : non requise, ordre de l'epic), V4-4 (liste des
  specs : celle du § *Gates* du fichier d'epic). Rapports : `kesh-gate-logs/s55-kickoff/55-2-validation-p{1..4}.md`.
- 2026-11-25 — **Ajout** sur remarque de l'utilisateur : le sélecteur d'exercice du Grand livre, avec « dates libres »
  (AC-11 à AC-15, défaut d, `bug` P3, issue à ouvrir) ; l'epic étant à six stories, l'ajout entre ici ; statut repassé à
  `drafted`, validation reprise (P5).
- 2026-11-25 — Validation P5 ciblée sur l'ajout (Opus) : 0/0/4/7 ; remédiée : préremplissage des dates vides à toute
  arrivée, exception à la 55-1a déclarée (V5-1) ; sélecteur dérivé des dates, `selectedFiscalYearId` jamais écrit (V5-2) ;
  deux clés neuves (V5-3) ; en-tête, note de version, tâche T3bis, captures (V5-4) ; LOW V5-5 à V5-11 (tests nommés, item du
  manuel `:4198-4205`, commentaires, numérotation, texte d'issue, fichier d'epic, libellé partagé, société sans exercice).
  Prochaine passe : P6 ; au-delà, retrait ou renvoi.
- 2026-11-25 — Validation P6 ciblée (Sonnet) : 0/0/1/5. **Arrêt de la boucle par renvoi au développement** (R-1).
  Tendance : P1 Sonnet 0/0/4/6 → P2 Opus 0/0/3/5 → P3 ciblée Haiku 0/0/1/1 → P4 ciblée Sonnet 0/0/0/4 (close) → ajout du
  sélecteur d'exercice → P5 ciblée Opus 0/0/4/7 → P6 ciblée Sonnet 0/0/1/5. `ready-for-dev` **sous réserve** du T0 (R-1).
- 2026-11-25 — **Décisions du mainteneur versées** : Q3 acceptée (en-tête, AC-6) ; Q6 (§ *Gates*, story purement écran) ;
  Q9 (C-55-2-4 : `AccountLink` réutilisable par l'epic à part ; la ligne surlignée n'est pas posée par l'Epic 55). Passe P7
  ciblée sur ces changements ; au-delà de la sixième passe, ses points vont au T0 (R-n).
- 2026-11-25 — Validation P7 ciblée (Opus) sur les décisions du mainteneur : 0/0/1/4. Au-delà de la sixième passe :
  **renvoi au développement** (R-2, LOW listés en tête). Tendance : … → P6 ciblée Sonnet 0/0/1/5 → (décisions du
  mainteneur) P7 ciblée Opus 0/0/1/4. La P8 reste en réserve.
- 2026-11-26 — **Décisions du mainteneur du 2026-11-26 versées** : R-2 tranché (resserrement de la dérogation Q6 :
  § *Gates*, T5) ; ligne surlignée à l'epic à part des liens de compte, 56 proposé, depuis une écriture, une facture ou une
  transaction seulement (C-55-2-4, V7-3). Passe P8 ciblée (la dernière permise ; au-delà de la sixième, ses points vont au
  T0).
- 2026-11-26 — Validation P8 ciblée (Sonnet, `kesh-gate-logs/s55-decisions-2/55-2-55-1b-validation-p8.md`, commune aux 55-2 et 55-1b) sur les
  décisions du 2026-11-26 : 0/0/0/2 — P8-1 (R-2 : « l'AC-9 nomme la garde A » est faux ; écrire « § *Gates* et T5 »), P8-2 (R-2 au passé, « Défaut : … proposée » retiré). **Validation terminée**, dernière passe permise ; les LOW se reprennent au **T0**, ligne
  lue citée au Change Log.
- 2026-11-26 — Précision de l'utilisateur sur la 55-1a (AC-14 : le **nom** de l'exercice seul en gras, par le composant
  `components/ui/select`) : si le T0 de la 55-1a retient les trois sélecteurs d'exercice (R-5 de la 55-1a), le sélecteur
  du Grand livre (AC-11) emploie le même composant et le même rendu ; « — dates libres » reste une entrée en graisse
  normale. Renvoi au **T0** (fiche au plafond de huit passes).
- 2026-11-26 — Passe ciblée sur les précisions d'interface (Opus, `kesh-gate-logs/s55-decisions-2/interface-2026-11-26-validation.md`), hors décompte (fiche au plafond) :
  0/0/1/0. Renvoi au **T0** : **R-3 (UI-14, MEDIUM)** — les trois sélecteurs étant le défaut de R-5 de la 55-1a, l'AC-11
  passe au composant `ui/select` dès que la 55-1a l'adopte : la fonction de libellé partagée rend `{ name, dates }` (et non
  une chaîne), la valeur dérivée passe par un `Select.Root` contrôlé, et l'E2E clique le déclencheur puis
  `getByRole('option')` ; à trancher avec R-7 de la 55-1a.
- 2026-11-26 — **Décision de l'utilisateur et avis UX versés, à confirmer par le mainteneur** (D15 du fichier d'epic, AC-14
  à AC-19 de la 55-1a) : AC-11 récrit — le bandeau du Grand livre est `ReportSelector`, menu déduit des dates (exercice
  même partiel, « Exercice entier », « Plusieurs », « Hors exercice »), *Générer* actif hors d'un exercice ; AC-15, tests,
  captures, AC-9, T3bis et note de version alignés. **Retirés** : l'entrée « — (dates libres) » et sa clé
  `reports-nav-ledger-free-dates`, la fonction `fiscalYearMatching` (remplacée par `etatDuMenu` de la 55-1a), R-1 (caduc),
  R-3 (`ui/select`). Passe de validation ciblée unique (Sonnet, prompt
  `55-1a-55-2-epic-validation-prompt-2026-11-26-selecteur.md`), hors décompte ; ses MEDIUM vont au T0 (R-4 et suivants).
- 2026-11-26 — Passe ciblée D15 (Sonnet, `kesh-gate-logs/s55-selecteur/selecteur-2026-11-26-validation.md`), hors décompte : 0/0/1/1 (SE-3, SE-9). Aucune remédiation : R-5 au **T0**.
