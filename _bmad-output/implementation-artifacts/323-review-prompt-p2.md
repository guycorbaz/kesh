# Prompt — revue de code P2 de la Story 32-3, passe ciblée, une lentille (regression hunter), Opus

Tu relis la **remédiation de la revue de code P1** de la Story 32-3 de Kesh (comptabilité suisse, Rust/Axum, MariaDB) :
*la couche de services — le motif, sur deux modules pilotes (rapports, plan comptable)*. Worktree
`/home/gcorbaz/devel/kesh-32-3`, branche `story/32-3`. **Passe ciblée** (`CLAUDE.md` § *La passe ciblée*) : une seule
lentille, braquée sur **le seul diff de la remédiation**, et non sur la story entière. Le diff à relire :

```sh
git -C /home/gcorbaz/devel/kesh-32-3 diff 210c5808 HEAD
```

(`210c5808` = dernier commit avant la remédiation ; la remédiation tient en deux commits : un commit de test, un commit
de documentation.) Langue du rapport : français. Modèle : **Opus**. Contexte frais : ne t'appuie sur aucune passe
antérieure ; les rapports P1 (`/home/gcorbaz/devel/kesh-gate-logs/323-review-p1-{B,E,A}.md`) se lisent pour savoir ce
que la remédiation prétend régler, non comme vérité.

