# Prompt — revue de code P2 de la Story 32-8b, passe **ciblée**, une lentille (regression hunter), Sonnet

Tu relis **le seul commit de remédiation de la revue de code P1** de la sous-partie b de la Story 32-8 de Kesh
(comptabilité suisse, Rust/Axum, MariaDB) : *sous une clé MCP `required`, la sortie d'un outil est déposée dans
`mcp_sends` ; un Administrateur la valide telle quelle, modifiée, ou la refuse ; l'agent la relit par
`kesh_lire_envoi`.* Propriété centrale : « **ce que l'utilisateur voit est exactement ce qui part**, octet pour octet,
et rien ne part sans lui ».

C'est une **passe ciblée** au sens de `CLAUDE.md` (§ *La passe ciblée*) : *la sévérité se déplace vers ce qu'on vient
d'écrire.* Ta question unique : **la remédiation P1 a-t-elle introduit un défaut, ou laissé un résidu du symptôme
qu'elle corrigeait ?** Tu ne refais pas la revue de la story.

Worktree `/home/gcorbaz/devel/kesh-32-8a`, branche `story/32-8b`. Le commit à relire est le **commit de code** de la
remédiation P1, dont le message commence par « Story 32-8b : remédiation de la revue de code P1 » (le commit suivant
ne porte que le Change Log) :

```sh
git -C /home/gcorbaz/devel/kesh-32-8a log --oneline origin/main..HEAD
git -C /home/gcorbaz/devel/kesh-32-8a show --stat <commit de code de la remédiation P1>
git -C /home/gcorbaz/devel/kesh-32-8a show <commit de code de la remédiation P1>
```

Langue du rapport : français. Modèle : **Sonnet**. Contexte frais.

Lis ensuite, pour savoir ce que la remédiation prétend faire : les trois rapports
`/home/gcorbaz/devel/kesh-gate-logs/328b-review-p1-{B,E,A}.md`, puis l'entrée **« Revue de code P1 »** du Change Log
de `_bmad-output/implementation-artifacts/32-8-validation-avant-envoi.md` et le Dev Agent Record de la 32-8b. Le
Change Log déclare des gates et des mutations : **ne les prends pas pour argent comptant**, recoupe-les avec les
journaux `/home/gcorbaz/devel/kesh-gate-logs/328b-p1-*` (lecture seule).

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
- **Données réelles** : aucune ; les fixtures sont synthétiques (`PIEGE_*_328`). Ne lis rien sous `~/kesh-test/`.

## Lentille unique — Regression hunter

Pour **chaque hunk** du commit de remédiation :

1. **Ce qu'il corrige** : à quel finding de la P1 il répond (B-n, E-n, A-Mn, A-Ln) ; la prémisse du finding est-elle
   vraie au code d'avant (`git show <commit>^:<fichier>`) ?
