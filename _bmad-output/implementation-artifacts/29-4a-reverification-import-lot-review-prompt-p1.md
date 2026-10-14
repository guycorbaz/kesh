# Story 29-4a — revue de code P1 (trois lentilles B, E, A)

*Prompt versionné de la passe 1 de `bmad-code-review` de la clé `29-4a-reverification-import-lot` (#651). Modèle :
**Opus**. Trois lentilles indépendantes, chacune en **contexte frais**, chacune écrit **son** rapport.*

## Contexte commun aux trois lentilles

- Dépôt : `/home/gcorbaz/devel/kesh-29-4`, branche `story/29-4`. **Base** : `089fcd80` (`main`, Stories 29-2 et 29-3
  mergées). **Périmètre** : le diff `git diff 089fcd80..HEAD` — un seul diff aplati, pas la suite des commits.
- Fiche : `_bmad-output/implementation-artifacts/29-4-cloture-compte-concurrente.md` — elle porte **deux** clés ;
  seule la **partie a** (AC1 à AC5, AC9 part a, AC10) est dans le périmètre. La 29-4b (annulations, manuel,
  `docs/api-external.md`) **n'est pas écrite** et ne doit pas l'être : son absence n'est pas un défaut.
  Registre des choix : `_bmad-output/implementation-artifacts/epic-29-choix-autonomes-29-4.md` (C-29-4-1 à C-29-4-27 ;
  C-29-4-23 à C-29-4-27 sont ceux du développement). Dev Agent Record de la fiche : ce qui a tourné, avec les chiffres.
- `CLAUDE.md` à la racine fait foi (*Review Iteration Rule*, *Migration breaking policy*, *Pattern batch*).
- Fichiers de production touchés : `crates/kesh-db/src/repositories/bank_accounts.rs` (deux SQL et deux fonctions
  neufs, doc-comments), `bank_imports.rs` (appel W22), `payment_batches.rs` (appel W17 et conversion de l'archivé),
  `crates/kesh-db/src/errors.rs` (doc-comment), `crates/kesh-api/src/routes/bank_accounts.rs` (trois enveloppes
  `close_once` / `reopen_once` / `patch_once`, `conclure`), `crates/kesh-api/src/routes/bank_imports.rs`
  (doc-comment). Tests : `crates/kesh-db/tests/{bank_accounts_repository,payment_batches_repository}.rs`,
  `crates/kesh-api/tests/{rejeu_interblocage_e2e,bank_account_closed_inventory,audit_route_registry}.rs`.
  Documents : `CHANGELOG.md` (§ `[0.13.4]`, `### Corrigé`).
- **Manuel** : la partie a ne touche **pas** `docs/manual/fr/user-manual.tex` ni son PDF ; c'est voulu (état
  transitoire écrit à l'AC9 et au Dev Agent Record : la puce l. 1748-1752 reste trop large pour l'import et le lot
  jusqu'à la 29-4b). Une lentille **vérifie** cependant que le manuel et le **PDF** (`docs/manual/fr/user-manual.pdf`,
  aplati : `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '`) ne disent rien qui soit devenu
  **faux dans l'autre sens** (une promesse que la partie a contredit), et le dit.

## Interdits — lecture seule stricte

- **Aucune** commande qui écrit, compile ou exécute : ni `cargo` (aucune sous-commande), ni `npm`, ni `make`, ni
  `docker`, ni `mariadb`, ni `sqlx`, ni `python3`, ni `perl`, ni `sed -i`, ni script du dépôt — **en particulier**
  `scripts/prepare-release.sh`, `scripts/test-fast.sh`, `scripts/mem-guard.sh`.
- **Aucun** `git checkout`, `git switch`, `git stash`, `git reset`, `git commit`, sous **aucune** forme (ni `--detach`,
  ni sur un fichier). Les seules commandes `git` permises : `git diff`, `git show`, `git log`, `git grep`,
  `git blame`.
- Aucun fichier écrit **hors** de ton rapport. Permis : `Read`, `grep`, `rg`, `pdftotext … -` vers la sortie standard.
- Toute affirmation de **présence** ou d'**absence** d'un code s'appuie sur une sortie `grep -nF` **copiée** dans le
  rapport.

## Rapport exigé (chaque lentille)

Fichier : `/home/gcorbaz/devel/kesh-gate-logs/29-4a-review-p1-<B|E|A>.md` (jamais sous `target/`). Contenu :

1. **Axes exercés** et **axes non exercés**, listés — un « 0 finding » sans cette liste ne compte pas.
2. Findings numérotés `<lentille>-n`, chacun : sévérité (`CRITICAL` / `HIGH` / `MEDIUM` / `LOW`), fichier et ligne,
   constat, preuve (sortie copiée), remède proposé.
3. Décompte final par sévérité, recompté depuis la liste.

---

## Lentille B — Blind Hunter (le diff seul, sans la fiche)

Lis **seulement** le diff (`git diff 089fcd80..HEAD -- crates CHANGELOG.md`) et les fichiers qu'il touche ; **ne lis
pas** la fiche ni le registre avant d'avoir écrit tes findings (tu peux les ouvrir ensuite pour reclasser, en le
disant). Axes :

- correction du code : `recheck_open_locked_in_tx` / `recheck_open_unarchived_locked_in_tx` (ordre des liaisons
  `id`, `company_id` contre l'ordre des `?` du SQL ; `match` et ordre « archivé avant clos ») ; appel W22 (entre
  l'`INSERT` d'entête et les transactions) ; appel W17 et sa conversion `NotFound` → `IllegalStateTransition` ;