Lis aussi, de la fiche `_bmad-output/implementation-artifacts/32-3-services-motif.md`, l'entrée « Revue de code P1 » du
Change Log et les annotations datées du Dev Agent Record ; du registre
`_bmad-output/implementation-artifacts/epic-32-choix-autonomes-323.md`, l'annotation de C-32-3-39. Les journaux de la
remédiation : `/home/gcorbaz/devel/kesh-gate-logs/323-p1rem-*.log` et `323-p1rem-mut-*.diff` (lecture seule). La
remédiation ne touche **aucune ligne de production** (à vérifier : `git diff --stat 210c5808 HEAD`).

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier, hormis ton rapport. **Aucun fichier temporaire**, ni
  dans `kesh-gate-logs`, ni dans le scratchpad, ni ailleurs ; **aucune redirection** (`>`, `>>`, `tee`), pas même vers
  un fichier « jetable » : compare par des commandes enchaînées (pipe), lis les PDF vers la sortie standard seulement.
  Un fichier créé par erreur se **déclare** au rapport ; il ne se supprime pas (supprimer, c'est écrire).
- **Aucun `git checkout` ni `git switch`, sous aucune forme** (ni `git checkout -- <fichier>`, ni `git restore`), aucun
  `git` qui écrit (`commit`, `stash`, `reset`, `tag`, `push`, `rebase`, `worktree`, `add`).
- **N'exécute rien qui écrit, compile ou exécute du code** : ni **`scripts/prepare-release.sh`**, ni
  `scripts/test-fast.sh`, ni `scripts/mem-guard.sh`, ni `scripts/install-hooks.sh`, ni aucun script de
  `kesh-gate-logs` ou du scratchpad, ni `make`, ni `latexmk`, ni `cargo` (aucune sous-commande, `cargo fmt` compris),
  ni `rustc`, ni `npm`/`npx`/`node`, ni `docker`, ni `mysql`/`mariadb`, ni `sqlx`, ni `curl`, ni `python3`, ni
  `sed -i`, ni `perl`. Aucune commande `gh` qui écrit ; `gh issue view 723|729|730|431` est permis.
- Permis : `git diff`, `git show`, `git log`, `git grep`, `git ls-files`, `git ls-tree`, `grep`, `sed -n`, `awk` (sans
  redirection), `cat`, `ls`, `find`, `wc`, `pdftotext <f>.pdf - | tr '\n' ' ' | tr -s ' '`.
- **Toute affirmation d'absence ou de présence** s'appuie sur une sortie **`grep -nF`** (ou `sed -n`) copiée.

## Lentille — regression hunter (le diff de la remédiation seulement)

1. **Le test `grand_livre_les_deux_regimes`** (`crates/kesh-api/tests/services_motif.rs`) : tue-t-il **chacune** des
   mutations que l'entrée de Change Log déclare tuées — export tronqué à `MAX_LEDGER_LIMIT`, export qui lit `offset`,
   affichage sans plafond par défaut, affichage sans `offset` lu ? Lis la prémisse **au code du générateur**
   (`crates/kesh-report/src/general_ledger.rs`, `generate` et `fetch_lines` : le plafond borne-t-il par compte ou au
   total ? `limit` est-il écrêté, et à quoi ?) et dis si la fixture (`MAX_LEDGER_LIMIT + 1` lignes sur un compte) est
   la bonne. Recoupe les journaux `323-p1rem-mut-*.log` et les diffs `.diff` (une mutation par diff, diff résiduel nul
   déclaré). Le test peut-il passer **à vide** (section absente, comparaison de deux listes vides, ordre des lignes non
   déterministe — même date, même écriture : quel tri ?) ? Une mutation plausible du régime **non jouée** survivrait-elle
   (p. ex. l'export qui lit `limit`, l'affichage qui ignore `limit` explicite) ?
2. **La ligne 8 de `TABLE_DES_TRACES`**, passée à un export filtré (`journal: Some("Achats")`) : la table reste-t-elle
   de dix-huit lignes, et la couverture du `journal_filter: null` en export est-elle encore tenue ailleurs (lignes 2, 4,
   6, 10 : vérifie qu'elles sont bien des exports **à période**, passant par la même branche de `tracer`) ? Le volet
   (b′) (`table_des_traces_par_la_route`) envoie-t-il `journal=Achats` à `/api/v1/reports/journals/export`, et la route
   le lit-elle ? La mutation E2 (`journal_filter.as_deref()` → `None` dans la seule branche d'export de `tracer`) est-elle
   tuée, et par quelles lignes ? Le changement de nom de la ligne casse-t-il une référence (fiche, journaux) ?
3. **Les comptes rendus** : l'annotation datée du Dev Agent Record (« 85 fichiers lus ») et celle de C-32-3-39 au
   registre sont-elles **exactes** (recompte : `find crates/kesh-api/src -name '*.rs' | wc -l` à HEAD,
   `git ls-tree -r --name-only 08a6f1af crates/kesh-api/src | grep -c '\.rs$'` à la base) et **ajoutées** sans réécrire
   l'entrée d'origine ? L'annotation A2 dit-elle vrai de `323-t3-cible.log` (`grep -c "fmt\|clippy"` sur le journal) ?
   L'entrée « Revue de code P1 » du Change Log : trend, modèles, sort de chaque finding (B1-B5, E1-E4, A1-A6), gates
   et leurs chiffres — recoupés aux journaux `323-p1rem-*.log` ? Le décompte de tests (20 dans `services_motif`, gate
   complet) est-il recompté depuis la source avec son périmètre ?
4. **Gates** : le gate complet et l'E2E complet ont-ils tourné **au dernier commit de code** de la remédiation (révision
   en tête des journaux contre `git log --oneline 210c5808..HEAD`) ? Les échecs E2E sont-ils jugés fichier par fichier
   contre `docs/testing.md` § *Les échecs attendus* ?
5. **Messages de commit** : `git log 210c5808..HEAD --format=%B | grep -niE '\b(close[sd]?|fix(e[sd])?|resolve[sd]?)
   #[0-9]'` doit ne rien rendre ; chaque commit porte `refs #723`.

## Rapport

Écris ton rapport dans `/home/gcorbaz/devel/kesh-gate-logs/323-review-p2.md` :

- bilan en tête : CRITICAL / HIGH / MEDIUM / LOW ;
- findings numérotés `P2-1…`, chacun avec sévérité, fichier:ligne, **preuve copiée** (sortie `grep -nF`, `sed -n` ou
  `git diff`), correctif proposé ;
- **la liste des axes réellement exercés ET de ceux qui ne l'ont pas été** — un « 0 finding » sans cette liste ne compte
  pas comme passe ;
- rien d'autre n'est écrit nulle part.
