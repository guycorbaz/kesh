# Prompt — revue de code P2 de la Story 32-9a, passe ciblée « regression hunter », Sonnet

Tu relis **une seule remédiation** : celle de la revue de code P1 de la **partie a de la Story 32-9** de Kesh
(comptabilité suisse, Rust/Axum, MariaDB) — les outils MCP de lecture des livres et des factures. C'est une **passe
ciblée** (`CLAUDE.md`, § *La passe ciblée*) : une seule lentille, *regression hunter*, braquée sur le **seul commit de
la remédiation**. Le motif est mesuré sur ce dépôt : *la sévérité se déplace vers ce qu'on vient d'écrire* — sur les
Epics 22 et 23, presque chaque passe tardive a trouvé un défaut **introduit par le patch précédent**, non un défaut
d'origine. Ta question n'est donc pas « la story est-elle bonne ? », mais **« ce commit a-t-il cassé, affaibli ou
mal dit quelque chose ? »**.

Worktree `/home/gcorbaz/devel/kesh-32-9`, branche `story/32-9a`. Le commit à relire :
`git -C /home/gcorbaz/devel/kesh-32-9 show 64054cca` (« Story 32-9a : remédiation de la revue de code P1 »). Le commit
qui le suit (Change Log) ne porte que la fiche. Langue du rapport : français. Modèle : **Sonnet**. Contexte frais : ne
t'appuie sur aucune passe antérieure.

Lis, pour le contexte : les trois rapports de la P1, `/home/gcorbaz/devel/kesh-gate-logs/329a-review-p1-{B,E,A}.md` ;
l'entrée « Revue de code P1 » du Change Log de la fiche `_bmad-output/implementation-artifacts/32-9-outils-de-lecture.md`
(et l'annotation de l'AC-a19, le § *Ce que les gardes ne voient pas*) ; les choix **C-32-9-49 à C-32-9-53** du registre
`_bmad-output/implementation-artifacts/epic-32-choix-autonomes-329.md`. L'entrée du Change Log déclare des gates, des
mutations (Ma31 à Ma38) et un E2E : **ne les prends pas pour argent comptant**, recoupe-les avec les journaux
`/home/gcorbaz/devel/kesh-gate-logs/329a-*-p1*` et `329a-mutation-p1-Ma*.out` (lecture seule).

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier, hormis ton rapport. **Aucun fichier temporaire**, ni
  dans `kesh-gate-logs`, ni dans un scratchpad, ni sous `/tmp`, ni ailleurs ; **aucune redirection** (`>`, `>>`,
  `tee`) : compare par des commandes enchaînées (pipe), lis les PDF vers la sortie standard seulement. Un fichier créé
  par erreur se **déclare** au rapport ; il ne se supprime pas (supprimer, c'est écrire).
- **Aucun `git checkout` ni `git switch`, sous aucune forme** (ni `git checkout -- <fichier>`, ni `git restore`), aucun
  `git` qui écrit (`commit`, `stash`, `reset`, `tag`, `push`, `rebase`, `merge`, `worktree`, `add`, `fetch`).
- **Aucun interpréteur, rien qui écrit, compile ou exécute du code** : ni `python`/`python3`, ni `perl`, ni `ruby`, ni
  `node`/`npm`/`npx`, ni `bash -c`/`sh -c` d'un script, ni `jq` ; ni **`scripts/prepare-release.sh`**, ni
  `scripts/regen-test-schema.sh`, ni `scripts/test-fast.sh`, ni `scripts/mem-guard.sh`, ni `scripts/install-hooks.sh`,
  ni aucun script de `kesh-gate-logs` ou d'un scratchpad ; ni `make`, ni `latexmk`, ni `cargo` (aucune sous-commande,
  `cargo fmt` et `cargo tree` compris), ni `rustc`, ni `docker`, ni `mysql`/`mariadb`, ni `sqlx`, ni `curl`, ni
  `sed -i`.