- les trois enveloppes : la fermeture rejouée refait-elle **toute** la transaction ? un effet de bord hors
  transaction ? une conversion qui ferait sortir un `1213` d'`AppError::Database(DbError::Sqlx(_))` avant le
  prédicat (`crate::retry::is_app_deadlock`) ? `rollback` explicite sur **tout** refus (`conclure`) ? ordre des
  refus inchangé (forme, onboarding, `validate_journal_account_id` devant l'enveloppe) ;
- tests : un test qui passerait **à vide** (sonde jamais exercée, montage qui ne forme pas le cycle, assertion
  vraie au sol) ; tâches non jointes, transactions laissées ouvertes ; minutage ;
- le registre `bank_account_closed_inventory.rs` (troisième inventaire) : un extracteur qui ne voit rien
  ferait-il passer le registre ? Le volet inverse éprouve-t-il réellement chaque assertion ?
- doc-comments : chaque phrase neuve est-elle vraie du code ?

## Lentille E — Edge Case Hunter (verrous, cycles, naissance de la vue)

⛔ **Table obligatoire** dans le rapport : **la table des verrous et des cycles**. Pour chaque chemin touché ou
rejoué — W22 (import CAMT et CSV), W17 (création de lot), clôture, réouverture, `PATCH` du compte — la suite
**ordonnée** des verrous pris (ligne, mode S/X, explicite ou par clé étrangère, intervalle ou non), et pour chaque
verrou **neuf** ou chaque demande **en attente** neuve : les partenaires qui peuvent fermer un cycle avec lui (à deux,
à trois par la file d'attente InnoDB, à quatre), si chaque participant est rejoué, et qui InnoDB désignerait pour
victime (argument de poids : écritures et verrous). Compare ta table à celle de la fiche (Dev Notes § *Position par
chemin* et § *Cycles*) et dis chaque écart. Axes :

- **le critère décidable** de la fiche, trois volets : (1) aucun verrou neuf hors « déjà tenu » (W17, W22) ;
  (2) tout cycle fermé a ses participants rejoués, aux deux résidus écrits près (C-29-4-7, cycle à quatre) ;
  (3) **aucune naissance de vue déplacée** — la revérification est-elle bien après la première lecture non
  verrouillante de chaque chemin, et aucune lecture non verrouillante n'a-t-elle été ajoutée ou avancée ?
- la redemande d'un partagé tenu par clé étrangère (prémisse P-d (2)) : T29a-6 et T29a-7 l'établissent-ils
  vraiment (sonde de référence (1 bis), sonde de file `LOCK IN SHARE MODE NOWAIT`, `timeout`) ? Un cas où le
  verrou de clé étrangère **ne serait pas** tenu au moment de la revérification (p. ex. `INSERT` qui échoue sur un
  doublon, lot sans facture acceptée — `accepted.is_empty()` — qui ne fait pas d'`INSERT`) ?
- `innodb_snapshot_isolation` (angle mort renvoyé à #669) : le doc-comment le dit-il juste ? un chemin où `1020`
  sortirait autrement qu'en `500` ?
- les cas limites des enveloppes : refus métier rejoué à tort ? nombre de tentatives ? `version` du corps
  réutilisée à chaque tentative (correct : la tentative précédente n'a rien commité) ?
- les tests de rejeu T29a-4, T29a-9, T29a-10 : la victime observée est-elle prouvée (événement `warn!` capturé,
  `.expect` des transactions de test) ? Que se passe-t-il si InnoDB désigne une autre victime (seconde contingence de
  l'AC4) — le test le dit-il ?

## Lentille A — Acceptance Auditor (la fiche contre le code)

Lis la fiche (partie a), le registre des choix et le Dev Agent Record, puis le diff. Axes :

- chaque AC de la partie a (AC1 à AC5, AC9 part a, AC10) : satisfait, partiellement, ou non — preuve par
  `grep -nF` ; chaque écart à la fiche est-il **écrit** (registre C-29-4-23 à C-29-4-27, Dev Agent Record) ?
- les tests prescrits (T29a-1 à T29a-7, T29a-9 à T29a-12, T-a45 étendu) : présents, au bon fichier ou déplacés avec
  leur choix, et éprouvant ce que la fiche dit ;
- la campagne de mutations (M29-1 à M29-4, M29-6, M29-7, M29-19 à M29-23, M29-25) : le Dev Agent Record donne-t-il
  pour chacune « tuée par … » ou « équivalente, démontrée », avec ses journaux
  (`/home/gcorbaz/devel/kesh-gate-logs/294-M29-*.log`) ?
- la table *Position par chemin* recopiée au Dev Agent Record avec les lignes retenues, contre le code ;
- `audit_route_registry.rs` : le point (vi) dit « dix » — **recompte** les routes nommées ; les « sept » des points
  (ii) et (iii bis) restent sept (famille `DbError`) ;
- le CHANGELOG (§ `[0.13.4]`, `### Corrigé`) : la ligne de la fiche, par union avec les lignes de la 29-2 et de la
  29-3, sans toucher la section `[0.13.3]` ;
- le **manuel** et son **PDF** : rien n'est prescrit en partie a ; vérifie qu'aucun texte n'a été touché à tort, et que
  l'état transitoire est écrit (AC9) ;
- les gates déclarés : chaque chiffre du Dev Agent Record est-il adossé à un journal de
  `/home/gcorbaz/devel/kesh-gate-logs/` (`294-gate-*.log`, `294-e2e-*.log`) ? Rien n'est-il déclaré qui n'ait tourné ?
- aucun mot-clé de fermeture (`closes`, `fixes`, `resolves`) dans les messages de commit
  (`git log 089fcd80..HEAD --format=%B | grep -niE 'clos(e|es|ed)|fix(es|ed)?|resolv'`) — seul `refs #651` est permis.
