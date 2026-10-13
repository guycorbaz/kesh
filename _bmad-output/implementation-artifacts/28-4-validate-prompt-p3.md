# Prompt — validation P3 de la spec, Story 28-4 (clore un compte bancaire, en lecture seule)

*Versionné le 2026-10-13. Passe **complète**, deux lentilles, contexte frais chacune, en lecture seule. Rotation (D6,
rétro 25) : passes complètes Sonnet ↔ Opus ; Haiku réservé à une passe ciblée de fin de boucle. P1 : Sonnet ; P2 :
Opus ; **P3 : Sonnet** pour les deux lentilles. La remédiation de la P2 a été écrite par Opus et **change la
conception** de la concurrence : ce n'est pas une passe ciblée, c'est toute la fiche que tu relis, en commençant par ce
qui vient d'être écrit.*

Dépôt `/home/gcorbaz/devel/kesh-28-4`, branche `story/28-4` (base `5aad8b35`, kickoff de l'Epic 28).
Fiche : `_bmad-output/implementation-artifacts/28-4-cloture-compte-bancaire.md` (une fiche, **deux** stories :
28-4a serveur, AC1-AC10 et AC6-bis ; 28-4b écran, AC11-AC17). Epic :
`_bmad-output/planning-artifacts/epic-28-recette-v0-13-2.md` § 28-4. Issue #642 (`gh api repos/guycorbaz/kesh/issues/642`
et `…/642/comments`). Registre : `_bmad-output/implementation-artifacts/epic-28-choix-autonomes.md`, section *Story
28-4* (C-28-4-1 à C-28-4-22 ; **C-28-4-21 et C-28-4-22 neufs** ; C-28-4-16 et C-28-4-17 **remplacés** par C-28-4-21,
C-28-4-14 en partie, C-28-4-12 annoté). Story voisine non mergée (W30, R8) : `story/28-5`, fiche
`28-5-transaction-rejetee-atteignable.md` (`git show story/28-5:_bmad-output/implementation-artifacts/28-5-transaction-rejetee-atteignable.md`
si la branche est visible ; sinon, le dire).

**La remédiation à relire** : `git show 9fc879a1` (fiche + registre + `sprint-status.yaml`), soit
`git diff 12426723 9fc879a1`. Les rapports de la P2 (hors dépôt, à lire pour savoir ce qui a été demandé et ce qui a
été déclaré sans objet) : `/home/gcorbaz/devel/kesh-gate-logs/28-4-validate-p2-R.md` et `…-p2-F.md`. L'entrée P2 du
Change Log de la fiche dit, finding par finding, ce qui a été corrigé et ce qui est « sans objet après
simplification ».

