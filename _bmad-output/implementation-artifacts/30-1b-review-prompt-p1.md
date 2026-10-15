# Prompt — revue de code P1 de la Story 30-1b (trois lentilles B, E, A — Sonnet)

Tu relis le développement de la **Story 30-1b — proposer et accepter les factures fournisseur pour un débit** de Kesh
(comptabilité suisse, Rust/Axum + SvelteKit + MariaDB 10.11), dans le worktree `/home/gcorbaz/devel/kesh-30-1`, branche
`story/30-1b`. Modèle : **Sonnet** (l'auteur est Opus). Le diff à relire, **aplati** :
`git -C /home/gcorbaz/devel/kesh-30-1 diff fecc8566 HEAD` (base : `main` après le merge de la 30-1a). Langue du rapport :
français. Contexte frais : ne présume rien du travail de l'agent.

Lis d'abord : la fiche `_bmad-output/implementation-artifacts/30-1-reconciliation-factures-fournisseur.md`, **partie
b** (AC-b1 à AC-b17, tâches T12 à T23, tests T-b*, mutations M-b*, angles morts L1 à L10, et la synthèse de la
validation au Change Log — en particulier la **simplification** de la P4 et le **retrait** de la garde de référence en
P6 : **aucune** garde « dépense déjà réglée par un autre chemin » ne doit exister ; L1 est un angle mort assumé, #671),
son **Dev Agent Record** (ce que l'agent déclare avoir exécuté), le registre `epic-30-choix-autonomes-301.md`
(C-30-1-47 à C-30-1-60, choix du développement), et le `CLAUDE.md` du worktree (§ *Pattern batch*, § *Review Iteration
Rule*, § *Un appariement automatique propose, il ne crée jamais*, § *Migration breaking policy*).
**La 30-2 (documents, création de factures) n'est PAS dans le périmètre.**

## Interdits absolus

- **Lecture seule stricte.** N'écris, ne modifie, ne crée aucun fichier, hormis ton rapport.
- **Aucun `git checkout` ni `git switch`, sous aucune forme** (ni `--detach`, ni sur un fichier), aucun `git` qui écrit
  (`commit`, `stash`, `reset`, `rebase`, `tag`, `push`, `restore`).
- **N'exécute rien qui écrit, compile ou exécute** : ni `scripts/prepare-release.sh`, ni `scripts/test-fast.sh`, ni
  `scripts/mem-guard.sh`, ni `cargo`, ni `npm`/`npx`, ni `make`, ni `latexmk`, ni `python3`, ni `sed -i`, ni `perl`, ni
  `docker`, ni `mariadb`. Aucune commande `gh` qui écrit.
- Permis : `git diff`, `git show`, `git log`, `git grep`, `grep`, `sed -n`, `cat`, `ls`, `wc`, et
  `pdftotext <f>.pdf - | tr '\n' ' ' | tr -s ' '` **vers la sortie standard seulement** (jamais vers un fichier).
- Aucune donnée réelle dans le rapport (`docs/agents-orchestration.md` § 8).

## Lentille B — Blind (le diff seul, sans la fiche)

Lis le diff comme un relecteur qui ne connaît pas l'intention. Cherche : défaut de logique, erreur de borne (fenêtre
`D + 7` / `D − 365`, tolérance `0,05`, plafond 200, tranches de 500), signe (un débit est **négatif** : chaque
comparaison de montant, chaque `abs()`), `match` non exhaustif ou bras faux, valeur nulle mal traitée
(`supplier_invoice_number`, `payment_reference`, `contact_id`), régression de contrat JSON (un champ existant qui change
de sens : `invoiceId`, `invoiceAmount`, `details` des refus de dates), code d'erreur caché au décompte (un code qui n'est
pas écrit `error_code: "…"` au site de construction), message ou doc-comment qui contredit le code, test qui ne prouve
rien (assertion tautologique, décor vide, sonde qui rend vrai trop tôt), DRY (un symétrique client recopié au lieu d'être
appelé ; `pay_in_tx` réécrit au lieu d'être appelé). Rapport : `/home/gcorbaz/devel/kesh-gate-logs/301b-review-p1-B.md`.

## Lentille E — Edge cases (chemins, verrous, naissance de la vue, concurrence)

Parcours chaque branche et chaque ordre de verrous neuf :

1. `crates/kesh-reconciliation/src/matching.rs` : `MatchableDocument`, la note de référence fournisseur (normalisation,
   six caractères alphanumériques, égalité stricte — un caractère non ASCII, une référence faite de blancs, une valeur
   courte égale), la valeur absolue du montant (le chemin **client** inchangé au bit près ?), le tri à score égal.
2. `crates/kesh-db/src/repositories/reconciliation.rs` : les trois requêtes fournisseur (filtres, ordre d'imputation
   **avant** `LIMIT`, comparaison décimale `DECIMAL(19,4)` contre `DECIMAL(18,2)`, jointure `pb.company_id`),
   l'exigibilité SQL contre Rust, la liste masquante (bornes `BETWEEN`, plafond, tri), `find_contacts_by_ids_chunked`.
3. `crates/kesh-db/src/repositories/supplier_invoices.rs` : `lock_for_bank_settlement_in_tx` — ordre facture → compte →
   articles (verrouillant) → garde (d) en lecture ordinaire ; **où naît la vue** dans `accept_one_supplier_invoice`
   (première lecture ordinaire) et ce que (c) et (d) y voient ; le verrou d'intervalle de (c) ; `pay_in_tx_as` (corps
   unique, acteur, `paid_at` lié). Les cycles de l'AC-b10 (C1 à C7) : chaque participant est-il rejoué
   (`grep -n retry` dans les routes concernées) ? un cycle neuf non rejoué ?
4. `crates/kesh-api/src/routes/reconciliation.rs` : `get_proposals` (crédit inchangé ; débit ; montant nul ; non CHF ;
   `notice` exclusive de toute candidate et de la règle), `rule_mask_for` / `load_rule_mask` (les trois cas rejoués,
   et **seulement** eux ; lectures ordinaires), `accept_one_supplier_invoice` (ordre des étapes, chaque refus en
   `failed[]` et jamais en `AppError`, l'`UPDATE` de la transaction avec `version`), `accept_one_rule` (le rejeu placé
   après `rule_matches` et avant toute écriture), `refuse_wrong_direction` (`-0.00`), la traduction commune des refus de
   dates (le côté client garde-t-il son contrat ?).
5. Les tests de course et de verrou (T-b22, T-b34, T-b24, T-b25) : chorégraphie réellement forcée ? sondes **sans**
   `INNODB_TRX` ? un test peut-il passer à vide ? T-b25 prouve-t-il ce que dit son doc-comment ?

Rapport : `/home/gcorbaz/devel/kesh-gate-logs/301b-review-p1-E.md`.

## Lentille A — Acceptance (la fiche contre le code, et le contrat écrit)

1. Pour **chaque** AC-b1 à AC-b17 : satisfait, partiel ou absent — preuve `fichier:ligne`. Pour chaque test T-b1 à
   T-b47 (sauf T-b43 et T-b44, retirés) : où est-il, asserte-t-il ce que la fiche exige ? Pour chaque mutation M-b1 à
   M-b33 (M-b28 retirée) et les deux ajoutées (M-b5r, M-b12e) : le Dev Agent Record la déclare-t-il tuée, par quel test,
   et la mutation décrite est-elle bien celle de la fiche ? L'inventaire des refus de l'AC-b6 et la table des verrous de
   l'AC-b10, recopiés au Dev Agent Record, correspondent-ils au code ?
2. **Ce qui ne doit PAS exister** : aucune garde ne lit les factures fournisseur **payées** (ni requête, ni `notice`, ni
   raison de refus du type `reference_of_paid…`) ; aucun numéro d'issue ni conseil de **confirmer** un lot dans un texte
   d'écran (`grep -rn "671\|confirm" frontend/src/lib/features/reconciliation/*.svelte` et les catalogues).
3. **Écarts déclarés** (C-30-1-47 à C-30-1-60) : chacun est-il justifié, et le code les suit-il ? En particulier
   C-30-1-60 (deux aides E2E non factorisées dans les specs existantes) et C-30-1-56 (captures en clair seulement).
4. **Textes** : chaque clé neuve ou réécrite est-elle présente dans les **quatre** locales
   (`crates/kesh-i18n/locales/*/messages.ftl`), et le repli de l'écran égal, mot pour mot, à la valeur fr-CH ? Le
   décompte « 37 codes » de `failed-proposal-label.ts` et de son test se retrouve-t-il par
   `grep -ohE 'error_code: "[A-Z_]+"' crates/kesh-api/src/routes/reconciliation.rs | sort -u | wc -l` (+ les codes
   posés par `error_code()`) ?
5. **Contrat écrit** : `docs/api-external.md` (§ *Accepter des propositions de rapprochement*, § *Factures fournisseur
   pour un débit*, § lettrage), `CHANGELOG.md` (`## [0.14.0] — Non publié`, *Ajouté*, *Modifié*, *Corrigé*) et le README
   disent-ils exactement ce que fait le code (champs, codes, raisons, `details`, `paidAt`, L1 sous toutes ses formes) ?
   Aucun mot-clé de fermeture d'issue (`closes`, `fixes`, `resolves`) dans les messages de commit
   (`git log fecc8566..HEAD`) ni dans le diff.
6. **Manuel** : `docs/manual/fr/user-manual.tex` **et son PDF**
   (`pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '`, sortie standard) — chaque passage nommé par
   l'AC-b13 est-il écrit, et vrai au code (fenêtre, montant exact, référence, lot du compte, masquage de la règle,
   acceptation d'une règle refusée, annulation du rapprochement, `paidAt`, l'encadré L1 et son geste sûr) ? Un passage
   devenu faux subsiste-t-il (« seules les transactions créditrices », « en réglant une facture client », « depuis la
   fiche de la facture qu'il a réglée ») ? Greppe la **valeur**, pas la formulation.
7. **Registres** : `bank_account_closed_inventory.rs` (`accept_one_supplier_invoice` classé, `pay_in_tx_as`, compteur
   33), `letterings_lexical.rs` (`pay_in_tx_as`), doc-comments « sept chemins » (`grep -rnE "\b(six|sept) chemins"
   crates`), `i18n-keys.test.ts` (`sitesTotal` 2206, et son décompte).

Rapport : `/home/gcorbaz/devel/kesh-gate-logs/301b-review-p1-A.md`.

## Rapport (chaque lentille)

- Findings numérotés `B1…` / `E1…` / `A1…`, chacun : sévérité (CRITICAL / HIGH / MEDIUM / LOW), `fichier:ligne`,
  **preuve copiée** (sortie `grep -nF` ou `sed -n`) pour toute affirmation de présence ou d'absence, correctif proposé.
- **La liste des axes réellement exercés ET de ceux qui ne l'ont pas été.** Un « 0 finding » sans cette liste ne compte
  pas comme passe.
- Rien d'autre n'est écrit nulle part.
