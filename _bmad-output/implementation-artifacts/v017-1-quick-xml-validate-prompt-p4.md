# Validation P4 de la fiche v017-1-quick-xml — passe ciblée (Opus, une lentille, contexte frais)

Un agent **Opus**, en contexte frais, **une seule lentille**. Langue : français soigné. Worktree :
`/home/gcorbaz/devel/kesh-0171` (branche `story/v017-1-quick-xml`). Fiche :
`_bmad-output/implementation-artifacts/v017-1-quick-xml.md` ; registre : `v017-1-quick-xml-choix-autonomes.md`.

C'est une **passe ciblée** (`CLAUDE.md` § *La passe ciblée*), braquée sur **le seul commit de la remédiation P3**,
`docs(v017-1) : remédiation de la validation P3 (refs #789)` (le dernier commit de la branche). Le rapport P3 qui l'a
motivée : `/home/gcorbaz/devel/kesh-gate-logs/v017-1-quick-xml-validate-p3.md` ; l'entrée « Validation P3 » du Change
Log de la fiche dit comment chaque finding a été traité. Le finding central de la P3 était un contrôle **muet par
construction** : le motif `'crates/*/src'` ne désignait aucun fichier. Ta tâche principale est de prouver que les
contrôles corrigés **discriminent** : qu'ils rendent vide quand il n'y a rien, et non vide quand il y a quelque chose.

Rapport : `/home/gcorbaz/devel/kesh-gate-logs/v017-1-quick-xml-validate-p4.md`.

## Ce que tu peux exécuter — et rien d'autre

Tu **exécutes** les commandes `git` de lecture du § *Gates* de la fiche, et celles qui servent à les éprouver :
`git diff`, `git show --stat`, `git ls-files`, `git merge-base`, `git log`, `git rev-parse`, `git cat-file -t`. Rien
d'autre ne s'exécute.

Interdits : aucune autre commande qui écrit, compile ou exécute — ni `cargo`, ni `npm`, ni `python3`, ni `docker`, ni
`curl`, ni `iconv`, ni aucun script du dépôt ; aucun `git checkout`, `switch`, `reset`, `rebase`, `stash`, `commit`,
`push`, `tag`, `worktree`, `fetch`, `gc` ; aucune commande `gh` qui écrit ; aucun `rm`, `pkill`, `claude`. Aucune
redirection (`>`, `>>`, `tee`), aucun fichier temporaire, rien dans le scratchpad. Un fichier créé par erreur se déclare
au rapport, il ne se supprime pas. `~/kesh-test/` ne s'ouvre jamais. **La seule écriture permise est ton rapport**, par
l'outil d'écriture de fichier.

## Axes

1. **Les contrôles discriminent.** Le commit A n'existe pas encore (la story n'est pas développée) : éprouve chaque
   commande du § *Gates* en remplaçant `$A`, `$BASE` et `HEAD` par des commits **réels** de l'historique, choisis pour
   que la réponse attendue soit connue — et cite les commandes exactes et leurs sorties :
   - (1) `merge-base --is-ancestor` : un ancêtre réel de `HEAD` (`c5101b07`) doit afficher la phrase ; un commit qui
     n'en est pas un (trouve-en un, par exemple sur une autre branche du dépôt : `git log --all --format=%H -n 50` puis
     `git merge-base --is-ancestor <x> HEAD`) ne doit pas l'afficher ;
   - (2) motif `'crates/*/src/*'` (et les manifestes) : sur un commit qui touche du code de production (trouve-le par
     `git log -1 --format=%H -- crates/kesh-import/src`), la sortie doit être **non vide** ; sur un commit qui ne touche
     que de la documentation (un commit de cette branche), elle doit être **vide** ;
   - (3) liste de la caractérisation : sur un intervalle qui touche une fixture CAMT existante (trouve-le par
     `git log --format=%H -- crates/kesh-import/tests/fixtures/camt053`), non vide ; sur `c5101b07..HEAD`, vide ;
   - (4) `--diff-filter=M` : sur un intervalle où une fixture CAMT ou `pain001_sample.xml` a été **modifiée** (s'il en
     existe ; sinon dis-le), non vide ; sur un intervalle où elles ont été seulement **ajoutées**, vide, et la sonde
     `--diff-filter=A` correspondante non vide ;
   - les **sondes positives** rendent-elles ce que la fiche annonce (`git ls-files 'crates/*/src/*' | wc -l` > 0,
     `git ls-files 'frontend/tests/e2e/fixtures/camt053_*.xml' | wc -l` = 4) ?
   - les **motifs** : chaque pathspec entre guillemets désigne-t-il ce que la fiche croit (`git ls-files <motif>`) ? Un
     chemin de la liste qui n'existe pas encore (`camt053_golden`, `pain001_special_chars.rs`) rend-il le contrôle muet
     de manière **acceptable** — vide aujourd'hui, mais couvrant les fichiers une fois créés au commit A (raisonne sur la
     sémantique des pathspecs git, et dis-le) ?
2. **Le diff de la remédiation P3** (`git show HEAD -- _bmad-output/implementation-artifacts/v017-1-quick-xml.md
   _bmad-output/implementation-artifacts/v017-1-quick-xml-choix-autonomes.md`) : chaque passage ajouté ou modifié est-il
   juste au code (`grep -nF`, sorties copiées) et cohérent avec les passages non touchés ? En particulier :
   - les `<Ntry>` complets de `v04_characterization.xml` et du témoin d'AC5 : le relevé prescrit rend-il `Ok` au regard
     de `emit_transactions` et `finalize_statement` (`crates/kesh-import/src/camt053/mod.rs:557-685`) ? Les marqueurs
     prescrits (`transactions[0].currency`, `transactions[1].currency`, `transactions[1].details`) sont-ils bien ceux que
     la sortie porte, dans cet ordre ?
   - le garde d'inventaire : l'expression `^\s*\w+\s*\.\s*to_string\(\)\s*$` reconnaît-elle les sept relais tels qu'ils
     sont écrits (`grep -n "MalformedXml" crates/kesh-import/src/camt053/mod.rs`, y compris ceux en `.map_err(|e| …)`),
     et aucun des cinq messages de Kesh ? Le retrait des commentaires laisse-t-il intacts les chaînes qui contiendraient
     `//` (une URL dans un message, par exemple) ? M7 et M7-bis sont-elles justes ?
   - la phrase du CHANGELOG (AC14) ne promet-elle que les deux avis, avec renvoi à #820, et reste-t-elle cohérente avec
     AC9, C-v017-1-11 et le § *Ce que les gardes ne voient pas* ?
   - la fixture `v04_latin1_declare_ascii.xml` en LF et la commande `grep -c '[^ -~]'` : décidable ?
3. **Propagation et valeurs** : greper les jetons changés (`'crates/*/src/*'`, `--diff-filter`, `LF`, `#820`, `M7`) et
   signaler toute occurrence vivante restée à l'ancienne valeur ou toute contradiction avec le registre. Les entrées
   datées du Change Log (Fiche créée, Validation P1, P2) sont des relevés : n'y signale rien qui était juste à leur date.

## Forme du rapport

- Findings numérotés `P4-n`, sévérité (CRITICAL, HIGH, MEDIUM, LOW), **ligne lue ou commande exécutée avec sa sortie
  copiée**, remède proposé.
- Pour l'axe 1, un tableau : commande, commits substitués, sortie attendue, sortie obtenue, verdict (discrimine / ne
  discrimine pas).
- **Liste des axes exercés et des axes non exercés** (avec la raison). Un « 0 finding » sans cette liste ne compte pas
  comme passe.