**La décision qui gouverne cette passe** (C-28-4-21, AC7) : la garde « compte clos » ne prend **aucun** verrou —
lecture cohérente non verrouillante (`bank_accounts::ensure_open_in_tx`, `ensure_open_unarchived_in_tx`), ou colonne
ajoutée à une instruction qui lit **déjà** la ligne du compte au sol ; l'archivage garde sa séquence au sol ; un angle
mort L1 est assumé (une écriture dont la vue cohérente précède le commit d'une clôture peut encore aboutir). Le
critère décidable est écrit à l'AC7 : *la story n'ajoute aucun verrou de ligne, explicite ou implicite par clé
étrangère, à aucun chemin de `origin/main`, et ne change l'ordre des verrous d'aucun*. Ta passe ne rediscute pas ce
choix de conception — elle vérifie qu'il est **tenu partout** et **dit vrai** ; si tu trouves qu'il laisse #642 non
fermée, c'est un finding, avec sa preuve.

Règles : `CLAUDE.md` (§ Review Iteration Rule et ses sous-sections — le manuel nommé, les axes déclarés, les sites
NON RÉSOLUS, la propagation post-patch, le recompte des comptes rendus ; § Migration breaking policy P1-P8 ; § Pattern
batch ; § Règle de splitting préventif et son amendement D5 ; § Issue Tracking Rule) ; `docs/agents-orchestration.md` ;
la mémoire *invariant dans le temps* (tout reclassement fondé sur un invariant : énumérer les écrivains de l'état et
dire pour chacun pourquoi il ne le casse pas — **et L1 dit désormais que I1 et I2 peuvent être défaits** : tout
raisonnement de la fiche qui les suppose inviolables est à relire).

## Axe obligatoire — le critère « aucun verrou ajouté, ordre inchangé », chemin par chemin (les deux lentilles)

Pour **chaque** ligne de la table *Position par chemin* (Dev Notes § *Concurrence*) **et** pour tout chemin d'écriture
de l'inventaire (W1-W30, R1-R8) qui n'y figurerait pas :

1. écris, **depuis le code au sol** (`5aad8b35`, `sed -n` des lignes), la suite de ses verrous dans l'ordre — y compris
   les **implicites** : verrou partagé sur le parent posé par une clé étrangère à l'`INSERT` d'un enfant, ou à
   l'`UPDATE` d'une colonne à clé étrangère ; verrous de trou ; verrous pris par les fonctions appelées
   (`journal_entries::create_in_tx`, `reverse_owned_in_tx`, `reverse_in_tx`, `letterings::dissolve_*`,
   `refuse_if_claim_account`, `company_invoice_settings::*`, `flip_primary_off_for_company`, l'audit) ;
2. écris ce que la fiche fait ajouter à ce chemin (instruction élargie de colonnes, lecture non verrouillante neuve,
   lecture existante élargie, rien) ;
3. réponds, avec preuve : **un verrou est-il ajouté ?** (une instruction neuve ou élargie porte-t-elle `FOR UPDATE`,
   `LOCK IN SHARE MODE`, `FOR SHARE`, ou est-elle un `INSERT` / `UPDATE` de colonne à clé étrangère ? une colonne
   ajoutée à un `SELECT … FOR UPDATE` existant change-t-elle la ligne verrouillée ou l'index utilisé ?) ; **l'ordre
   change-t-il ?** ;
4. compare ta table à celle de la fiche, ligne à ligne, et signale tout écart (ligne fausse, numéro de ligne faux,
   chemin manquant, verrou implicite omis).

Rends le résultat sous forme de table (chemin / suite des verrous au sol / ajout de la story / verrou ajouté ? / ordre
changé ? / écart avec la fiche). **Un verrou ajouté ou un ordre changé que la fiche ne déclare pas est au moins
HIGH.** Un chemin qui ne peut pas tenir le critère doit être nommé et justifié par la fiche, avec le remède
`retry_app_on_deadlock` des **deux** côtés du cycle (C-28-4-21) ; son absence est au moins HIGH.

Points à examiner en particulier :

- **Les deux chemins neufs** (clôture, réouverture) : la fiche dit qu'ils ne prennent qu'un verrou, leur propre ligne,
  en première instruction, sans autre verrou ensuite. Vrai au code prescrit (décomptes non verrouillants, `UPDATE` de la
  même ligne, `audit_log` sans clé étrangère — migration `20260915000001`) ?
- **W25** : `settlement_bank_account_id` ajouté au `SELECT` verrouillé de l'étape (2) — même ligne, même index ?
- **W17 / W20** : `closed_on` ajouté aux lectures non verrouillantes existantes (étape (0) l. 85-99, étape (2)
  l. 427-441) ; la relecture verrouillée de la P1 est-elle retirée **partout** dans la fiche (AC6, AC6-bis, inventaire,
  tâches, tests, mutations) ?
- **Archivage (W15)** : refus « clos » au pré-vol hors transaction (l. 839-845) et dans le `FOR UPDATE`
  d'`archive_for_company` (l. 619-623), après le contrôle de version ; plus aucune garde prise entre la sentinelle et
  le décompte ? L'ordre des refus écrit à l'AC5 est-il celui que produira ce code ?
- **W30 / `unreject` (28-5)** : la conclusion de la P2 sur F1 (plus de cycle avec W24, la route ne prenant que le
  verrou nommé hors InnoDB et sa ligne de `bank_transactions`) est-elle vraie au texte de la fiche 28-5 (§ *Séquence
  de la route*) ?

## Axe premier — les régressions de la remédiation P2

*La sévérité se déplace vers ce qu'on vient d'écrire.* Relis **chaque** changement de `9fc879a1` :

- **L1** (AC7, § *Angles morts*, AC14, C-28-4-21) : la liste des chemins exposés (W9, W17, W22, W24, W25, W28, W29, W30)
  est-elle **complète** et **exacte** — un chemin à garde non verrouillante manque-t-il, un chemin à lecture verrouillée
  (W8, W27, W13-W15) y est-il à tort, ou en est-il absent à tort ? La conséquence dite (« une opération réelle de plus,
  sans erreur comptable ») est-elle vraie pour **chacun** — en particulier pour un import (transactions `pending` sur un
  compte clos : peut-on les traiter après réouverture ? la réouverture est-elle permise avec des `pending` ?), un lot
  `generated` (`cancel_batch`, W21, est-il bien non gardé et atteignable ?), une annulation ? Le remède « rouvrir,
  traiter, clore » est-il praticable par l'API et l'écran prescrits ? La classe « P3 au plus » est-elle tenable contre
  la grille de priorités du `CLAUDE.md` ? La raison de ne pas l'écrire au manuel d'administration tient-elle ?
- **Snapshot** : le raisonnement REPEATABLE READ du § *Concurrence* (vue née à la première lecture non verrouillante ;
  lecture verrouillante = dernière version commitée ; la clôture attend un écrivain qui a **inséré**, par le verrou
  partagé de clé étrangère) est-il juste, et T-a31 en dépend-il comme la fiche le dit ?
- **AC6-bis** : la ligne W17 dit « aucun changement » ; la ligne W22 prescrit l'appariement aux **deux** appels
  (`routes/bank_imports.rs:1312` et `:2199`) par `import_create_error` ; l'affirmation « au sol,
  `create_with_transactions` ne rend jamais `NotFound` » est-elle vraie (lis la fonction entière) ? Le `### Corrigé` du
  CHANGELOG et la note d'`api-external.md` sont-ils réduits partout au règlement vers un archivé (aucune phrase ne
  promet encore de refuser un lot ou un import « créés pendant son archivage ») ?
- **AC8, second inventaire** (C-28-4-22) : les trois appelants de `invoice_settlements::create_in_tx` sont-ils les
  bons (`grep -rn "invoice_settlements::create_in_tx(" crates/*/src`, hors tests et commentaires) ? Le précédent
  `letterings_lexical.rs:577-627` est-il cité juste ? L'extracteur peut-il manquer un appel écrit autrement (import
  `use … create_in_tx`, chemin `super::`, `crate::repositories::invoice_settlements::create_in_tx`) ?
- **Renommage** `lock_open_for_write` → `ensure_open_in_tx`, `lock_open_unarchived_for_write` →
  `ensure_open_unarchived_in_tx` : reste-t-il un ancien nom dans une partie **vivante** de la fiche ou du registre (les
  entrées P1 du Change Log et le texte P1 des choix annotés le gardent à dessein) ?
- **Findings déclarés « sans objet après simplification »** (R1, R2, F1, F2, F3, F5, F9, F11, F12, F13 de la P2) :
  pour chacun, la raison écrite au Change Log est-elle **vraie** ? Un « sans objet » faux est un finding de la
  sévérité de l'original.
- **Tests et mutations** : T-a32, T-a39, T-a40, T-a41, T-a44 retirés — aucune autre partie de la fiche ne s'appuie-t-elle
  encore sur eux ? T-a31 (le seul test de concurrence restant) est-il réalisable avec `attendre_une_requete_en_cours`
  (`crates/kesh-db/src/test_fixtures.rs:561-592`) et ses motifs désignent-ils la requête **bloquée** ? T-a45 (statique)
  tue-t-il M52 ; T-a46 tue-t-il M48 et M11 ; T-a47 tue-t-il M53 pour **chacun** des deux appels ? M10-bis est-il
  vraiment tué par T-a27 ? M51 est-elle équivalente pour une raison vraie ? M9 est-elle tuée par la ligne W9 de type
  `Entry` de T-a27 ? M42 et M43 tiennent-elles avec les deux refus de l'archivage ?
- **Recomptes de la remédiation**, chacun **depuis la source** : 18 AC ; T-a1-T-a47, cinq retirés, 42 actifs ; 56
  numéros de mutation (M1-M53 + 3 `-bis`), 9 sans objet, 3 équivalentes, 44 à tuer ; 16 sites « Gardée » hors W30 ;
  dix-sept lignes pour T-a27 ; 22 choix ; et le trend du Change Log (P1 R 0/1/6/7, F 0/3/3/5 ; P2 R 0/0/4/2, F 0/1/5/7)
  contre les rapports.

## Axes communs (repris de la P2)

1. **Périmètre minimal** : la section *Périmètre minimal* contre #642, phrase par phrase ; l'extension n° 6 (archivé)
   est-elle bornée comme elle le dit ? La simplification a-t-elle retiré quelque chose que #642 exige (« plus rien ne
   s'y enregistre ») d'une manière que L1 ne couvre pas ? Toute exclusion qui laisserait #642 non fermée est au moins
   **HIGH**. Le décompte des modules et la dérogation (C-28-4-1).
2. **Inventaire clos des écrivains** (W1-W30, R1-R8, R5 retiré) : refais les `grep` **et élargis-les** — les sept
   tables sous toutes graphies, les colonnes `bank_account_id`, `settlement_bank_account_id`, `is_primary`,
   `journal_account_id`, `status`, `auto_match_rejected_at`, `archived`, `closed_on` ; les crates annexes,
   `admin_backup`, `post_restore`, `test_endpoints.rs`. Un écrivain absent, ou mal classé, est au moins **HIGH**.
   Chaque classement **Invariant** (I1, I2, I3) contre l'énumération de **tous** les écrivains de l'état **et contre
   L1**, qui peut défaire I1 et I2 : un écrivain classé « hors d'atteinte par I1 / I2 » (W1-W6, W21) le reste-t-il quand
   une `pending` ou un lot `generated` existe sur un compte clos par L1 ? Si non, que fait-il, et la fiche le dit-elle ?
3. **Manuel contre code** (obligatoire) : `docs/manual/fr/user-manual.tex` — § *Comptes bancaires* (l. 1642 et
   suivantes), § *Supprimer un import* (l. 1724, l. 1754-1757), § *Import bancaire* (l. 1696), § *Solde comptable*
   (l. 238-262), le règlement par virement ; `docs/manual/fr/admin-manual.tex` — l. 1943, § *Rollback en cas d'échec*
   (l. 2234, l. 2251), § *Passer à la 0.13.2* (l. 1972). Ce que l'AC14 prescrit d'écrire — dont la phrase neuve sur
   L1 — sera-t-il vrai au code prescrit ? Contrôle aussi le **PDF**, aplati :
   `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '` (les aplatissements de la P2 existent dans
   `kesh-gate-logs/28-4-validate-p2-R-*-flat.txt`).
4. **Recomptes** : chaque nombre de la fiche depuis la source (migrations 80 → 81, compteurs de
   `docs/migrations-idempotence-audit.md`, `migrations_upgrade_path.rs` et ses lignes `\b(80|46)\b`,
   `migrations.sha384`, routes 115 / 118 et les assertions d'`audit_route_registry.rs`, `sitesTotal` 2075, les 29 sites,
   les seize clés UI, numéros de ligne cités).

## Lentilles

- **R — Relecture contre le code au sol (auditeur d'acceptation)** : `.claude/skills/bmad-create-story/checklist.md` ;
  l'axe obligatoire ; chaque phrase de #642 couverte par un AC testable, chaque AC par une tâche et par un test **qui
  rougirait**, chaque mutation par un test qui la tue ; chaque fait, chemin et numéro de ligne cité **revérifié**
  (`sed -n`, `grep -nF`) ; les textes prescrits (i18n ×4, dont les deux messages modifiés de l'AC9 et le test
  `bank_accounts_e2e.rs:2316-2320` qui doit rester vert à l'octet ; manuels ; CHANGELOG `### Ajouté` et `### Corrigé` ;
  `api-external.md`) confrontés à ce que le code prescrit fera ; les gardes existantes qui changent de sens (G9 de
  `textes_coherents.rs` ; `audit_route_registry.rs` ; `audit_label_registry.rs` ; `letterings_lexical.rs` ;
  `test_schema_guard.rs` ; `migrations_upgrade_path.rs` ; `i18n-keys.test.ts` ; `lint-i18n-ownership` ; en-têtes CSV ;
  `bank-accounts-crud.spec.ts`) — la fiche les nomme-t-elle toutes ?
- **F — Faisabilité, chemins d'écriture et concurrence (adversaire de la conception)** : l'axe obligatoire en entier,
  puis : L1 sous InnoDB REPEATABLE READ (vue, lecture verrouillante, verrous de clé étrangère) — y a-t-il une fenêtre
  que la fiche ne déclare pas, ou une fenêtre déclarée qui n'existe pas ? ; T-a31 et sa prémisse (verrou partagé du
  parent à l'`INSERT` d'un enfant) ; la contrainte `CHECK` de l'AC1 et le non-bump de `min_required` (C-28-4-11) ; le
  refus par proposition dans `accept` / `reject` contre § *Pattern batch* et le pré-vol de la 27-4 ; le registre
  statique de l'AC8 et son second inventaire (extraction hors commentaires et hors `mod tests`, fonction englobante,
  deux sens du diff) ; l'AC6-bis route par route (mappage `DbError` → `AppError` lu, code HTTP rendu).

## Ce que tu rends

Findings numérotés (`R1-…` ou `F1-…`) avec sévérité **CRITICAL / HIGH / MEDIUM / LOW**, `fichier:ligne`, **preuve**
(commande exécutée et sortie copiée), correction proposée ; pour chacun, dis s'il vient de la **remédiation P2**
(commit `9fc879a1`), d'une remédiation antérieure, ou de la conception d'origine. Toute affirmation de présence ou
d'absence = sortie de `grep -nF` copiée. **Les deux lentilles rendent la table de l'axe obligatoire.** ⛔ **La liste des
axes exercés ET non exercés** — un « 0 finding » sans elle ne compte pas comme passe. Rapport dans
`/home/gcorbaz/devel/kesh-gate-logs/28-4-validate-p3-R.md` (lentille R) ou
`/home/gcorbaz/devel/kesh-gate-logs/28-4-validate-p3-F.md` (lentille F) ; dernier message : le chemin du rapport, le
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
