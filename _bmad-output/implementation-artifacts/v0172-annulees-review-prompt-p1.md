# Prompt — revue de code P1 de la Story v0172-annulees (#868), trois lentilles, Sonnet

Tu relis le développement de la **Story v0172-annulees** de Kesh (comptabilité suisse, Rust/Axum, SvelteKit, MariaDB
10.11) : **une facture fournisseur annulée repasse en brouillon** (route `POST /api/v1/supplier-invoices/{id}/unvalidate`,
bouton « Repasser en brouillon » de la fiche). Le geste supprime l'écriture d'achat **et** sa contre-passation (la
contre-passation d'abord, toutes deux par `journal_entries::delete_in_tx`), après avoir dissous les paires de lettrage
`reversal` de l'annulation ; il est refusé par deux motifs neufs — `SUPPLIER_INVOICE_PAID_RECONCILED` (payée, règlement
rapproché) et `SUPPLIER_INVOICE_DETACHED_PAYMENT` (annulée alors que payée, paiement non contre-passé) — et par les gardes
de la dévalidation, appliquées aux **deux** écritures ; chaque refus dit quoi faire d'abord et nomme l'objet (lot,
transaction par sa date, écriture de paiement par son numéro et sa date), par cinq champs neufs de `details`. Le
vocabulaire fournisseur devient « repasser en brouillon » dans les quatre locales, les manuels, l'API externe.

**Aucune migration, aucune table, aucune route neuve.** Kesh 0.17.1 tient des livres réels : rien ne doit toucher une
donnée existante hors du geste demandé.

Worktree `/home/gcorbaz/devel/kesh-0172a`, branche `story/v0172-annulees`. Le diff à relire, **aplati** :
`git -C /home/gcorbaz/devel/kesh-0172a diff ce9badd9...HEAD` (`ce9badd9` = fin de la validation de la fiche ; la branche
part de `d7ec2ea2`, le merge de la v0.17.1). Langue du rapport : français. Modèle : **Sonnet**. Passe **complète**
(première passe d'une revue de code). Contexte frais : ne t'appuie sur aucune passe antérieure.

⚠️ **Les propriétés centrales.**
1. **Rien ne s'écrit sur un refus** (facture `cancelled`, version, deux écritures, lettrage, audit inchangés), et
   **l'ordre** est celui de la fiche : statut → version → lot → paiement détaché → rapprochement (achat, puis
   contre-passation) → pré-contrôle d'exercice clos → refus de `delete_in_tx` pour la contre-passation, puis pour l'achat.
2. **Les verrous d'abord, tous, avant toute lecture ordinaire** : la contre-passation se trouve par une lecture
   **verrouillante** sur `reverses_entry_id`, jamais par `reversed_by` (M14, non tuable par un test séquentiel) ;
   `fetch_lines` et la lecture d'audit viennent après.
3. **Seules les paires `reversal` dont toutes les lignes sont dans l'achat ou sa contre-passation se dissolvent** ; tout
   autre lettrage fait refuser (`ENTRY_LETTERED`).
4. **Le paiement détaché ne perd jamais son gel en silence** : tant qu'une trace `supplier_invoice.cancelled` nomme un
   paiement existant non contre-passé, le geste est refusé ; la lecture partage son fragment SQL avec
   `modification_guard` (un seul site lit `'$.settlementJournalEntryId'` dans `crates/kesh-db/src`).
5. **Les messages serveur et écran sont mot pour mot le FTL fr-CH**, arguments substitués, repli serveur construit par
   `format!` (sans catalogue, `t_args` rend le repli tel quel).

Lis d'abord (lentilles E et A ; la lentille B lit le diff avant) : la fiche
`_bmad-output/implementation-artifacts/v0172-annulees.md` — *Ce que le code dit* (§ 1 à 7), les deux *Inventaires
clos*, *Piste d'audit*, **AC-1 à AC-16**, *Tests et mutations*, *Tâches*, *Manuel*, *CHANGELOG*, *Captures*, *Dev
Notes*, le **Dev Agent Record** et le **Change Log** ; puis le registre `v0172-choix-autonomes.md`, section
`v0172-annulees`, **C-v0172a-1 à 26** (fiche) et **C-v0172a-27 et suivants** (développement). Le Dev Agent Record
déclare des rouges, des gates, des mutations, un E2E et des captures : **ne les prends pas pour argent comptant**,
recoupe-les avec les journaux hors dépôt (lecture seule), sous `/home/gcorbaz/devel/kesh-gate-logs/v0172a/` :
`rouges.log`, `rouges-db-complet.log`, `rouges-api.log`, `rouges-front.log`, `cible-*.log`, `mut/SYNTHESE.log` et
`mut/M*.log` (le script `mut/mutations-v0172a.py` est à **lire**, jamais à exécuter), `gate-backend-*.log`,
`build-*.log`, `front-*.log`, `e2e-*.log`, `make-fr-*.log`, `captures.log` ; le script des captures
`captures-v0172a.mjs` est à **lire** seulement. Les captures (thème clair) sont sous
`/home/gcorbaz/devel/kesh-gate-logs/v0172a/captures/` (regarde-les si ton outil lit les images) : `v0172a-01` à
`v0172a-10` (validée, payée, refus payée, annulée, confirmation, brouillon obtenu, refus payée rapprochée, refus paiement
détaché, refus période verrouillée, refus exercice clos). La validation P5 de la fiche, dont les LOW sont appliqués à
l'étape 0 : `/home/gcorbaz/devel/kesh-gate-logs/v0172a-validate-p5.md`.

