# Prompt — revue de code P1 de la Story v017-3a (rattacher une transaction à une écriture déjà passée), trois lentilles, Sonnet

Tu relis le développement de la **Story v017-3a** de Kesh (comptabilité suisse, Rust/Axum, SvelteKit, MariaDB 10.11) :
**#671** — une transaction bancaire en attente, déjà comptabilisée par un autre chemin (facture fournisseur payée depuis
sa fiche ou par un lot d'une facture, écriture saisie au journal, règlement client par compte interne), se **rattache**
à l'écriture **existante** dont le mouvement sur le compte du grand livre de la banque l'égale exactement : routes
`GET …/entry-candidates` et `POST …/link-entry` (`crates/kesh-api/src/routes/reconciliation.rs`), règle, lecture et
geste au dépôt (`crates/kesh-db/src/repositories/reconciliation_link.rs`), scission de la fonction de blocage de
l'annulation (`reconciliation_cancel.rs`, `entry_cancel_blocker`), erreur `RECONCILIATION_ENTRY_NOT_LINKABLE`, trace
`reconciliation.entry_linked`, registres de garde, section « écritures » du dialogue
(`frontend/src/lib/features/reconciliation/LinkSettlementModal.svelte`), bouton renommé, i18n (quatre catalogues),
spec Playwright, manuels FR (utilisateur **et** administration, `.tex` **et** PDF), `docs/api-external.md`,
`CHANGELOG.md` (en-tête provisoire `[0.17.0] — Non publié`), `README.md`. **Aucune migration, aucune version bumpée.**

Worktree `/home/gcorbaz/devel/kesh-0174`, branche `story/v017-3a-rattacher-ecriture`. Le diff à relire, **aplati** :
`git -C /home/gcorbaz/devel/kesh-0174 diff 899f564a...HEAD` (`899f564a` = tête de la fiche validée ; la base du
code est `c5101b07`, identique pour `crates/`). Langue du rapport : français. Modèle : **Sonnet**. Passe **complète**
(première passe d'une revue de code). Contexte frais : ne t'appuie sur aucune passe antérieure.

⚠️ **Les quatre propriétés centrales.**
1. **Une règle, une définition.** Les conditions 2 à 6 sont **une** requête (`entry_candidates_sql`), lue entière par
   la liste et restreinte à `e.id` par le geste ; la condition 1 est une fonction pure. Aucun second test de montant,
   de compte, de lien ou de contre-passation, ni en Rust ni en SQL, ni à l'écran (`canLink` en est le miroir).
2. **Les verrous et les courses (AC-3, T-race-1 à 3).** Ordre `t → je`, deux verrous de ligne par clé primaire, la
   première lecture ordinaire **après** le dernier verrou ; aucune lecture verrouillante sur `matched_entry_id`. Le point
   (b) de l'AC-3 : **aucun geste ne tient une ligne `journal_entries` puis ne verrouille une ligne de
   `bank_transactions` sans tenir le verrou nommé du compte de cette ligne** — à **refaire par fonction** (pas un grep
   mono-ligne), y compris la suppression d'une écriture (`ON DELETE SET NULL`) et celle d'un import ; et la
   sérialisation avec le rattachement à un règlement (`link_in_tx`), qui peut viser la même écriture quand deux comptes
   bancaires partagent un compte du grand livre (AC-2.12).
3. **Un appariement automatique propose, il ne crée jamais.** Rien de présélectionné, ni dans une section ni dans
   l'autre ; un seul choix ; aucune écriture passée par le rattachement ; aucune donnée existante réécrite à la lecture
   (AC-21, AC-22).
4. **La vérité datée (D12).** `cancelBlockedBy` vient de **la** fonction de blocage de l'annulation (aucune seconde
   précédence) ; le dialogue ne promet jamais une annulation inconditionnelle ; la case de l'obstacle conditionne
   *Rattacher*.

