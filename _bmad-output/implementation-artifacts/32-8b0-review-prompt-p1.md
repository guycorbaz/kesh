# Prompt — revue de code P1 de la Story 32-8b0 (le prédicat partagé des caractères cachés), trois lentilles, Opus

Tu relis le développement de la **sous-partie b0 de la Story 32-8** de Kesh (comptabilité suisse, Rust/Axum,
MariaDB) : *le prédicat partagé des caractères cachés `kesh_core::text::is_hidden_format` (catégories Unicode Cf, Co,
Cn par `unicode-properties` 0.1.4, plus deux listes écrites), ses deux emplois `is_hidden_in_text` (texte sur plusieurs
lignes : la 32-8b et la 32-10a-ii le réemploieront) et `is_hidden_in_line` (valeur d'une ligne), la description d'un
compte bancaire qui l'emploie désormais (changement de comportement), le test déménagé de `bank_accounts.rs` en
`kesh-core`, et l'entrée du CHANGELOG.* Le prédicat est la fondation de la règle « ce que l'utilisateur voit est
exactement ce qui part » des stories suivantes : un caractère caché qu'il laisserait passer est, plus tard, un texte
que l'administrateur approuve sans le voir.

Worktree `/home/gcorbaz/devel/kesh-32-8`, branche `story/32-8b0`. Le diff à relire, **aplati**, limité au code et au
CHANGELOG de la 32-8b0 (les autres commits de la plage sont la fiche et ses prompts de validation) :

```sh
git -C /home/gcorbaz/devel/kesh-32-8 diff origin/main...HEAD -- crates Cargo.lock CHANGELOG.md
```

Langue du rapport : français. Modèle : **Opus**. Contexte frais : ne t'appuie sur aucune passe antérieure.

⚠️ **Seule la 32-8b0 est en revue.** La table des envois (32-8a), la poignée côté serveur et `rendre_visible`
(32-8b), l'écran (32-8c) et les prompts (32-10a) ne sont pas écrits : leur absence n'est pas un finding. La 32-8b0 a
été développée **avant** la 32-8a, sur décision de l'orchestrateur (C-32-8-30) : ce n'est pas un finding non plus.

Lis d'abord (lentilles E et A ; la lentille B lit le diff avant) : la fiche
`_bmad-output/implementation-artifacts/32-8-validation-avant-envoi.md` — fait 10 de *Ce que le code dit*, la
*Partie B0* (AC-01 à AC-04), les tâches *32-8b0* (T-00 à T-02), les mutations **M01 à M08**, le § *Gates* (32-8b0), le
§ *Le manuel (axe de revue)*, le point 11 des *Points à soumettre à Guy*, la *Note pour la 32-10a* (point 1), le
constat **P3-R1** et **P3-R4** du § *À recontrôler*, puis le **Dev Agent Record** et les entrées *32-8b0* du **Change
Log** — puis le registre `_bmad-output/implementation-artifacts/epic-32-choix-autonomes-328.md` (C-32-8-23, C-32-8-28,
et les choix du développement **C-32-8-30 à C-32-8-32**). La référence de l'AC-01 est l'**AC-ii1 de la fiche 32-10a**
(`/home/gcorbaz/devel/kesh-32-10a/_bmad-output/implementation-artifacts/32-10a-prompts-en-base.md`, l. 583-635, et la
note de l'orchestrateur l. 1437-1465), en lecture seule. Le Dev Agent Record déclare des gates, des mutations et un
E2E : **ne les prends pas pour argent comptant**, recoupe-les avec les journaux
`/home/gcorbaz/devel/kesh-gate-logs/328b0-*` (lecture seule). Aucun contrôle visible ne change : pas de capture
attendue.

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier, hormis ton rapport. **Aucun fichier temporaire**, ni
  dans `kesh-gate-logs`, ni dans un scratchpad, ni sous `/tmp`, ni ailleurs ; **aucune redirection** (`>`, `>>`,
  `tee`) : compare par des commandes enchaînées (pipe), lis les PDF vers la sortie standard seulement. Un fichier créé
  par erreur se **déclare** au rapport ; il ne se supprime pas (supprimer, c'est écrire).
- **Aucun `git checkout` ni `git switch`, sous aucune forme** (ni `git checkout -- <fichier>`, ni `git restore`),
  aucun `git` qui écrit (`commit`, `stash`, `reset`, `tag`, `push`, `rebase`, `worktree`, `add`).
- **Aucun interpréteur, rien qui écrit, compile ou exécute du code** : ni `python`/`python3`, ni `perl`, ni `ruby`, ni
  `node`/`npm`/`npx`, ni `bash -c`/`sh -c` d'un script, ni `jq` ; ni **`scripts/prepare-release.sh`**, ni
  `scripts/regen-test-schema.sh`, ni `scripts/test-fast.sh`, ni `scripts/mem-guard.sh`, ni `scripts/install-hooks.sh`,
  ni aucun script de `kesh-gate-logs` ou d'un scratchpad ; ni `make`, ni `latexmk`, ni `cargo` (aucune sous-commande,
  `cargo fmt` compris), ni `rustc`, ni `docker`, ni `mysql`/`mariadb`, ni `sqlx`, ni `curl`, ni `sed -i`.
- **Aucune commande `gh`**, pas même en lecture.
- Permis : `git diff`, `git show`, `git log`, `git grep`, `git ls-files`, `grep`, `sed -n`, `awk` (sans redirection ni
  `system()`), `cat`, `ls`, `find` (sans `-exec`, `-delete`), `wc`, `sort`, `uniq`,
  `pdftotext <f>.pdf - | tr '\n' ' ' | tr -s ' '`.
- **Toute affirmation d'absence ou de présence** s'appuie sur une sortie **`grep -nF`** (ou `sed -n`) **copiée** dans
  le rapport.
- **Données réelles** : aucune ; les fixtures sont synthétiques. Ne lis rien sous `~/kesh-test/`.

## L'axe obligatoire, pour chaque lentille : la dépendance lue à la source

Toute affirmation du code, de ses commentaires, de la fiche ou du Dev Agent Record sur `unicode-properties` 0.1.4 se
vérifie dans `~/.cargo/registry/src/*/unicode-properties-0.1.4/src/{lib.rs,tables.rs}`, ligne citée : la version
d'Unicode (`tables.rs:17`), la recherche et le repli `Unassigned` (`:24-37`, `:129-131`), la fonctionnalité
`general-category` et ce qu'elle exporte (`lib.rs`), **la catégorie de chaque point des deux listes** et de chaque
voisin testé, et les citations de lignes du Change Log (dont l'écart déclaré U+26A0 `:1425` contre `:1424`). Le
`Cargo.lock` : la crate est-elle bien déjà au graphe, en une seule version, et `kesh-import` (seul crate publié)
reste-t-il hors de sa portée (`crates/kesh-import/Cargo.toml`) ?

## Lentille B — Blind (le diff seul, sans la fiche)

Lis le diff aplati **avant** la fiche, comme un relecteur qui ne connaît pas l'intention.

1. **`crates/kesh-core/src/text.rs`** — `is_hidden_format`, `is_hidden_in_text`, `is_hidden_in_line` : la précédence
   des `||` et `&&`, les plages écrites (bornes incluses, aucune inversée, aucune qui déborde sur un voisin visible),
   les commentaires qui nomment une catégorie — sont-ils justes ? Un caractère **invisible** usuel passe-t-il encore
   (cherche : U+00A0, U+2000–U+200A, U+202F, U+205F, U+3000, U+1680, U+2800, U+3164 voisins, U+FFFC, U+FFFD, U+115F
   voisins, U+E0000 voisins) — et si oui, est-ce dans l'angle mort écrit ?
2. **`crates/kesh-api/src/routes/bank_accounts.rs`** — `is_forbidden_in_description` délègue-t-il exactement ? Tout ce
   qui était refusé l'est-il encore ? Le message de refus et son test `tests/bank_accounts_e2e.rs` (non modifié)
   restent-ils justes ? Le doc-comment décrit-il le code ? Un autre site du dépôt recopiait-il l'ancienne liste
   (`grep -rnF "206A" crates frontend/src`, `grep -rnF "is_forbidden_in_description" crates`) ?
3. **Les tests** — un test peut-il passer **à vide** (boucle sur une liste vide, `assert!` qui teste la mauvaise
   fonction, emoji écrit en littéral déjà décomposé, chaîne qui ne contient pas le caractère annoncé) ? Le test
   déménagé (`ancienne_liste_reste_refusee`) couvre-t-il **chaque point** de l'ancienne liste, lue dans
   `git show origin/main:crates/kesh-api/src/routes/bank_accounts.rs` ? Une mutation plausible que **M01 à M08** ne
   couvrent pas survit-elle (par exemple : `is_hidden_in_line` qui oublie `is_control`, `is_hidden_in_text` qui admet
   U+0085, une liste raccourcie d'un point au milieu) ?

## Lentille E — Edge cases

1. **Les catégories** : un `char` peut-il être `Surrogate` ? Un point attribué par Unicode après la 17.0.0 est `Cn`
   pour la crate, donc refusé : est-ce dit, et est-ce testé (`version_d_unicode_epinglee`) ? Les noncharacters
   (U+FDD0–U+FDEF, U+FFFE, U+FFFF, U+1FFFE…) sont-ils `Cn` dans la table, donc refusés ?
2. **Les emplois** : `\r` dans un texte (refusé, M07), `\n` et `\t` dans une ligne (refusés), U+0085 (Cc, NEL),
   U+2028/U+2029 (refusés par la liste, non par `is_control`). Un texte qui contiendrait **seulement** des emojis
   simples ; un emoji avec modificateur de teint (U+1F3FB–U+1F3FF, `ModifierSymbol`) ; un drapeau (indicateurs
   régionaux U+1F1E6–U+1F1FF) ; une séquence de clavier (`1️⃣` = `1` U+FE0F U+20E3) — admis ou refusés, et le
   CHANGELOG le dit-il juste (« les emojis simples restent admis ») ?
3. **La description d'un compte, après la 32-8b0** : une description **déjà enregistrée** qui porte un caractère
   désormais refusé — que se passe-t-il à la mise à jour du compte (`update_bank_account`, la description étant
   renvoyée en entier), à l'archivage, à l'import d'une sauvegarde, à l'export ? Le CHANGELOG le dit-il juste ? Y a-t-il
   un autre chemin d'écriture de `bank_accounts.description` qui ne passe pas par `normalize_description`
   (`grep -rn "description" crates/kesh-db/src/repositories/bank_accounts.rs`, l'import CAMT, l'onboarding, le seed) ?
4. **Le frontend** : fait-il un contrôle propre des caractères de la description (`grep -rn "200B\|FEFF\|is_control"
   frontend/src/lib/features/bank-accounts`) qui divergerait désormais du serveur ?

## Lentille A — Acceptance (la fiche contre le code)

1. **AC-01** : la définition est-elle **exactement** celle de l'AC-ii1 de la 32-10a (les trois catégories, les deux
   listes point par point, l'exception des emojis retirée) ? La dépendance est-elle écrite à la lettre
   (`default-features = false, features = ["general-category"]`) ? `kesh-core` reste-t-il sans I/O ?
2. **AC-02** : `is_hidden_in_text` est-il l'expression écrite, à la lettre ?
3. **AC-03** : `is_forbidden_in_description` vaut-il `c.is_control() || is_hidden_format(c)` (par
   `is_hidden_in_line`, C-32-8-31 — ce choix est-il fondé et tient-il la lettre) ? Le test
   `description_refuse_par_categorie` joue-t-il les cinq points prescrits ? L'entrée du CHANGELOG reprend-elle le texte
   prescrit, et ce qu'elle ajoute est-il vrai au code ? La rectification P3-R1 (U+2065) est-elle juste ?
4. **AC-04** : chaque cas énuméré par l'AC-04 est-il un `assert` du code (liste-les un par un, `grep -nF` à l'appui) ?
   Le test `emojis_simples_admis_composes_refuses` existe-t-il **dans les deux fichiers** prescrits ?
5. **M01 à M08** : chacune est-elle tuée par le test que la fiche nomme (journal `kesh-gate-logs/328b0-mutations.log`) ?
6. **Le Dev Agent Record et le Change Log** : chaque décompte (tests neufs, mutations, gates, E2E) se recompte-t-il
   depuis la source, avec son périmètre ? Les gates déclarés ont-ils tourné (journaux `328b0-gate-*`, `328b0-e2e*`) ?
   Les échecs E2E déclarés sont-ils ceux de la KF-029, et la pollution éventuelle rejouée seule au vert ?
7. **Le manuel et le CHANGELOG** : la fiche dit qu'aucun manuel ne décrit les caractères refusés d'une description de
   compte — vérifie-le dans le **PDF** comme dans le `.tex` (`pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' '
   | tr -s ' ' | grep -oiE ".{80}(description|invisible).{80}"`, et `grep -niE "invisible|description du compte"
   docs/manual/fr/*.tex`). Le CHANGELOG est-il à sa place (section *Modifié* de la 0.16.0) ?
8. **La note pour la 32-10a** : ce que la 32-8b0 livre (noms, signatures, tests) correspond-il à ce que la note promet
   à la 32-10a-ii (réemployer, constater Mii10, Mii12, Mii13) ?

## Le rapport

Un fichier par lentille : `/home/gcorbaz/devel/kesh-gate-logs/328b0-review-p1-{B,E,A}.md`. Chacun contient :

- **les axes exercés et ceux qui ne l'ont pas été**, nommément (un « 0 finding » sans cette liste ne compte pas) ;
- les findings, chacun avec sa sévérité (`CRITICAL`, `HIGH`, `MEDIUM`, `LOW`), le fichier et la ligne, la preuve
  (sortie `grep -nF` ou `sed -n` copiée), et la correction proposée ;
- le décompte final par sévérité.