⛔ Ne lis rien sous `~/kesh/` ni `~/kesh-test/`.

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier, hormis ton rapport. **Aucun fichier temporaire**, ni
  dans `kesh-gate-logs`, ni dans un scratchpad, ni sous `/tmp`, ni ailleurs ; **aucune redirection** (`>`, `>>`,
  `tee`, `2>`, `&>`, `2>&1`) : compare par des commandes enchaînées (pipe). Un fichier créé par erreur se **déclare** au
  rapport, il ne se supprime pas.
- **Aucun `rm`, sous aucune forme** (ni `rm -f`, ni `rm -rf`, ni `unlink`, ni `find -delete`) ; ni **`pkill`**, ni
  **`killall`**.
- **Aucun `git checkout` ni `git switch`, sous aucune forme** (ni `git checkout -- <fichier>`, ni `git restore`), aucun
  `git` qui écrit (`commit`, `stash`, `reset`, `tag`, `push`, `rebase`, `worktree`, `add`, `fetch`, `merge`).
- **Aucun interpréteur, rien qui écrit, compile ou exécute du code** : ni `python`/**`python3`**, ni `perl`, ni `ruby`,
  ni `node`/**`npm`**/`npx`, ni `bash -c`/`sh -c` d'un script, ni `jq` ; ni **`scripts/prepare-release.sh`**, ni
  `scripts/test-fast.sh`, ni `scripts/mem-guard.sh`, ni **aucun script du dépôt** ni de `kesh-gate-logs` (dont
  `mut/mutations-v0172a.py` et `captures-v0172a.mjs`) ; ni **`make`**, ni `latexmk`, ni **`cargo`**, ni **`docker`**, ni
  `mysql`/`mariadb`, ni `sqlx`, ni **`curl`**, ni **`claude`** sous aucune forme, ni **`sed -i`**, ni `playwright`, ni
  `vitest`.
- **Aucune commande `gh`**, pas même en lecture. **Aucun accès réseau.**
- Permis : `git diff`, `git show`, `git log`, `git grep`, `git ls-files`, `/usr/bin/grep` (dans ce shell, `grep` est une
  fonction vers `ugrep` : écris `/usr/bin/grep`), `sed -n`, `awk` (sans redirection ni `system()`), `cat`, `ls`, `find`
  (sans `-exec`, `-delete`), `wc`, `sort`, `uniq`, `cmp`, `pdftotext <f>.pdf - | tr '\n' ' ' | tr -s ' '`, et
  `git show <réf>:<f>.pdf | pdftotext - - | tr '\n' ' ' | tr -s ' '`.
- ⛔ **Sur le PDF aplati, jamais de `grep -oE` ni de `grep -E` à quantificateur `{0,N}` ou `{N,M}`** : `grep -oF … |
  wc -l` ou `grep -cF` sur des **motifs fixes**, sous `timeout 60`. ⚠️ Le PDF rend l'apostrophe en `’` (U+2019) :
  compte les deux graphies.
- **Toute affirmation d'absence ou de présence** s'appuie sur une sortie **`grep -nF`** (ou `sed -n`) copiée dans le
  rapport.
- **Données réelles** : aucune ; les fixtures sont synthétiques.

## Les axes obligatoires, pour chaque lentille

