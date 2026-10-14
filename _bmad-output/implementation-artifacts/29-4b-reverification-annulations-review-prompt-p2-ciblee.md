# Story 29-4b — revue de code P2, passe ciblée (une lentille : chasseur de régressions de la remédiation P1)

*Prompt versionné de la passe 2 de `bmad-code-review` de la clé `29-4b-reverification-annulations` (#651). Passe
**ciblée** (`CLAUDE.md`, § *La passe ciblée*) : **une seule lentille**, en **contexte frais**, braquée sur **le seul
diff de la remédiation de la P1**. Modèle : celui qu'indique l'orchestrateur.*

## Contexte

- Dépôt : `/home/gcorbaz/devel/kesh-29-4`, branche `story/29-4b`. **Périmètre** : le diff aplati
  `git diff 4862cd0e..HEAD` — rien d'autre. `4862cd0e` est le dernier commit relu par la P1 ; le diff porte la
  remédiation : `7dabbfcf` (test B-1), `9848de33` (code, doc-comments, `docs/api-external.md`) et le commit de
  compte rendu qui les suit (fiche, registre des choix).
- Les rapports de la P1, qui disent ce que la remédiation devait corriger :
  `/home/gcorbaz/devel/kesh-gate-logs/29-4b-review-p1-{A,B,E}.md`. La fiche :
  `_bmad-output/implementation-artifacts/29-4-cloture-compte-concurrente.md` (Change Log, entrée « Revue de code P1 de
  la 29-4b » ; Dev Agent Record, § *Partie b*, sous-section de la remédiation). Registre :
  `_bmad-output/implementation-artifacts/epic-29-choix-autonomes-29-4.md` (C-29-4-33 annoté, C-29-4-38, C-29-4-39).
- `CLAUDE.md` à la racine fait foi (*Review Iteration Rule*, *La passe ciblée*, *Recompter ses propres comptes rendus*,
  *Propagation post-patch*).
- **Manuel** : la remédiation **ne touche pas** `docs/manual/` — le vérifier (`git diff 4862cd0e..HEAD --stat --
  docs/manual`). Si un texte de la remédiation contredit ce que dit le manuel, le relever : contrôler alors le **PDF**
  aplati, `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '`, § *Clore un compte bancaire*
  (puce « ne passe pas sur le compte clos »), contre `docs/api-external.md` l. 242 et 571 réécrites.

## Interdits — lecture seule stricte

- **Aucune** commande qui écrit, compile ou exécute : ni `cargo` (aucune sous-commande), ni `npm`, ni `make`, ni
  `docker`, ni `mariadb`, ni `sqlx`, ni `python3`, ni `perl`, ni `sed -i`, ni script du dépôt — **en particulier**
  `scripts/prepare-release.sh`, `scripts/test-fast.sh`, `scripts/mem-guard.sh`.
- **Aucun** `git checkout`, `git switch`, `git stash`, `git reset`, `git commit`, sous **aucune** forme. Seules
  commandes `git` permises : `git diff`, `git show`, `git log`, `git grep`, `git blame`.
- Aucun fichier écrit hors de ton rapport. Permis : `Read`, `grep`, `rg`, `pdftotext … -` vers la sortie standard.
- Toute affirmation de présence ou d'absence s'appuie sur une sortie `grep -nF` **copiée** dans le rapport.

## Axes (tous à déclarer exercés ou non exercés)

1. **Le test neuf de B-1** (`the_last_acquisition_rule_sees_each_locking_token`,
   `crates/kesh-api/tests/bank_account_closed_inventory.rs`) : chaque cas porte-t-il **un seul** jeton ? (un littéral
   qui en porterait deux — `FOR UPDATE` contient `UPDATE` — ferait-il nommer le mauvais ? `aplatir` rogne-t-il l'espace
   final dont dépend `UPDATE ` ?) ; la table des cas est-elle **réellement** indépendante des listes, de sorte que
   retirer un jeton d'une liste rougisse par son cas ? un jeton ajouté sans cas rougit-il ? le décompte anti-muet
   (dix) est-il juste ? Le journal `/home/gcorbaz/devel/kesh-gate-logs/294b-mutations.log` (M-B1a, M-B1b) rend-il
   compte de ce que le code permet ?
2. **T29b-1** (`annuler_pendant_une_cloture`, `bank_accounts_closed_e2e.rs`) : l'assertion neuve sur la sonde —
   chemin d'échec (la clôture est-elle annulée avant la panique ? la tâche est-elle jointe ?) ; le doc-comment dit-il
   vrai de ce qui tue désormais M29-8 à M29-12 ?
3. **Les doc-comments réécrits**, phrase par phrase, contre le code : (6 bis) de `reconciliation_cancel::cancel_in_tx`
   (la route prend-elle bien `GET_LOCK` avant la transaction ? le délai, 5 s, est-il juste ?) ; la limite « deux
   comptes différents » (W9 imbriquant W25) ; (4 bis) de W28 et W29 ; § *Angle mort* de
   `bank_accounts::recheck_open_locked_in_tx` (les cinq appelants, « tout commit sur la ligne du compte », `1020` non
   rejoué, l'import et le lot par la variante `_unarchived_`) ; le bras `NotFound` dit inatteignable (vrai ? la
   restauration est-elle bien la seule suppression ?) ; `primary_flag_locked_in_tx` (B-2 : ce que le test de R-2 prouve
   et ne prouve pas ; l'exception de l'intégration, `upsert_primary_in_tx`, est-elle exacte ?) ; le cycle « pour les
   quatre annulations qui tiennent un exercice » et la victime de T29b-3 (E-2) ; les doc-comments de l'archivage
   (`routes/bank_accounts.rs`) ; `bank_imports.rs` (étape 5).
4. **`docs/api-external.md`** l. 242 (attente de W9 sous le verrou de réconciliation, `409
   RECONCILIATION_ACCOUNT_LOCKED`, liste des gestes) et l. 571 (état laissé par la v0.13.3) : vrais du code ? cohérents
   avec l. 610 et avec `routes/reconciliation.rs` (`closed_account_failed`) ?
5. **Propagation** : la remédiation a-t-elle laissé ailleurs dans le dépôt une copie du symptôme corrigé ? Greper la
   valeur : « un import ou un lot » (comme appelant de `recheck_open_locked_in_tx`), « clôture commitée » (comme seule
   cause du `1020`), « même cycle », `archive_for_company` comme lieu du verrou dans les doc-comments, « ou terminée »,
   « hors de cette fenêtre ».
6. **Comptes rendus** : l'entrée du Change Log et la sous-section du Dev Agent Record — décomptes recomptés depuis la
   source (tests neufs, findings corrigés et laissés, chiffres des gates contre les journaux
   `294b-gate-p1-remed.log`, `294b-p1-*.log`) ; aucune entrée **passée** réécrite (seules des annotations datées).

## Rapport exigé

Fichier : `/home/gcorbaz/devel/kesh-gate-logs/29-4b-review-p2-ciblee.md` (jamais sous `target/`). Contenu :

1. **Axes exercés** et **axes non exercés**, listés — un « 0 finding » sans cette liste ne compte pas.
2. Findings numérotés `R-n`, chacun : sévérité (`CRITICAL` / `HIGH` / `MEDIUM` / `LOW`), fichier et ligne, constat,
   preuve (sortie copiée), remède, et **origine** (remédiation P1, ou antérieure — dans ce cas, le dire et le classer
   séparément).
3. Décompte final par sévérité, recompté depuis la liste ; et la question qui permet de clore : la remédiation de
   cette passe toucherait-elle du **code de production** ?