Lis d'abord (lentilles E et A ; la lentille B lit le diff avant) : la fiche
`_bmad-output/implementation-artifacts/v017-3a-rattacher-ecriture-existante.md` — les **AC-1 à AC-22**, *La règle*,
*Tests et mutations prévus* (M1 à M30), *Ce que les gardes ne voient pas* (L1 à L10), *À recontrôler à l'étape 0*, le
**Dev Agent Record** et le **Change Log** ; puis le registre `v017-3-choix-autonomes.md`, **C-v017-3-2 à 12, 17, 21,
25** (fiche) et **C-v017-3-40 et suivants** (choix du développement ; les numéros 28 à 39 sont ceux de la v017-3b,
hors de ton périmètre). Le Dev Agent Record déclare des rouges, des gates, des mutations, une mesure et un E2E : **ne
les prends pas pour argent comptant**, recoupe-les avec les journaux hors dépôt (lecture seule) :
`/home/gcorbaz/devel/kesh-gate-logs/v0173a-*.log`, `v0173a-mutations/*.log`, `v0173a-mutations/bilan.txt`,
`v0173a-mutations/muter.py` et `campagne.sh` (à **lire**, jamais à exécuter), `v0173a/` (captures, journaux E2E).

⛔ Ne lis rien sous `~/kesh-test/` (données réelles).

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
  `muter.py` et `campagne.sh`) ; ni **`make`**, ni `latexmk`, ni **`cargo`**, ni **`docker`**, ni
  `mysql`/`mariadb`, ni `sqlx`, ni **`curl`**, ni **`claude`** sous aucune forme, ni **`sed -i`**, ni `playwright`.
- **Aucune commande `gh`**, pas même en lecture : les issues citées (#671, #672, #823, #827, #828, #401, #687) se jugent
  sur ce que la fiche et le registre en disent ; un doute s'écrit « non vérifiable sans `gh` ».
- Permis : `git diff`, `git show`, `git log`, `git grep`, `git ls-files`, `grep`, `sed -n`, `awk` (sans redirection ni
  `system()`), `cat`, `ls`, `find` (sans `-exec`, `-delete`), `wc`, `sort`, `uniq`, `cmp`,
  `pdftotext <f>.pdf - | tr '\n' ' ' | tr -s ' '`, et `git show <réf>:<f>.pdf | pdftotext - - | tr '\n' ' ' | tr -s ' '`.
- ⛔ **Sur le PDF aplati, jamais de `grep -oE` ni de `grep -E` à quantificateur `{0,N}` ou `{N,M}`** : `grep -cF` ou
  `grep -oF … | wc -l` sur des **motifs fixes**, sous `timeout 60`. Exemple :
  `timeout 60 pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' | grep -oF 'rattach' | wc -l`.
  Les apostrophes du PDF sont typographiques (`’`) : cherche les deux formes.

## Les trois lentilles

Chaque lentille écrit **un** rapport, et lui seul :

- **B — Blind Hunter** → `/home/gcorbaz/devel/kesh-gate-logs/v017-3a-review-p1-B.md`. Lis **le diff seul** d'abord,
  sans la fiche : défauts de logique, SQL (liaisons et ordre des `?`, `DECIMAL`, `-0.00`, signe, `LIMIT` avant tri),
  erreurs de concurrence, fuites entre sociétés, gestion d'erreur, textes (fallbacks Rust et Svelte égaux mot pour mot
  au catalogue fr-CH, quatre locales cohérentes), tests muets (un test qui passerait sans la condition qu'il prétend
  garder). Puis seulement la fiche, pour trier.
- **E — Edge Case Hunter** → `…-review-p1-E.md`. Énumère les chemins et les bornes : transaction nulle, `-0.00`, EUR,
  rejetée, rapprochée entre la liste et le clic ; écriture contre-passée, contre-passation, supprimée, d'une autre
  société, d'un exercice clos, à compte archivé, sans ligne sur le compte, sur un ancien compte lié ; règlement client
  par virement sur ce compte / sur un autre compte partageant le grand livre / par compte interne ; règlement
  fournisseur payé, détaché, en lot d'une ou plusieurs factures ; 50 et 51 candidates ; compte bancaire clos, archivé,
  non lié, clos pendant l'attente du verrou ; courses avec `…/link`, `…/cancel`, `reverse`, `PUT` et `DELETE` d'une
  écriture, suppression d'un import, clôture du compte ; le dialogue (débit, crédit, quatre cas de listes vides, case
  de l'obstacle, relecture sur course, garde de session). Pour chaque chemin : ce que fait le code (ligne lue), et le
  test qui l'exerce — ou son absence.