- **Aucune commande `gh`**, pas même en lecture.
- Permis : `git diff`, `git show`, `git log`, `git grep`, `git ls-files`, `grep`, `sed -n`, `awk` (sans redirection ni
  `system()`), `cat`, `ls`, `find` (sans `-exec`, `-delete`), `wc`, `sort`, `uniq`,
  `pdftotext <f>.pdf - | tr '\n' ' ' | tr -s ' '`.
- **Toute affirmation d'absence ou de présence** s'appuie sur une sortie **`grep -nF`** (ou `sed -n`) copiée.
- **Données réelles** : aucune ; les fixtures sont synthétiques (`PIEGE_329_*`, `P329*`). Ne lis rien sous
  `~/kesh-test/`.

## La lentille — regression hunter, sur `64054cca` seulement

Pour **chaque** hunk du commit, demande-toi ce qu'il a pu casser, affaiblir ou faire dire de faux. En particulier :

1. **`factures::detail_sans_fichier`** (`services/factures.rs`, `mcp/outils/factures.rs`) — la projection rend-elle
   **exactement** ce que l'outil rendait avant (aucun champ perdu, aucun champ mal recopié : `contact_id` ↔
   `clientId`, `total_amount` ↔ `montantHt`, `rounding_amount` ↔ `arrondi`…) ? La déstructuration d'`Invoice` est-elle
   bien exhaustive (aucun `..`) ? L'exigence et la société sont-elles celles de `detail` ? Un autre site de `src/mcp/`
   reçoit-il encore un type qui porte un chemin (`pdf_storage_path`, `storage_path`, `file_path`, un `Invoice` entier) ?
   Les gardes de `services_motif.rs` et `services_rollout.rs` qui comptent les fonctions publiques des services
   voient-elles la fonction neuve, et le doivent-elles ?
2. **`scelle_seul!`** (`mcp/sortie.rs`) — poser le sceau sur `char`, `u8`, `usize`… ouvre-t-il une voie nouvelle : un
   `impl ChampSur for u8` écrit ailleurs dans `crate::mcp` compile-t-il désormais plus facilement, et la garde
   `champ_sur_en_un_lieu` le verrait-elle ? Un primitif qui manque à la liste (`i128`, `f32`, `u16`…) laisserait-il
   `champ_enum!(…)` compiler ? L'en-tête du module dit-il encore vrai ?
3. **`tiers_visibles_du_canal`** (`mcp/outil.rs`) — le comportement de `ContexteMcp::tiers_visibles` est-il inchangé ?
   La garde `trace_confinee_dans_outil` (le compte de `self.trace`) et la garde du constructeur restent-elles justes ?
   Le module de test ajouté nomme-t-il un jeton qu'une garde de `mcp_gardes_lexicales.rs` interdit dans `src/mcp/` ?
4. **`mcp_sans_fichier`** (motif élargi : `storage_path\b` sans frontière initiale, imports groupés
   `\b(std|tokio)\s*::\s*\{[^}]*\bfs\b`) — greppe toi-même le motif élargi sur `crates/kesh-api/src/mcp/` : rend-il un
   **faux positif** sur une ligne de code réelle ? Un import groupé **sur plusieurs lignes** (`use std::{\n fs,\n};`)
   échappe-t-il, et est-ce écrit ? Les témoins rouges et verts testent-ils vraiment chaque branche du motif ?
