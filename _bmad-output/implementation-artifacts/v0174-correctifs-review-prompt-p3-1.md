# Prompt — revue P3 ciblée de la fiche `v0174-correctifs-1` (#819), une lentille, Haiku

Passe **ciblée**, une seule lentille, contexte frais, braquée sur **le seul commit `cc24a86a`** (remédiation de la revue
P2 ciblée, `/home/gcorbaz/devel/kesh-gate-logs/v0174c-review-p2-1.md` : M-1, L-1, L-2, L-3). Modèle : **Haiku**. Langue :
français. Rapport (le **seul** fichier que tu écris) : `/home/gcorbaz/devel/kesh-gate-logs/v0174c-review-p3-1.md`.

Le diff : `git -C /home/gcorbaz/devel/kesh-0174c show cc24a86a` (diff unique, un seul commit). Le PDF
`docs/manual/fr/user-manual.pdf` est binaire : lis-le aplati.

## Interdits absolus

- **Lecture seule** : aucun fichier hormis ton rapport, aucun fichier temporaire, **aucune redirection** (`>`, `>>`,
  `tee`, `2>&1`). **Aucun `rm`**, aucun `git` qui écrit (`checkout`, `restore`, `commit`, `stash`, `reset`, `add`…).
- **Rien qui compile ou exécute** : ni `cargo`, ni `npm`/`npx`/`node`, ni `python`, ni `make`, ni `docker`, ni `curl`,
  ni `claude`, ni `sed -i`, ni **`scripts/prepare-release.sh`**, ni aucun script de `kesh-gate-logs`. **Aucune commande `gh`.**
- Permis : `git show/diff/log/grep`, `grep`, `sed -n`, `cat`, `ls`, `wc`,
  `timeout 60 pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '` puis `grep -cF` à motif fixe.
- ⛔ **Toute affirmation de présence ou d'absence s'appuie sur une sortie `grep -nF` (ou `grep -cF` sur le PDF aplati)
  copiée dans le rapport.** Un finding sans cette preuve ne compte pas.

## Axes

1. **Le manuel dit vrai au code.** `docs/manual/fr/user-manual.tex` (§ *Détection des doublons*, liste des deux limites)
   contre `crates/kesh-api/src/routes/bank_imports.rs` (`legacy_0173_aliases` : ses trois sorties vides) et le test
   `crates/kesh-api/tests/bank_imports_e2e.rs` (`imports_without_alias_819` : l'aperçu ne signale rien, trois transactions
   créées). La première limite dit-elle « sans signal » et la seconde « l'aperçu la montre » ? Les trois causes
   (fichier refusé, IBAN lu autrement, nombre de lignes) sont-elles celles du code ?
2. **Le PDF porte le nouveau texte** (« sans aucun signal », « ne retrouve pas le même relevé », « celle-là, l'aperçu la
   montre ») et plus l'ancien (« Dans les deux cas l'aperçu le montre »).
3. **CHANGELOG** (`[0.17.4]`, puce #819) aligné sur le manuel.
4. **Les commentaires** de `camt053_tests.rs` (test v08) et `camt053_golden.rs` (en-tête) disent vrai ; aucune ligne de
   code de production touchée (`git show --stat cc24a86a`).
5. **Le Dev Agent Record** de la fiche 1 (`_bmad-output/implementation-artifacts/v0174-correctifs-1.md`, entrée P2) cite des
   journaux qui existent (`ls /home/gcorbaz/devel/kesh-gate-logs/v0174c/cible-819-p2.log pdf-controle3.txt make-user3.log`).

## Rapport

Findings (sévérité, fichier:ligne, preuve `grep -nF` copiée, remède) ; liste des axes **exercés** et **non exercés** ;
décompte recompté.