- **A — Acceptance Auditor** → `…-review-p1-A.md`. Chaque AC (1 à 22, retirées comprises pour vérifier qu'elles le
  restent) et chaque mutation M1 à M30 : satisfait / non satisfait / partiel, preuve citée (fichier:ligne). Recoupe le
  Dev Agent Record avec les journaux. Vérifie les **compteurs de garde** recalés (`audit_route_registry.rs`,
  `bank_account_closed_inventory.rs`, `i18n-keys.test.ts`, le manuel d'administration « N des M routes ») en
  **recomptant la source**, jamais en relisant la valeur.

## Axes obligatoires (toutes lentilles, chacune sous son angle)

1. **La règle** : `entry_candidates_sql` contre *La règle* de la fiche, condition par condition ; ordre des liaisons
   de `query_entry_candidates` ; la jointure redondante dite telle ; `bank_transactions` hors du `FROM` de la somme.
2. **Verrous et courses (AC-3, (a) et (b))** : refaire l'inventaire **par fonction** des sites qui verrouillent ou
   écrivent `bank_transactions` (`UPDATE`, `DELETE`, `FOR UPDATE`, `LOCK IN SHARE MODE`, clé étrangère `ON DELETE`) et
   des gestes qui tiennent `journal_entries` avant ; T-race-1 à 3 ; ce que chacune prouve et ne prouve pas. Lire le
   doc-comment de `link_entry_in_tx` contre le code.
3. **`cancelBlockedBy`** : une seule fonction (`entry_cancel_blocker`, appelée par `cancel_blocker` et la route), une
   seule précédence, codes de fil (`SettlementCancelBlocker::code()`), `cancelBlockedLabel` ; la lecture dans un seul
   instantané (C-v017-3-43).
4. **Le dialogue** : un seul choix, rien de présélectionné, `link` ou `link-entry` selon la section, la case de
   l'obstacle, `RELIRE_SUR`, les quatre cas de listes vides, la garde de session inchangée pour la 29-8, les textes.
5. **Les registres et gardes** (`audit_route_registry.rs`, `bank_account_closed_inventory.rs`, `audit_labels.rs`,
   `textes_coherents.rs` G9, `i18n-keys.test.ts`) : la route classée, les compteurs **recomptés**, la prose qui compte.
6. **Le manuel utilisateur et son PDF** (`docs/manual/fr/user-manual.tex` et `.pdf`), **et** le manuel
   d'administration (`admin-manual.tex` et `.pdf` : le compte des routes, le paragraphe *Intégrations par l'API* de
   § *Passer à la 0.16.1*) : chaque passage de la liste close de l'AC-18 (points 1 à 16) contre le code ; le filet
   `rattach` au `.tex` (lignes) et au **PDF aplati** (occurrences, `grep -oF … | wc -l`), comparé au Dev Agent Record ;
   aucun `#671` restant ; chaque reste renvoyé à **son** issue ; le PDF régénéré **du même commit** que son `.tex`
   (`git log` des deux fichiers). Ce que le manuel promet, le code le fait-il (angle mort L3 des comptes partagés,
   annulation datée) ?
7. **`docs/api-external.md`, `CHANGELOG.md`, `README.md`** contre le code (ordre des refus, champs, codes).
8. **Données existantes** (AC-21, AC-22) : aucune migration, aucune instruction d'écriture nouvelle hors du geste.
9. **Tests muets et mutations** : pour chaque mutation déclarée tuée, le test cité rougit-il **pour la bonne raison**
   (lire son journal) ? Une mutation équivalente ou survivante est-elle déclarée ?

## Le rapport

En tête : **décompte** `CRITICAL / HIGH / MEDIUM / LOW`, **recompté** depuis la liste. Puis chaque finding : sévérité,
lieu (fichier:ligne), constat, **preuve** (sortie `grep -nF` ou ligne lue, copiée), remède proposé. Puis **deux listes
obligatoires** : les **axes exercés** (avec ce qui a été lu) et les **axes non exercés** (avec la raison). Un « 0
finding » sans ces listes ne compte pas comme passe. Toute affirmation de présence ou d'absence s'appuie sur une sortie
`grep -nF` copiée.
