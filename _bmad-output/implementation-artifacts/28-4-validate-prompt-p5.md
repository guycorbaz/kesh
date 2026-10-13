# Prompt — validation P5 de la spec, Story 28-4 (clore un compte bancaire, en lecture seule)

*Versionné le 2026-10-13. Passe **complète**, deux lentilles, contexte frais chacune, en lecture seule. Rotation (D6,
rétro 25) : passes complètes Sonnet ↔ Opus ; Haiku réservé à une passe ciblée de fin de boucle. P1 : Sonnet ; P2 :
Opus ; P3 : Sonnet ; P4 : Opus ; **P5 : Sonnet** pour les deux lentilles. La remédiation de la P4 a été écrite par
Opus ; elle ne change pas la conception (garde non verrouillante, C-28-4-21) mais **déplace une garde** (W24, après le
comptage, C-28-4-24), **complète la règle de placement** (clause « avant sa première écriture »), ajoute un test
d'entrelacement (T-a49) et trois mutations (M54 reclassée, M55, M56), et trace L1 par l'issue **#651** : ce n'est pas
une passe ciblée, c'est toute la fiche que tu relis, en commençant par ce qui vient d'être écrit.*

Dépôt `/home/gcorbaz/devel/kesh-28-4`, branche `story/28-4`. **Base au sol : `5aad8b35`** (kickoff de l'Epic 28) — le
code de la branche est celui de cette base : `git diff --stat 5aad8b35 HEAD -- crates frontend` doit être **vide**.
⚠️ `origin/main` a avancé depuis : la **28-3** (`c1667e8a`, #633) y est mergée et touche `kesh-db/src/backup.rs`,
`kesh-db/src/repositories/bank_imports.rs`, `routes/admin.rs`, `admin_backup/export.rs` ; la fiche garde les numéros
de `5aad8b35` et renvoie la relecture au rebase (T0). Lis le code au sol par `git show 5aad8b35:<chemin>` ou dans le
worktree (identique) ; lis la 28-3 par `git show c1667e8a -- <chemin>`.

Fiche : `_bmad-output/implementation-artifacts/28-4-cloture-compte-bancaire.md` (une fiche, **deux** stories : 28-4a
serveur, AC1-AC10 et AC6-bis ; 28-4b écran, AC11-AC17). Epic : `_bmad-output/planning-artifacts/epic-28-recette-v0-13-2.md`
§ 28-4. Issues #642 et **#651** (`gh api repos/guycorbaz/kesh/issues/642`, `…/651`, et leurs `/comments`). Registre :
`_bmad-output/implementation-artifacts/epic-28-choix-autonomes.md`, section *Story 28-4* (C-28-4-1 à C-28-4-24 ;
**C-28-4-24 neuf** — W24 après le comptage ; **C-28-4-21 annoté en P4** — clause finale de la règle, W24, M54, #651 ;
C-28-4-14, -15 et l'annotation P3 de -21 inchangés). Story voisine non mergée (W30, R8) : `story/28-5`, fiche et code
(`git show story/28-5:…` si la branche est visible ; sinon, le dire).

**La remédiation à relire** : `git show 968a9ee3` (fiche + registre), soit `git diff f923f153 968a9ee3`. Les rapports
de la P4 (hors dépôt, à lire pour savoir ce qui a été demandé) : `/home/gcorbaz/devel/kesh-gate-logs/28-4-validate-p4-R.md`
et `…-p4-F.md`. L'entrée P4 du Change Log de la fiche dit, finding par finding, ce qui a été corrigé, et nomme les
**écarts** de la remédiation à la consigne (refus du sol de W24 en `409`, non `412` ; M54 rendue tuable plutôt que
déclarée assumée ; décalage de la 28-3).

**La décision qui gouverne cette passe** (C-28-4-21 et ses annotations P3 et P4, AC7) : la garde « compte clos » ne
prend **aucun** verrou ; elle se pose *après le dernier verrou que le chemin prend avant sa première lecture ordinaire
— ou, si le chemin n'en a aucune hors l'audit, après le dernier verrou qu'il prend avant sa première écriture, et avant
cette écriture* ; L1 est un angle mort assumé, **P2**, tracé par #651. Le critère décidable de l'AC7 a **deux volets** :
(1) *aucun verrou de ligne ajouté, explicite ou implicite par clé étrangère, à aucun chemin du sol, et aucun ordre de
verrous changé* ; (2) *aucun déplacement de la naissance de la vue* — les sous-`SELECT` de l'audit
(`audit_log.rs:94-95`) exclus, par écrit, depuis la P4. Ta passe ne rediscute pas la conception — elle vérifie que les
deux volets sont **tenus partout** et que la fiche **dit vrai** ; si tu trouves que la conception laisse #642 non
fermée **au sens où la fiche le borne** (hors fenêtre de #651, sauf `cancel_batch`), c'est un finding, avec sa preuve.

Règles : `CLAUDE.md` (§ Review Iteration Rule et ses sous-sections — le manuel nommé, les axes déclarés, les sites NON
RÉSOLUS, la propagation post-patch, le recompte des comptes rendus, *recompter n'est pas réécrire l'historique* ;
§ Migration breaking policy P1-P8 ; § Pattern batch ; § Priorités des défauts ; § Règle de splitting préventif et son
amendement D5 ; § Issue Tracking Rule, dont le mot-clé de fermeture porté par la PR) ; `docs/agents-orchestration.md` ;
la mémoire *invariant dans le temps* (L1 dit que I1 et I2 peuvent être défaits : tout raisonnement de la fiche qui les
suppose inviolables est à relire — y compris le « par I1, tout import non vide d'un compte clos est rapproché » de
C-28-4-24).

## Axe obligatoire — le DOUBLE critère, chemin par chemin, sur le code du sol (les deux lentilles)

Pour **chaque** ligne de la table *Position par chemin* (Dev Notes § *Concurrence*) **et** pour tout chemin d'écriture
de l'inventaire (W1-W30, R1-R8) qui n'y figurerait pas :

1. écris, **depuis le code au sol** (`sed -n` des lignes), la suite de ses verrous dans l'ordre — y compris les
   **implicites** (verrou partagé sur le parent posé par une clé étrangère à l'`INSERT` d'un enfant ou à l'`UPDATE`
   d'une colonne à clé étrangère ; verrous de trou ; verrous des fonctions appelées : `journal_entries::create_in_tx`,
   `reverse_owned_in_tx`, `reverse_in_tx`, `letterings::dissolve_*`, `refuse_if_claim_account`,
   `company_invoice_settings::*` — dont `rounding_account_for_write` —, `flip_primary_off_for_company`,
   `fiscal_years::find_open_covering_date`, l'audit) ;
2. écris, **depuis le code au sol**, la **première lecture non verrouillante** de la transaction — hors les sous-`SELECT`
   de l'audit —, en remontant **jusqu'à la route** et à toute fonction qui compose (W25 depuis W9, et tout autre
   appelant que `grep -rn "<fonction>(" crates/*/src` révèle) ; un `SELECT` sans table (`GET_LOCK` de
   `with_account_lock`) n'ouvre pas de vue InnoDB : vérifie-le, ne le suppose pas ;
3. écris ce que la fiche fait ajouter au chemin, **et où** (instruction élargie, lecture non verrouillante neuve — à
   quelle ligne exacte, entre quels verrous —, lecture existante élargie, rien) ;
4. réponds, avec preuve : **un verrou est-il ajouté ?** **l'ordre change-t-il ?** **la vue naît-elle plus tôt qu'au
   sol ?** ;
5. compare ta table à celle de la fiche, ligne à ligne, et signale tout écart (ligne fausse, numéro faux, chemin
   manquant, verrou implicite omis, lecture ordinaire du sol omise).

Rends le résultat sous forme de table (chemin / suite des verrous au sol / 1re lecture non verrouillante au sol / ajout
de la story et sa position / verrou ajouté ? / ordre changé ? / vue déplacée ? / écart avec la fiche). **Un verrou
ajouté, un ordre changé, ou une vue avancée devant un verrou que la fiche ne déclare pas est au moins HIGH** — une vue
avancée devant un verrou d'exercice est une erreur comptable possible (P1).

Points à examiner en particulier :

- **W24, position neuve** (après le comptage de l'étape 3, avant le `DELETE`, C-28-4-24) : la clause finale de la règle
  de placement est-elle tenue (rien entre l'étape 2 et la garde que le comptage en mémoire) ? L'ordre des refus écrit
  à l'AC6 (`404` → `409 BANK_IMPORT_HAS_RECONCILED_TRANSACTIONS` → `409 BANK_ACCOUNT_CLOSED`) est-il celui que le code
  prescrit produira, **route comprise** (`routes/bank_imports.rs:1455-1480` : comment la route traduit chaque issue) ?
  Le message du refus du sol (« annulez d'abord leur rapprochement ») mène-t-il, sur un compte clos, à un geste
  possible — et la fiche le dit-elle ?
- **W9, W25, W28, W29** : positions inchangées en P4 ; toujours après **tous** les verrous qui, au sol, précèdent la
  première lecture ordinaire ?
- **W8** : le verrou conditionnel d'arrondi (l. 225) est-il juste et complet dans la ligne refaite ?
- **W22** (« en tête de la fonction ») et **W30** (« aucune hors l'audit », code de `story/28-5`) ;
- **les deux chemins neufs** : la vue de la clôture naît-elle bien après son verrou (condition de T-a31) ?

## Axe premier — les régressions de la remédiation P4 (`968a9ee3`)

*La sévérité se déplace vers ce qu'on vient d'écrire.* Relis **chaque** changement de `968a9ee3` :

- **La clause finale de la règle de placement** (AC6, AC7, T4, annotation P4 de C-28-4-21) et **l'exclusion des
  sous-`SELECT` de l'audit** du second volet (AC7, table) : formulées de façon **identique** aux trois lieux et
  **décidable** ? L'exclusion ouvre-t-elle un cas qu'elle ne dit pas (un chemin dont une décision dépendrait d'une
  lecture faite dans l'audit, ou un audit qui ne serait pas la dernière instruction) ?
- **C-28-4-24** : la raison (« par I1, tout import non vide d'un compte clos est rapproché ») tient-elle sous L1 ? Le
  refus « compte clos » ne vaut-il bien que pour un import vide ou laissé par L1 ? Reste-t-il, dans une partie
  **vivante** de la fiche, du registre ou des tâches, une position ancienne de la garde de W24 (« après l'étape 2,
  avant le comptage », « entre les étapes 1 et 2 ») hors des entrées passées et du texte annoté ?
- **T-a27, lignes W24 (a) et (b)** : construites comme la fiche le dit (b par l'API : import, rapprochement de toutes
  les transactions, clôture — la clôture passe-t-elle alors ?) ; tuent-elles M54 et M55 ? « dix-huit lignes »
  (16 + 1 + 1) est-il juste ?
- **T-a49** (neuf, montage de F-M1) : **réalisable** avec `attendre_une_requete_en_cours`
  (`crates/kesh-db/src/test_fixtures.rs:561-592`) et le motif `["idx_bank_transactions_import", "FOR UPDATE"]`
  (`LOCK_IMPORT_TRANSACTIONS_SQL`, `bank_imports.rs:373-375`) — désigne-t-il la seule requête bloquée ? La prémisse
  (l'étape 2, plage vide pour N par `FORCE INDEX (idx_bank_transactions_import)`, attend la ligne non commitée de
  l'import N+1 d'un autre compte) est-elle plausible sous InnoDB REPEATABLE READ — et le test la **contrôle-t-il**
  vraiment à son étape (3) ? La clôture de A passe-t-elle sans attendre (qui tient quoi sur la ligne de A ?) ? Sous M56
  (et M54), l'import serait-il vraiment supprimé ? Le repli écrit (M56 « non tuée, assumée ») est-il borné juste
  (MariaDB 10.11, CI et dev ; production 10.11+) ?
- **M54 reclassée, M55, M56** : chaque énoncé est-il une mutation **unique et réalisable**, et le test nommé la tue-t-il ?
- **#651** : cité partout où L1 est nommée (AC7, AC14, angles morts, AC16, CHANGELOG prescrit, registre, T4, T9, T14) ;
  corps de PR (28-4a `refs #642` + `refs #651` ; 28-4b `closes #642` + `refs #651`, jamais `closes #651`) ; la borne de
  la promesse de #642 (« hors de la fenêtre de #651 ») dite au périmètre, à l'AC14, au CHANGELOG — et **nulle part
  contredite** (cherche « toute écriture », « rien ne s'y enregistre », « plus rien »). Le texte de #651 décrit-il la
  même fenêtre que l'AC7 ?
- **Restauration d'installation** (R2 de la P4) : `backup.rs:496` / `:548` / `restore_body` l. 473 /
  `restore_tables_in_tx` l. 444 / `routes/admin.rs:295` exacts au sol ; les numéros annoncés pour la 28-3 (l. 528, 557,
  581, 633) exacts dans `c1667e8a` ; le classement « angle mort au doc-comment, **pas** entrée du registre » est-il
  cohérent avec le diff dans les deux sens de l'AC8 et avec la liste des statuts (`HorsPerimetre` réécrit) ?
- **Bras `DbError::BankAccountClosed`** dans le `match` d'`AppError::Database` (`errors.rs:2905`) et rendu partagé :
  exact au sol ? toutes les voies de remontée nommées (W8, W9, W22 par `import_create_error`, W24 par la route, W25,
  W27-W30, `PUT`) y aboutissent-elles ?
- **Les LOW corrigés** (R4-R10, R12, F-L1, F-L2, F-L5-F-L7) : chaque correction est-elle juste, et sans résidu ailleurs
  (grep de la **valeur** : `1562`, `1596`, `l. 862`, `l. 79`, `étape (5)` pour W27, `n'importe où`, `seul site connu`,
  `admin_backup/import.rs`, `tenue par le lot`, `au moment même`) ?
- **Effet de la 28-3 sur la fiche** (`c1667e8a`, sur `origin/main`) : change-t-elle un fait sur lequel la fiche
  raisonne — W24 (`bank_imports.rs`, 6 lignes), la restauration, l'export, l'aller-retour `.keshbackup` de T-a11, les
  fichiers communs ? C'est une **entrée** du rebase (T0), mais une prémisse fausse se signale dès maintenant.
- **Recomptes de la remédiation**, chacun **depuis la source** : 18 AC ; T-a1-T-a49, cinq retirés, **44 actifs** ;
  **59** numéros de mutation (M1-M56 + 3 `-bis`), 5 sans objet (M10, M19, M35, M49, M50), **3 équivalentes**
  (M14-bis, M20-bis, M51), **51 à tuer** ; 16 sites « Gardée » hors W30 ; dix-huit lignes pour T-a27 ; **24** choix ;
  et le trend du Change Log (P1 R 0/1/6/7, F 0/3/3/5 ; P2 R 0/0/4/2, F 0/1/5/7 ; P3 R 0/1/2/5, F 0/1/2/6 ; P4 R 0/0/3/9,
  F 0/0/2/7) contre les rapports. Vérifie aussi que les entrées « Fiche créée », P1, P2 et P3 du Change Log sont
  **intactes** (`git show f923f153:<fiche>`), et que le registre n'a reçu que des **ajouts** (`git diff f923f153 968a9ee3
  -- <registre> | grep '^-'`).

## Axes communs (repris de la P4)

1. **Périmètre minimal** : la section *Périmètre minimal* contre #642, phrase par phrase, **avec ses deux bornes
   neuves** (fenêtre de #651, `cancel_batch`) ; l'extension n° 6 (archivé) bornée comme elle le dit. Le décompte des
   modules et la dérogation (C-28-4-1).
2. **Inventaire clos des écrivains** (W1-W30, R1-R8, R5 retiré) : refais les `grep` et élargis-les — les sept tables
   sous toutes graphies, `format!` et `QueryBuilder` (y compris un `format!(` dont la chaîne est à la ligne suivante),
   les colonnes `bank_account_id`, `settlement_bank_account_id`, `is_primary`, `journal_account_id`, `status`,
   `auto_match_rejected_at`, `archived`, `closed_on` ; crates annexes, `backup.rs`, `admin_backup`, `post_restore`,
   `test_endpoints.rs`. Un écrivain absent, ou mal classé, est au moins **HIGH**. Chaque classement **Invariant**
   contre **tous** les écrivains de l'état et contre L1.
3. **Manuel contre code** (obligatoire) : `docs/manual/fr/user-manual.tex` — § *Comptes bancaires* (l. 1642 et
   suivantes), § *Supprimer un import* (l. 1724, l. 1754-1757 — et ce que l'AC14 y fait écrire sur l'ordre des refus),
   § *Import bancaire* (l. 1696), § *Solde comptable* (l. 238-262), le règlement par virement ;
   `docs/manual/fr/admin-manual.tex` — l. 1943, § *Rollback en cas d'échec* (l. 2234, l. 2251), § *Passer à la 0.13.2*
   (l. 1972). Ce que l'AC14 prescrit d'écrire — dont les phrases sur L1, la fenêtre de l'import, l'exception du lot et
   l'ordre des refus de W24 — sera-t-il vrai au code prescrit ? Contrôle aussi le **PDF**, aplati :
   `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '`.
4. **Recomptes** : chaque nombre de la fiche depuis la source (migrations 80 → 81, compteurs de
   `docs/migrations-idempotence-audit.md`, `migrations_upgrade_path.rs` et ses lignes `\b(80|46)\b`,
   `migrations.sha384`, routes 115 / 118 et les assertions d'`audit_route_registry.rs`, `sitesTotal` 2075, les 29 sites,
   les seize clés UI, numéros de ligne cités).

## Lentilles

- **R — Relecture contre le code au sol (auditeur d'acceptation)** : `.claude/skills/bmad-create-story/checklist.md` ;
  l'axe obligatoire ; l'axe premier ; chaque phrase de #642 couverte par un AC testable, chaque AC par une tâche et
  par un test **qui rougirait**, chaque mutation par un test qui la tue ; chaque fait, chemin et numéro de ligne cité
  **revérifié** (`sed -n`, `grep -nF`) ; les textes prescrits (i18n ×4, manuels, CHANGELOG `### Ajouté` et
  `### Corrigé`, `api-external.md`) confrontés à ce que le code prescrit fera ; les gardes existantes qui changent de
  sens — la fiche les nomme-t-elle toutes ?
- **F — Faisabilité, chemins d'écriture et concurrence (adversaire de la conception)** : l'axe obligatoire en entier ;
  l'axe premier pour T-a49, C-28-4-24 et la clause finale ; L1 sous InnoDB REPEATABLE READ — fenêtre non déclarée, ou
  déclarée et inexistante ? relecture **après** une attente qui dépend de la fraîcheur de la vue, que la position d'une
  garde condamnerait ? ; T-a31, T-a48, T-a49 et leurs prémisses ; la contrainte `CHECK` de l'AC1 et le non-bump de
  `min_required` (C-28-4-11) ; le refus par proposition dans `accept` / `reject` contre § *Pattern batch* ; le
  registre statique de l'AC8 (cinq statuts, angles morts) et son second inventaire ; l'AC6-bis route par route
  (mappage `DbError` → `AppError` lu, code HTTP rendu).

## Ce que tu rends

Findings numérotés (`R1-…` ou `F1-…`) avec sévérité **CRITICAL / HIGH / MEDIUM / LOW**, `fichier:ligne`, **preuve**
(commande exécutée et sortie copiée), correction proposée ; pour chacun, dis s'il vient de la **remédiation P4**
(commit `968a9ee3`), d'une remédiation antérieure, ou de la conception d'origine. Toute affirmation de présence ou
d'absence = sortie de `grep -nF` copiée. **Les deux lentilles rendent la table de l'axe obligatoire, à deux volets.**
⛔ **La liste des axes exercés ET non exercés** — un « 0 finding » sans elle ne compte pas comme passe. Un verdict de
concurrence est un raisonnement statique sur le code : écris-le comme tel, jamais comme une observation. Rapport dans
`/home/gcorbaz/devel/kesh-gate-logs/28-4-validate-p5-R.md` (lentille R) ou
`/home/gcorbaz/devel/kesh-gate-logs/28-4-validate-p5-F.md` (lentille F) ; dernier message : le chemin du rapport, le
bilan par sévérité, une ligne par finding MEDIUM ou plus.

## Interdits

⛔ N'écris aucun fichier hors `/home/gcorbaz/devel/kesh-gate-logs/`. Aucune commande qui écrit, compile ou exécute :
`scripts/*` (dont **`scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/mem-guard.sh`,
`scripts/regen-test-schema.sh` et les scripts Synology), `make`, `latexmk`, `xelatex`, `git commit`/`add`/`checkout`/
`switch`/`stash`/`fetch`/`pull`/`reset`/`rebase`/`tag`/`push`, `cargo`, `npm`, `npx`, `node`, **`python3`, `python`,
`perl`, `ruby` et tout autre interpréteur**, `docker`, **`mariadb`, `mysql` et toute requête SQL**, `bash` sur un script
du dépôt, `gh` en écriture. Rien de `~/kesh-test/` (relevé réel de Guy) ne se lit ni ne se cite ; aucune donnée réelle
dans le rapport. Autorisés : lecture, `grep`, `sed -n`, `awk` en lecture, `git log`/`show`/`diff`/`blame`, `gh api` en
lecture, `pdftotext` vers la sortie standard (ou vers un fichier d'aplatissement **dans** `kesh-gate-logs/`).