5. **Les tests neufs ou amendés** (`mcp_outils_e2e.rs`, `mcp_enveloppe_e2e.rs`) — peuvent-ils passer **à vide** ?
   - `grand_livre_pagine_egal_a_la_lecture_entiere` : la lecture entière tient-elle vraiment en une page (assertion
     présente) ? La rupture d'exercice existe-t-elle sur K2 (assertion du nombre) ? K1 et K2 sont-ils de la classe
     annoncée (bilan, résultat) ? Une page vide ou une boucle qui ne tourne pas pourrait-elle rendre l'égalité vraie ?
     L'exercice précédent posé par `INSERT` est-il cohérent avec ce que les dépôts exigent (statut, bornes) ?
   - `pagination_transmise` : l'égalité de l'élément entier est-elle vraie **par construction** pour un outil (deux
     éléments identiques, une liste triée sur une clé non unique) ? Le témoin `assert_ne!(e1[0], e1[1])` suffit-il ?
   - `aucun_outil_n_ecrit` : le témoin positif tient-il si `accounts` a une colonne `updated_at` mise à jour par
     `ON UPDATE` (la table changerait-elle pour une autre raison que celle annoncée) ? L'exigence « `audit_log` parmi
     les tables changées » peut-elle échouer légitimement pour un outil ? Le nom de K2 modifié avant la boucle
     change-t-il ce que la boucle observe ?
   - `inventaire_couvre_le_registre` : la nouvelle assertion sur `nommes_par_jeux` peut-elle être vraie à vide ?
   - `fixture_tailles` à trois lignes : `compte_cr` (rôle `Payable`) reçoit 31 crédits — cela change-t-il ce que
     mesurent `kesh_postes_ouverts`, la balance, `plafond_au_maximum`, ou un autre test qui lit cette fixture ? Les
     tailles écrites au registre (C-32-9-51) égalent-elles le journal `329a-tailles-p1.log` ?
6. **Les mutations Ma31 à Ma38** (`329a-mutations-p1.log`, `329a-mutation-p1-Ma*.out`) : chacune est-elle tuée par le
   **test** visé et pour la **raison** visée (l'assertion relevée) ? Une mutation de la remédiation **non jouée**
   survivrait-elle — par exemple `detail_sans_fichier` qui rendrait `emailed_to: None`, ou le témoin positif qui
   mettrait à jour une table hors de la mesure ? Raisonne sur le code, n'exécute rien.
7. **Les manuels** (`docs/manual/fr/admin-manual.tex`, `user-manual.tex`) — **et leurs PDF aplatis** : la phrase
   sur le journal d'audit (#776) dit-elle vrai contre le code (`services/rapports.rs`, `emit_report_audit` ; l'ordre et
   le contenu des deux lignes ; « Rapport généré » est-il bien le libellé, `crates/kesh-i18n/locales/fr-CH/messages.ftl`) ?
   La phrase sur la règle de rapprochement contre `routes/reconciliation.rs:4130` ? La valeur masquée en `\texttt`
   rend-elle, au PDF, exactement `TEXTE_TIERS_MASQUE` (`mcp/textes.rs`) ? Une autre phrase des manuels dit-elle
   désormais le contraire ?
8. **Propagation** : greppe les symptômes corrigés (`pdf_storage_path`, `storage_path`, `Canal::Api`, `deux lignes`,
   `CHECKSUM TABLE`, `691`, `contacts :read`) sur le dépôt : reste-t-il un site non traité, ou une entrée datée réécrite
   (un Change Log historique ne se réécrit pas) ?

## Rapport

Écris ton rapport dans **un seul** fichier, `/home/gcorbaz/devel/kesh-gate-logs/329a-review-p2.md`, par l'outil
d'écriture de fichier, **jamais** par une redirection. Contenu :

1. **Les findings**, chacun : identifiant (`R1`, `R2`…), sévérité (`CRITICAL`, `HIGH`, `MEDIUM`, `LOW`), fichier et
   ligne, la preuve (sortie de commande copiée), la correction proposée, et s'il est **né de la remédiation**
   (`64054cca`) ou **antérieur**.
2. **Les axes réellement exercés, et ceux qui ne l'ont pas été** (les huit ci-dessus) — sans cette liste, un
   « 0 finding » ne compte pas.
3. **Le décompte** `CRITICAL / HIGH / MEDIUM / LOW`, recompté depuis ta liste.
4. Les fichiers créés par erreur, s'il y en a.