2. **Ce qu'il casse** : un défaut neuf, une garde affaiblie, un test qui peut désormais **passer à vide** (un attendu
   tiré de la donnée contrôlée, une boucle sur une liste vide, une comparaison à la fonction elle-même, un littéral qui
   ne dit plus ce que la production rend) ? En particulier :
   - **`mcp/textes.rs`** — le test `textes_envoi_epingles_par_un_litteral` : chaque littéral est-il **égal** au texte
     que rend la fonction (attention aux continuations de ligne `\` des littéraux Rust, qui mangent les espaces de tête
     de la ligne suivante) ? Les deux phrases de sécurité y sont-elles en toutes lettres ?
   - **`services/serveur_mcp.rs`** — `outil: &'static str` dans `entree_appel` et `journaliser_appel` : chaque site
     d'appel (`grep -rnF "entree_appel(" crates` et `"journaliser_appel("`) passe-t-il un texte du binaire ? La
     mutation déclarée « ne compile pas » l'est-elle pour la bonne raison (durée de vie, non une autre erreur) ?
   - **`services/envois_mcp.rs`** — l'évènement de la purge, désormais sans l'erreur : nomme-t-il encore autre chose que
     l'id de société ? La garde `aucun_evenement_ne_nomme_le_contenu` compte-t-elle toujours ses appels (`appels >=
     10`) ?
   - **`routes/mcp_envois.rs`** — `message_de_taille_lie_a_la_constante` : le format « 24 576 » / « 24,576 » est-il
     dérivé de la constante, et la ligne de chaque catalogue est-elle unique ? Le test dépend-il de l'état global de
     l'i18n (`I18N`), initialisé ou non par un autre test du même binaire ?
   - **`tests/mcp_gardes_lexicales.rs`** — le tokeniseur `squelette` à `i + 3` : essaie mentalement `'\n'`, `'\\'`,
     `'\''`, `'\u{1F600}'`, `'\x7F'`, et une durée de vie juste après (`'a`). Le rouge neuf échoue-t-il **bien** avec
     l'ancienne forme ? Les angles morts écrits sont-ils exacts au code ?
   - **`tests/mcp_transport_e2e.rs`** (`e1_rien_ne_part_sous_cle_requise`), **`tests/mcp_journal_e2e.rs`**
     (`journal_sans_arguments_ni_donnees`), **`tests/mcp_envois_e2e.rs`** (`arguments_de_kesh_lire_envoi`) : les
     indices (`reponses[3]`, `reponses[4]`, `i >= 2`) désignent-ils les bons appels ? Le numéro attendu vient-il de la
     base, non de la réponse comparée ? `SELECT MAX(id)` désigne-t-il bien le troisième dépôt ?
3. **La propagation** (`CLAUDE.md`, § *Propagation post-patch*) : le symptôme corrigé est-il **encore écrit ailleurs** ?
   Grepe la **valeur**, non la formulation : `outil: &str` (fiche, code, `Note pour la 32-10a`), « y ajoutera
   `proposer_prompt` » / « y ajoute `proposer_prompt` », « tuées par l'assertion visée », `328b-mutations-resultats.json`,
   « 24 576 » (code, quatre catalogues, fiche), `D1 à D27` (l'inventaire a gagné D4 bis, ter, quater). Une phrase de la
   fiche qui contredit sa voisine est un finding (précédent 14-4 de `CLAUDE.md`).
4. **Les comptes rendus** : l'entrée « Revue de code P1 » du Change Log et le Dev Agent Record ne déclarent-ils que ce
   qui a tourné ? Chaque nombre (tests neufs, mutations, gates, E2E) se **recompte** depuis la source ou le journal,
   avec son périmètre ? Les mutations déclarées « tuées par l'assertion visée » le sont-elles **à l'assertion** dans le
   journal `328b-p1-mutation-<id>.log` (et non seulement par le nom du test) ? L'entrée de la passe passée (le
   développement) n'a-t-elle pas été **réécrite** au lieu d'être annotée (`CLAUDE.md`, *Recompter n'est pas réécrire
   l'historique*) ?
5. **Le manuel** (axe obligatoire de `CLAUDE.md`) : la remédiation ne touche pas `docs/manual/` ; vérifie qu'aucune
   phrase n'est devenue fausse **du fait de ce commit** (`git show --stat` d'abord, puis, au besoin,
   `grep -niE "valid|requise|required" docs/manual/fr/admin-manual.tex` et le PDF aplati).

## Le rapport

Un fichier : `/home/gcorbaz/devel/kesh-gate-logs/328b-review-p2.md`. Il contient :

- **les axes exercés et ceux qui ne l'ont pas été**, nommément (un « 0 finding » sans cette liste ne compte pas) ;
- les findings, chacun avec sa sévérité (`CRITICAL`, `HIGH`, `MEDIUM`, `LOW`), le fichier et la ligne, la preuve
  (sortie `grep -nF` ou `sed -n` copiée), et la correction proposée ;
- le décompte final par sévérité. Une boucle ne se clôt que sur une passe qui ne rend **aucun** finding au-dessus de
  LOW.
