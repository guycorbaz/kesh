# Prompt — revue de code P2 de la Story 32-8b0, passe ciblée, une lentille (regression hunter), Sonnet

Tu relis **une seule remédiation** de la **sous-partie b0 de la Story 32-8** de Kesh (comptabilité suisse, Rust/Axum,
MariaDB) : le commit **`6bfd9872`**, qui applique la revue de code P1 du prédicat partagé des caractères cachés
`kesh_core::text::is_hidden_format` (catégories Unicode Cf, Co, Cn par `unicode-properties` 0.1.4, plus deux listes
écrites) et de ses deux emplois `is_hidden_in_text` (texte sur plusieurs lignes) et `is_hidden_in_line` (valeur d'une
ligne, la description d'un compte bancaire). Passe **ciblée** (`CLAUDE.md` § *La passe ciblée*) : la P1 a rendu trois
MEDIUM et douze LOW, tous sur ce que la remédiation vient d'écrire ou de laisser ; ce qu'il reste à relire n'est plus
la story, c'est ce commit. **Ta seule question : ce commit a-t-il introduit un défaut, ou laissé un résidu du symptôme
qu'il corrige ?**

Worktree `/home/gcorbaz/devel/kesh-32-8`, branche `story/32-8b0`. Le diff à relire :

```sh
git -C /home/gcorbaz/devel/kesh-32-8 show 6bfd9872
```

Langue du rapport : français. Modèle : **Sonnet**. Contexte frais : ne t'appuie sur aucune passe antérieure ; lis les
rapports P1 pour savoir ce qui était demandé, non pour en reprendre les conclusions :
`/home/gcorbaz/devel/kesh-gate-logs/328b0-review-p1-{B,E,A}.md` (lecture seule). Les remédiations demandées et ce qui
a été réfuté sont à l'entrée **« Revue de code P1 »** du Change Log de la fiche
`_bmad-output/implementation-artifacts/32-8-validation-avant-envoi.md` (dernière ligne du tableau). Le journal des
mutations de la remédiation est `/home/gcorbaz/devel/kesh-gate-logs/328b0-mutations-p1.log`, ceux des gates
`328b0-gate-complet-p1.log`, `328b0-gate-frontend-p1.log`, `328b0-e2e-p1.log` : **recoupe-les**, ne les prends pas pour
argent comptant.

⚠️ **Seule la 32-8b0 est en revue.** La table des envois (32-8a), la poignée côté serveur (32-8b), l'écran (32-8c) et
les prompts (32-10a) ne sont pas écrits : leur absence n'est pas un finding. Ne sont **pas** à remédier, par décision
de l'orchestrateur, et ne sont donc pas des findings : le message de refus de la description, qui ne nomme pas un
caractère d'usage privé ou non attribué (B-L4 = E3), la cible « description » de M05 non rejouée (A-L3).

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier, hormis ton rapport. **Aucun fichier temporaire**, ni
  dans `kesh-gate-logs`, ni dans un scratchpad, ni sous `/tmp`, ni ailleurs ; **aucune redirection** (`>`, `>>`,
  `tee`) : compare par des commandes enchaînées (pipe). Un fichier créé par erreur se **déclare** au rapport ; il ne se
  supprime pas (supprimer, c'est écrire).
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

## Les axes de la lentille

1. **Les catégories des points neufs, à la source.** Chaque voisin ajouté à `voisins_visibles_admis` (U+115E, U+1161,
   U+3163, U+3165, U+FF9F, U+FFA1, U+034E, U+0350, U+17B3, U+17B6, U+180A, U+1810, U+FDFF, U+FE10, U+2027) et chaque
   diacritique (U+0301, U+17B7) : sa catégorie dans
   `~/.cargo/registry/src/*/unicode-properties-0.1.4/src/tables.rs`, ligne citée — est-il **visible** et hors des deux
   voies (ni Cf, ni Co, ni Cn, ni dans une liste) ? Un voisin « manquant » que le doc-comment du test dit caché
   (U+1160, U+180E, U+202A, U+E00FF, U+E01F0) l'est-il vraiment ? Un côté visible a-t-il été oublié ?
2. **Les tests neufs prouvent-ils ce qu'ils annoncent ?** `voisins_visibles_admis` (le `chain` des deux tableaux
   parcourt-il bien tout ?), les quatre contrôles ajoutés à `texte_sur_plusieurs_lignes`, l'assertion
   `"Re\u{0301}serve"` de `bank_accounts.rs` (le littéral contient-il bien U+0301, la chaîne n'est-elle pas déjà
   précomposée, `normalize_description` la rend-elle telle quelle ?). Le journal `328b0-mutations-p1.log` montre-t-il
   chaque mutation (MP1 `FFA0..=FFBE`, MP2 `0300..=036F`, MP3 `NonspacingMark` en voie 1, MP4 U+0085 admis dans un
   texte) appliquée, tuée par l'assertion visée, puis restaurée à diff nul ? Une mutation voisine plausible survit-elle
   encore (par exemple `'\u{3164}'` élargi à sa plage, `'\u{17B4}' | '\u{17B5}'` élargi à `17B3..=17B6`) ?
3. **Les doc-comments récrits disent-ils vrai ?** `is_hidden_format` (le critère des emojis refusés : sélecteur
   U+FE0E/U+FE0F ou liant U+200D ; les drapeaux et les teints admis — vérifie les catégories des indicateurs
   régionaux et des modificateurs de teint à la table ; l'angle mort : U+2800, U+1D159, U+16FE4, les `Zs` nommés —
   catégories exactes ?) ; `is_hidden_in_text` (plus aucune promesse de normalisation ; cohérent avec l'AC-B9 de la
   32-8b, fiche l. ≈700-710, et avec la 32-10a) ; `voisins_visibles_admis`.
4. **Le CHANGELOG** (`CHANGELOG.md`, entrée *Modifié* de la 0.16.0) : chaque affirmation ajoutée est-elle vraie au
   code — « un emoji plus récent qu'Unicode 17.0 » refusé, le critère des emojis, « par sa modification » (vérifie que
   seule la modification par `update_bank_account` revalide, non le `PATCH` du lien au grand livre ni l'archivage) ?
   La phrase se lit-elle ?
5. **La fiche** : la phrase ajoutée au point 1 de la *Note pour la 32-10a* (`is_hidden_in_line`, non
   `is_hidden_in_text`) ; les annotations datées de l'AC-03 et du point 11 ; le renvoi de P3-R4 (a) au § *À recontrôler*.
   Contredisent-elles un autre endroit de la fiche ? Le constat U+26A0 (`tables.rs:1425`) est-il juste ?
6. **Propagation du symptôme** : `git grep -nE "emojis? compos"` et `git grep -nF "se normalise avant"` sur le
   worktree — reste-t-il, hors des entrées datées du Change Log et des prompts passés, une phrase qui dit « les emojis
   composés » refusés sans réserve, ou une promesse de normalisation par le prédicat ?
7. **Rien d'autre n'a bougé** : `git show --stat 6bfd9872` — le commit ne touche-t-il que ce que la P1 demandait ? Aucun
   changement de comportement de production (le prédicat lui-même, `is_forbidden_in_description`, le message) ?

## Le rapport

Un seul fichier : `/home/gcorbaz/devel/kesh-gate-logs/328b0-review-p2.md`. Il contient :

- **les axes exercés et ceux qui ne l'ont pas été**, nommément (un « 0 finding » sans cette liste ne compte pas) ;
- les findings, chacun avec sa sévérité (`CRITICAL`, `HIGH`, `MEDIUM`, `LOW`), le fichier et la ligne, la preuve
  (sortie `grep -nF` ou `sed -n` copiée), et la correction proposée ; pour chacun, dis s'il **naît du commit
  `6bfd9872`** ou lui est antérieur ;
- le décompte final par sévérité.
