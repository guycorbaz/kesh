# Prompt — revue de code P2 **ciblée** de la Story 36-4 (refs #1131) : une lentille, la seule remédiation de la revue P1

Tu relis la **remédiation de la revue P1** de la Story 36-4 de Kesh (comptabilité suisse, Rust/Axum, MariaDB), release 0.17.8 :
l'annulation d'un règlement par une clé d'API est attribuée à la clé. La revue P1 (trois lentilles) s'est close sans rien
au-dessus de LOW (B 0/0/0/2, E 0/0/0/3, A 0/0/0/4 : `/home/gcorbaz/devel/kesh-gate-logs/e36-4/review-p1-{B,E,A}.md`). Sur décision
de l'orchestrateur (« Tarir la source »), ces LOW ont été corrigés **dans la story** ; un correctif touche la production (un
commentaire de `reconciliation_cancel.rs`) et des tests. Cette passe relit **ce seul correctif**, rien d'autre. Modèle : Haiku ou
Sonnet (passe ciblée). Contexte frais. Langue du rapport : français.

Worktree `/home/gcorbaz/devel/kesh-36-4`. **Le diff à relire, aplati** : `git -C /home/gcorbaz/devel/kesh-36-4 diff <BASE> <TETE>`,
où `<BASE>` = `c52e10b50` (la story en `review`, avant la remédiation) et `<TETE>` = le commit de la remédiation (`git log --oneline
c52e10b50..HEAD` le nomme ; le dernier commit ne porte que la fiche et ce prompt).

Lis aussi les trois rapports de la revue P1 (ce que chaque LOW demandait) et, dans la fiche
`_bmad-output/implementation-artifacts/36-4-acteur-annulation-reglement.md`, l'entrée du Change Log « Remédiation de la revue P1 ».
Journaux (lecture seule) sous `/home/gcorbaz/devel/kesh-gate-logs/e36-4/` : `remediation-p1-cibles.log`,
`test-fast-ci-remediation-p1.log`, `manuels-make-remediation-p1.log`.

⚠️ **Ce qui n'a pas tourné** : `gate-ci` et l'E2E (à la clôture). ⛔ Rien sous `~/kesh/` ni `~/kesh-test/`. Aucune donnée réelle.

## Interdits absolus

- **Lecture seule.** N'écris aucun fichier, hormis ton rapport. **Aucun fichier temporaire.**
- **Aucune redirection, sous aucune forme** (`>`, `>>`, `<`, `tee`, `2>`, `&>`, `2>&1`, `2>/dev/null`), aucune substitution de
  processus.
- Ni `rm`, ni `pkill`/`killall`/`kill` ; aucun `git` qui écrit (`checkout`, `switch`, `restore`, `stash`, `reset`, `commit`).
- **Aucun interpréteur ni rien qui compile ou exécute** : ni `python3`, `perl`, `node`, `npm`, `bash -c`, `jq`, `cargo`, `make`,
  `docker`, `mariadb`, `curl`, `claude`, `sed -i` ; ni aucun script du dépôt ou de `kesh-gate-logs` (dont
  **`scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/gate-ci.sh`, `mut/jouer.sh`). Aucune commande `gh`.