1. **Les manuels (`.tex` et PDF aplati) contre le code.** `docs/manual/fr/user-manual.tex` § *Repasser une facture en
   brouillon* (`\label{sec:repasser-brouillon}`, sa `keshwarning` sur les deux périodes et la TVA, la lecture du droit
   « non avis juridique »), § *Annuler une facture fournisseur* (« Annuler ou repasser en brouillon ? », la phrase qui
   remplace « La 0.17.0 ne supprime aucune facture annulée », la fin du §), la numérotation (« deux trous »), *La
   corriger au centime*, la liste des corrections par pièce ; `docs/manual/fr/admin-manual.tex` (`:2546`, `:2550`, les
   clés `read-write`, § *Conformité OLICo Art. 9*, la liste des traces, la réserve de la fusion d'audit) ;
   `marketing-brochure.tex` et leurs **PDF**. Chaque affirmation est-elle vraie au code (`unvalidate_in_tx`,
   `+page.svelte`) — en particulier « le clic part sans confirmation » pour une payée, « se repasse alors en brouillon »
   après l'annulation du rapprochement, « le paiement et sa contre-passation restent au grand livre », « une facture
   annulée qui a figuré dans un lot … ne se supprime pas » ? Les phrases retirées sont-elles absentes du PDF aplati («
   se dévalide, se corrige et se revalide », « dévalider une facture fournisseur », « se dévalide par », « ne supprime
   aucune facture annulée », « ni ne se dévalide ») ? `README.md:42` et `website/` ?
2. **Les témoins et les mutations.** Pour chaque refus et chaque effet (T1 à T14, T7, T7b, T8, T12, T13) : quel test le
   couvre, peut-il passer **à vide** (un montage qui rendrait le refus par une autre règle avant d'atteindre le site ;
   la topologie des exercices de `scinder` ; `refus_sans_ecrire` qui compare une photo), et la mutation nommée est-elle
   tuée **par le test attendu** (`mut/M*.log`, `mut/SYNTHESE.log`) ? Une mutation plausible non jouée survivrait-elle
   (l'ordre des deux `delete_in_tx` inversé ; `matched_purchase.or(matched_reversal)` en `matched_reversal.or(…)` ; le
   pré-contrôle sur la seule contre-passation ; la dissolution en mode `Manual` ; le `UPDATE` sans garde de statut ;
   `DetachedPayment` testé avant le lot) ?
3. **Les gardes et compteurs.** Les gardes i18n du frontend (`i18n-keys` `sitesTotal` 2796 → 2801,
   `i18n-un-repli-par-cle` 389 → 391, `i18n-libelle-en-dur` 77 → 76 et `conforme` 63 → 62) : chaque variation se
   recompte-t-elle depuis sa source, aux deux bornes (`d7ec2ea2` → `HEAD`) ? La parité des quatre catalogues, la
   typographie des apostrophes par clé (AC-15), le contrôle de vocabulaire (`clés=18 verbe=0`, commande d'AC-15) ?
4. **Grep du symptôme, la valeur, sur tout le worktree** : `git grep -niE "d[ée]valid" -- frontend/src crates/kesh-i18n/locales crates/kesh-api/src/errors.rs docs/manual/fr/*.tex docs/api-external.md docs/i18n-glossaire.md`
   (trié client / fournisseur / nom de route) ; `git grep -niE "freigabe aufheben|convalida|unvalidat" -- crates/kesh-i18n/locales`
   (valeurs fournisseur seulement) ; `git grep -nE "annulée.*IllegalState|cancelled.*IllegalState"` ;
   `git grep -nF "seule une facture validée se dévalide"` ; `git grep -nF "annulez d'abord le lot."` ;
   `git grep -nF "(Lot 4)"` ; `git grep -nF "'\$.settlementJournalEntryId'" -- crates/kesh-db/src` (une ligne attendue).

## Lentille B — Blind (le diff seul, sans la fiche)

1. **Le code de production** : `unvalidate_in_tx` (`crates/kesh-db/src/repositories/supplier_invoices.rs`) ligne à
   ligne — les verrous et leur ordre, la destructuration `<[_; 1]>::try_from`, les `return Err` de chaque rang, le
   `CAST(… AS SIGNED)` (C-v0172a-27), le `let`-chain du pré-contrôle, l'`UPDATE` gardé par le statut lu, les deux
   `delete_in_tx`, l'audit ; `matched_transaction_locked` ; `letterings::reversal_groups_within_entries` (le `NOT
   EXISTS`, la société) ; les constantes `DETACHED_SETTLEMENT_TRACES` / `_ENTRY` et `modification_guard` réécrite (ordre
   des `bind`) ; `kesh-api/src/errors.rs` (`supplier_unvalidation_message`, le mappage et ses `details`) ; le frontend
   (`supplier-invoice-unvalidate.ts`, `+page.svelte` : condition du bloc, `requestUnvalidate`, corps de la boîte, succès
   par statut, `isStaleState`). Chaque doc-comment touché dit-il vrai au code ?
2. **Les tests** : les assertions neuves (valeurs littérales, messages d'échec), les montages SQL (`scinder`,
   `lettrer_a_la_main`, `photo`), les textes attendus du T7 et du T7b contre le FTL ; les tests Vitest réécrits.
3. **Les textes** : CHANGELOG (`## [0.17.2] — Non publié`), LaTeX, `docs/api-external.md`, glossaire ; aucun mot-clé de
   fermeture (`closes`, `fixes`, `resolves`, « validation close ») dans `git log ce9badd9..HEAD --format=%B`.

## Lentille E — Edge cases

1. **Les états** : une annulée dont l'achat ou la contre-passation est introuvable, deux contre-passations liées
   (`Invariant`) ; une trace d'annulation sans `settlementJournalEntryId`, avec un identifiant non numérique ou d'une
   écriture supprimée ; un paiement détaché contre-passé puis la contre-passation supprimée (est-ce possible ?) ; une
   annulée dont la contre-passation est datée d'un exercice « suivant » (Story 29-1b, `reverse_at_next_fiscal_year_start`
   ne concerne pas l'annulation, mais vérifie) ; une annulée en lot `generated` (inatteignable ?) ; une validée issue
   d'une annulée repassée en brouillon (nouvel achat, ancienne trace d'annulation) ; une clé d'API `read-write`.
2. **Les concurrences** : un lettrage concurrent des lignes de la paire, une contre-passation concurrente du paiement
   détaché (lecture sans verrou : le refus reste-t-il conservateur ?), une annulation concurrente (verrou de la facture),
   le rejeu sur interblocage de la route.
3. **L'écran** : la boîte pour une payée (aucune), un double clic sur *Repasser en brouillon* d'une payée
   (`unvalidating`), le rôle Consultation, la relecture après `SUPPLIER_INVOICE_PAID_RECONCILED` et
   `SUPPLIER_INVOICE_DETACHED_PAYMENT`, l'avertissement du lot confirmé (validée seulement sur la fiche ; boîte), les
   trois autres locales (longueurs, sens, apostrophes).

## Lentille A — Acceptance (chaque AC contre le dépôt)

Pour chaque AC (**AC-1 à AC-16**), rends « tenu », « non tenu » ou « non vérifiable en lecture seule », avec la preuve.
En particulier : AC-3 (`git grep -nF "DELETE FROM journal_entries" -- crates/kesh-db/src/repositories/supplier_invoices.rs`
vide) ; AC-5 (P1 à P6, chacune montée pour que les deux refus jouent) ; AC-6 (le contrôle DRY) ; AC-8 (cinq champs
toujours présents, T7 et T7b) ; AC-12 (séquences par statut) ; AC-13 (les trois parcours E2E) ; AC-14 (PDF régénérés,
aplatis) ; AC-15 (5 clés neuves, 18 valeurs modifiées, quatre locales, typographie) ; AC-16 (les neuf lignes de
`docs/api-external.md`, deux codes neufs). L'étape 0 : les quatre LOW de la validation P5 sont-ils appliqués, ligne lue
citée au Dev Agent Record ? Les entrées passées du Change Log et du registre sont-elles restées des relevés datés
(`git diff ce9badd9...HEAD -- _bmad-output | /usr/bin/grep '^-'`) ?

Et : les choix C-v0172a-27 et suivants sont-ils justifiés, l'un contredit-il un AC, la fiche ou le registre ? Chaque
nombre du Dev Agent Record (tests, mutations, gates, E2E, clés, compteurs) se recompte-t-il depuis sa source, avec son
périmètre ? Les **écarts à la fiche** déclarés sont-ils réels, et en manque-t-il ?

## Rapport

Écris ton rapport dans **un seul** fichier : `/home/gcorbaz/devel/kesh-gate-logs/v0172a-review-p1-B.md` (lentille B),
`/home/gcorbaz/devel/kesh-gate-logs/v0172a-review-p1-E.md` (lentille E) ou
`/home/gcorbaz/devel/kesh-gate-logs/v0172a-review-p1-A.md` (lentille A), selon la lentille qui t'est confiée — par
l'outil d'écriture de fichier, **jamais** par une redirection. Contenu :

1. **Les findings**, chacun : identifiant (`B1`, `E1`, `A1`…), sévérité (`CRITICAL`, `HIGH`, `MEDIUM`, `LOW`), fichier
   et ligne, la preuve (sortie de commande copiée, `grep -nF` pour toute présence ou absence), la correction proposée,
   et s'il est **né du développement** de la v0172-annulees ou **antérieur** (un défaut antérieur se **signale**, il ne
   se corrige pas ici).
2. **Les axes réellement exercés, et ceux qui ne l'ont pas été** — sans cette liste, un « 0 finding » ne compte pas.
3. **Le décompte** `CRITICAL / HIGH / MEDIUM / LOW`, recompté depuis ta liste.
4. Les fichiers créés par erreur, et toute commande lancée hors de la liste permise, s'il y en a.
