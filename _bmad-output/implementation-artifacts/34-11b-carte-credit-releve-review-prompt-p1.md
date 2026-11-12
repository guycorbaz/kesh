# Prompt — revue de code P1 de la Story 34-11b (relevé d'une carte de crédit, virement des deux côtés, #972), trois lentilles, première passe complète

Tu relis la **Story 34-11b** de Kesh (comptabilité suisse, Rust/Axum, SvelteKit 5 runes, MariaDB 10.11), release **0.17.6**, Epic 34 :
**le virement entre deux comptes propres se réconcilie des deux côtés** — l'écriture d'un virement, déjà rapprochée d'un côté, devient
candidate au rattachement de l'autre côté (condition 4 de la règle du rattachement, `reconciliation_link.rs`) ; **annuler un côté ne retire
que son lien** (`reconciliation_cancel::cancel_in_tx`, étape 3 bis), même dans un exercice clos ; la vue d'annulation et les candidates
portent `unlinkOnly` ; `reversalJournalEntryId` devient nullable ; l'avertissement « déjà comptabilisée » de la 34-4 conseille de rattacher
le second côté (clés `-transfer`, `-transfer-among`) ; un manuel **pas à pas** pour tenir une carte de crédit (relevé saisi depuis le PDF,
CSV, achats, prélèvement mensuel). **Aucune migration.**

Worktree `/home/gcorbaz/devel/kesh-e34b`, branche `story/34-11b-carte-credit-releve`. Le diff à relire, **aplati** :
`git -C /home/gcorbaz/devel/kesh-e34b diff a8dce4a9 <dernier commit de code>` — `a8dce4a9` est la fusion de la 34-4 close dans la base
(`3c1510d8`, tête de la 34-11a2) ; le dernier commit de code est nommé au Dev Agent Record de la fiche. Langue du rapport : français. Modèle :
**Sonnet ou Opus** (première passe : jamais Haiku). Contexte frais.

⚠️ **Écarts assumés à la fiche, à juger** (registre `e34-choix-autonomes-e34b.md`) :
- **C-e34b-1** — la condition 4 suit la règle de **sens contraire** de la 34-4 (C-e344-11 ; transmission
  `/home/gcorbaz/devel/kesh-gate-logs/e34-transmission-34-11b.md`, *Mise à jour du 2026-11-12*), et non le lien **actuel** de l'autre compte
  à son grand livre (C-e341-22, cas (ii), (iii)). Vérifie que chaque cas que la fiche voulait refuser l'est encore, et qu'aucun cas légitime
  n'est refusé.
- **C-e34b-2** — `m` = les candidates déjà rapprochées ; une écriture rapprochée ailleurs mais non candidate n'est plus signalée.
- **C-e34b-3** — `entry_cancel_blocker` reçoit `&BankTransaction` (non optionnel) ; les `…_unlinking` gardent `Option<i64>` ; `None` strict
  par construction (les écrans de facture ne traversent pas `entry_cancel_blocker`). M7c rejouée sous une autre forme.
- **C-e34b-4 à 8** — `unlinkOnly` aux candidates, toast du détail d'import, dette d'ouverture de l'E2E par une écriture (34-2 absente),
  exemple 2101, retrait seul quel que soit le genre de l'écriture.

⚠️ **Les propriétés centrales.**
1. **Jamais deux écritures pour un virement, jamais un virement compté deux fois, jamais une écriture contre-passée avec un lien vivant.**
2. **Une écriture porte au plus deux transactions**, de **comptes bancaires différents** et de **montants opposés** ; deux comptes qui
   partagent un compte du grand livre ne rapprochent pas deux fois le même mouvement ; un compte délié ou relié ailleurs n'ouvre aucune porte.
3. **Une seule définition de « l'autre côté »** (`other_side_predicate`) pour la liste, le geste, le compte par page et l'annulation.
4. **Un appariement automatique propose, il ne crée jamais** (CLAUDE.md) : rien n'est rattaché ni présélectionné d'office.