- Permis : `git diff`, `git show`, `git log`, `git grep`, `/usr/bin/grep`, `sed -n`, `awk` (sans redirection), `cat`, `ls`, `wc`,
  `pdftotext … -` (sortie à l'écran).
- `grep` est une fonction vers `ugrep` : emploie `/usr/bin/grep` ou `git grep`.
- Toute affirmation de présence ou d'absence s'appuie sur une sortie `grep -nF` ou `sed -n` copiée ; tout chemin cité se vérifie
  (`ls` ou `git cat-file -e <ref>:<chemin>`).
- **Contrôle d'un PDF, seule forme permise (E6)** :
  `pdftotext docs/manual/fr/<f>.pdf - | tr '\n' ' ' | tr -s ' ' | /usr/bin/grep -c '<phrase>'` ; l'existence par `ls -l`.
  ⚠️ `pdftotext` peut faire tomber le trait d'union à une coupure de ligne (« contrepassation »).

## La lentille unique — chasse aux régressions de la remédiation

Pour chaque LOW de la P1, dis s'il est **traité tel que demandé**, et si le correctif **introduit** un défaut :

1. **B1** — `crates/kesh-db/src/repositories/reconciliation_cancel.rs`, commentaire de l'étape (7) : le décompte des traces est-il
   juste au code (règlement soldé : six ; partiel : sans `lettering.removed` ; `unlink_only` : la seule `reconciliation.cancelled`) ?
   **Aucune ligne de code** n'a-t-elle bougé dans ce fichier (`git diff <BASE> <TETE> -- <fichier>` : commentaires seuls) ?
2. **B2 = E1** — `an_api_key_unlinks_one_side_and_is_named` (`crates/kesh-api/tests/reconciliation_link_entry_e2e.rs`) : le montage
   pose-t-il bien un simple détachement (`unlinkOnly` vrai avant le geste, `reversalJournalEntryId` nul après) ? L'assertion
   peut-elle passer à vide (identifiant relevé après la création de la clé ; une seule action attendue) ?
3. **E2** — `an_api_key_cancels_a_partial_settlement_and_is_named_everywhere` et le paramètre `piece_lettree` ajouté à
   `cancel_by_key` (`invoice_settlement_response_e2e.rs`) : les deux appels d'avant passent-ils `true` ; le cas partiel affirme-t-il
   l'**absence** de lettrage de pièce ; la liste `PARTIAL_SETTLEMENT_CANCEL_ACTIONS` est-elle expliquée juste ? La contre-passation à
   l'exercice suivant : le Change Log dit que `reverse_owned_in_tx` ne date qu'au jour (`ReversalDating::Today`) — vrai au code
   (`crates/kesh-db/src/repositories/journal_entries.rs`) ?
4. **E3 = A1** — le témoin de session ajouté à `cancelling_a_supplier_reconciliation_by_the_route` (`reconciliation_e2e.rs`) :
   l'utilisateur attendu (`seeded.admin_user_id`, celui du JWT forgé) est-il le bon ; l'identifiant est-il relevé juste avant le
   geste ? La cellule de la fiche (`| assertions de session ajoutées |`) est-elle accordée ?
5. **A2** — le Dev Agent Record cite-t-il `test-fast-ci-c5d67ce42.log:4` pour `fmt` ; les journaux de gate de la remédiation
   portent-ils le commit en première ligne ?
6. **A3** — `docs/api-external.md:565` : « `lettering.removed` quand la pièce était lettrée ».
7. **A4** — `docs/manual/fr/admin-manual.tex:3212` et `docs/manual/fr/user-manual.tex` (≈ 4019-4025) : « un simple détachement ne
   contre-passe rien » hors de la phrase « depuis la 0.17.8 » ; plus de « solde » fournisseur ; les deux PDF régénérés :
   - `… | /usr/bin/grep -c 'compris, et un simple détachement'` → `0` dans `user-manual.pdf` ;
   - `… | /usr/bin/grep -c 'délettrage ; un simple détachement'` → `0` dans `admin-manual.pdf` ;
   - `… | /usr/bin/grep -c 'client ou fournisseur, par sa route'` → `0` dans `user-manual.pdf` ;
   - `… | /usr/bin/grep -c 'une écriture rapprochée seule, et un simple détachement ne contre-passe rien'` → `1` dans chacun
     (attention aux coupures de ligne : si `0`, cherche « simple détachement ne contre » et dis ce que tu lis).
8. **Le compte rendu de la remédiation** (Change Log) : chaque chiffre au journal qu'il cite, avec son commit.

## Rapport

Écris **un seul fichier** : `/home/gcorbaz/devel/kesh-gate-logs/e36-4/review-p2-ciblee.md`. Findings numérotés (`R1`, `R2`…), sévérité
(CRITICAL / HIGH / MEDIUM / LOW), lieu (`fichier:ligne`), ligne lue copiée, preuve, remède ; **la liste des axes exercés ET non
exercés** ; le décompte final recompté depuis la liste.