Lis d'abord (lentilles E et A ; la lentille B lit le diff avant) : la fiche `_bmad-output/implementation-artifacts/34-11b-carte-credit-releve.md`
en entier (Choix, AC-1 à AC-10, T1 à T7, mutations, Change Log et **Dev Agent Record**) ; les registres `e34-choix-autonomes-e341.md`
(C-e341-22 à 28, 35 à 37), `e34-choix-autonomes-e344.md` (C-e344-11) et `e34-choix-autonomes-e34b.md`. Le dernier rapport de validation qui
couvre la fiche : `/home/gcorbaz/devel/kesh-gate-logs/e34-validate-p4-e341.md`. Ne prends pas le Dev Agent Record pour argent comptant :
recoupe-le avec les journaux hors dépôt (lecture seule) sous `/home/gcorbaz/devel/kesh-gate-logs/e34b/` : `rouges.log`, `rouges-backend.log`,
`rouges-front.log`, `cible-*.log`, `gates-*.txt`, `gate-*.log`, `front-*.log`, `e2e-r*/`, `mut/M*.log`, `mut/resume.log` (les scripts
`scripts/mut-e34b.py`, `scripts/mut-run-e34b.sh`, `rouges-e34b.sh`, `e2e-e34b.sh` sont à **lire**, jamais à exécuter). **Captures** (thème
clair) sous `/home/gcorbaz/devel/kesh-gate-logs/e34b/captures/` : `34-11b-releve-saisi.png`, `34-11b-paiement-signale.png`,
`34-11b-rattachement-prelevement.png`, `34-11b-liste-des-comptes.png`, `34-11b-annulation-retrait-seul.png` — regarde-les si ton outil lit les
images.

⛔ Ne lis rien sous `~/kesh/` ni `~/kesh-test/`, ni le NAS. Aucune donnée réelle.

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier, hormis ton rapport. **Aucun fichier temporaire**, nulle part ; **aucune
  redirection** (`>`, `>>`, `tee`, `2>`, `&>`, `2>&1`). Un fichier créé par erreur se **déclare**.
- **Aucun `rm`**, sous aucune forme ; ni **`pkill`**, ni **`killall`**.
- **Aucun `git checkout`/`switch`/`restore`/`stash`**, aucun `git` qui écrit.
- **Aucun interpréteur, rien qui écrit, compile ou exécute du code** : ni `python3`, `perl`, `node`/`npm`/`npx`, `bash -c`/`sh -c`, `jq` ; ni
  **`scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/gate-ci.sh`, `scripts/recette-montee.sh`, `scripts/mem-guard.sh`, ni aucun
  script du dépôt ou de `kesh-gate-logs` ; ni `make`, `latexmk`, `cargo`, `docker`, `mariadb`, `sqlx`, `curl`, **`claude`**, `sed -i`,
  `playwright`, `vitest`.
- **Aucune commande `gh`**. **Aucun accès réseau.**
- Permis : `git diff`, `git show`, `git log`, `git grep`, `git ls-files`, `/usr/bin/grep`, `sed -n`, `awk` (sans redirection ni `system()`),
  `cat`, `ls`, `find` (sans `-exec`/`-delete`), `wc`, `sort`, `uniq`, `cmp`, `pdftotext <f>.pdf - | tr '\n' ' ' | tr -s ' '`.
- ⚠️ Dans ce shell, `grep` est une fonction vers `ugrep` : emploie `/usr/bin/grep` ou `git grep`.
- **Toute affirmation de présence ou d'absence** s'appuie sur une sortie `grep -nF` (ou `sed -n`) copiée au rapport.

## Les axes obligatoires, pour chaque lentille

1. **La condition 4** (`linkable_side`, `other_side_predicate`, `HAS_LINK`, `entry_sql`, `count_already_booked_for_page`) : ordre des `?` et des
   `bind` dans les trois requêtes (liste, geste `One`, page) ; `NOT EXISTS … AND NOT (…)` avec des colonnes `NOT NULL` ? ; le plafond `<= 1` ;
   cas à dérouler à la main — virement A→B (crédit et débit), comptes partagés, compte délié puis relié, troisième transaction, écriture à trois
   lignes de banque (A, B, C), écriture dont `M` est nul sur un grand livre, transactions de même montant sur deux comptes du même grand livre,
   écriture contre-passée. L'égalité « page = dialogue » (test AC-8 de la 34-4) tient-elle dans tous ces cas ?
2. **L'annulation d'un côté** (`cancel_in_tx`, étape 3 bis ; `entry_cancel_blocker`, étape 0 ; `get_view`) : ordre des verrous et place de la
   première lecture ordinaire ; deux annulations simultanées (le test `s34_11b_ac4_deux_annulations_simultanees` est-il déterministe, prouve-t-il
   ce qu'il dit, et tue-t-il M5 ?) ; une annulation d'un côté **contre** un rattachement concurrent du même côté ou de l'autre ; une
   contre-passation directe concurrente (`journal_entries::reverse_in_tx`) ; `recheck_open_locked_in_tx` en dernière acquisition ; l'exercice
   clos et le lettrage figé (non refusés pour le retrait seul : est-ce juste ?) ; le verrou de période (`PERIOD_LOCKED`) ; l'écriture d'un
   règlement client ou fournisseur qui aurait un « autre côté » ; la trace (`unlinkOnly`, `reversalJournalEntryId: null`) et l'écran d'audit.
3. **`None` strict** : les écrans de facture (`invoice_settlements_write::settlement_cancel_blocker`, `supplier_invoices`) gardent le rang 3
   strict ; la candidate passe `Some(t.id)` au lieu de `None` — équivalent ? Les lecteurs de `matched_entry_id` qui supposaient « une écriture,
   une transaction » (inventaire de la fiche, § Recoupements ; cherche-en d'autres : K2, rapports, exports, clôture du compte, suppression
   d'un import, `journal_entries` garde de modification, `bank_accounts` soldes) restent-ils justes avec deux transactions ?
4. **Les écrans** : `already-booked.ts` (conditions (a), (b′), (c′), `k − m ≤ 0`) ; `CancelReconciliationDialog.svelte` ; `LinkSettlementModal.svelte`
   (`unlinkOnly`, case à cocher) ; `reconciliation-cancel.ts` ; le toast du détail d'import ; les types TS (`reversalJournalEntryId: number | null`
   — qui d'autre le lit ?) ; les quatre locales (clés retirées `-elsewhere`, `-combined` : plus aucun usage ?) ; `sitesTotal`.
5. **Les manuels et la documentation contre le code** : `docs/manual/fr/user-manual.tex` § *Tenir une carte de crédit, pas à pas*
   (`sec:tenir-carte`) — **rejoue son exemple chiffré** à la main (signes, soldes, écart) —, § *Une transaction qui semble déjà comptabilisée*,
   l'encadré « Dépense déjà réglée », § *Écriture manuelle* (« Un virement entre deux de vos comptes »), § *Rattacher …* (conditions d'une
   écriture, exercice clôturé, « Défaire un rattachement », « Ce qui ne se rattache pas »), § *Annuler un rapprochement* ; `docs/api-external.md` ;
   `CHANGELOG.md` § `[0.17.6] — Non publié`. **Le PDF n'est pas régénéré dans la story** (consigne de l'epic) : contrôle le `.tex`, et dis si le
   PDF du dépôt ment désormais. Greppe les valeurs : `rejetez-la`, `already-booked-elsewhere`, `already-booked-combined`, `#823`, `second côté`,
   `#672` dans `docs/manual/`, `crates/kesh-i18n/`, `frontend/src/`, `CHANGELOG.md`, `docs/api-external.md`.
6. **Les tests** : chaque mutation de la fiche (M1 à M11, et `Macc`, `M2b`) a-t-elle été jouée, et le test qui la tue l'est-il **pour la bonne
   raison** (relis `mut/M*.log`) ? Les deux tests verts dès le départ (`s34_11b_ac4c_comptes_relies`, `s34_11b_ac6_releve_csv_d_une_carte`)
   prouvent-ils quelque chose ? L'E2E `credit-card-statement.spec.ts` dépend-il d'un état de la base partagée (montants, comptes) ?

### Lentille B — Blind Hunter (rapport `/home/gcorbaz/devel/kesh-gate-logs/e34b-review-p1-B.md`)

Lis le **diff seul** d'abord, sans la fiche ; cherche ce qui est faux, fragile ou incohérent, puis seulement recoupe avec la fiche.

### Lentille E — Edge Case Hunter (rapport `/home/gcorbaz/devel/kesh-gate-logs/e34b-review-p1-E.md`)

Parcours chaque branche et chaque borne des axes 1 à 3 ; ne rapporte que les cas non traités.

### Lentille A — Acceptance Auditor (rapport `/home/gcorbaz/devel/kesh-gate-logs/e34b-review-p1-A.md`)

AC-1 à AC-10, T1 à T7, mutations, recompte des décomptes du Dev Agent Record **depuis la source** (tests ajoutés, clés i18n, `sitesTotal`), et
les écarts du registre e34b : chacun est-il écrit, justifié et réversible ?

## Rapport

Pour chaque finding : identifiant (`B1-n`, `E1-n`, `A1-n`), sévérité (`CRITICAL`, `HIGH`, `MEDIUM`, `LOW`), fichier:ligne, preuve (sortie
`grep -nF`/`sed -n` copiée), conséquence, correction proposée. Puis **la liste des axes exercés et de ceux qui ne l'ont pas été** — un « 0
finding » sans cette liste ne compte pas comme passe.
