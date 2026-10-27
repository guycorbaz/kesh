# Prompt — validation P5, Story 32-10a (les prompts MCP en base) — passe ciblée

*Versionné le 2026-10-25. **Passe ciblée** (`CLAUDE.md` § *La passe ciblée*), cinquième sur huit (P1 0/0/11/18, P2
0/0/13/19, P3 0/0/3/20, P4 ciblée 0/0/5/6 — **les cinq MEDIUM de la P4 étaient nés de la remédiation P3** ; la P4 a été
remédiée **par retrait**). **Une seule lentille, « regression hunter »**, en contexte frais, **Sonnet** (modèle
différent de la P4). Lecture seule stricte, **sans `gh`**.*

## Ce que tu relis — et seulement cela

Dépôt `/home/gcorbaz/devel/kesh-32-10a`, branche `story/32-10a`. **Le commit à relire est
`bb406fcb081445de1c15c2609a547dee377de779`** (« validation P4 remédiée par retrait ») :

```sh
git -C /home/gcorbaz/devel/kesh-32-10a show bb406fcb081445de1c15c2609a547dee377de779 --stat
git -C /home/gcorbaz/devel/kesh-32-10a show bb406fcb081445de1c15c2609a547dee377de779
```

Il remédie la P4 (rapport `/home/gcorbaz/devel/kesh-gate-logs/3210a-validate-p4.md`, à lire pour savoir ce qui était
demandé, **non pour le reprendre cru** ; l'entrée « validation P4 — retrait » du Change Log de la fiche dit ce qui a été
fait). **Ta seule question : ce commit a-t-il introduit un défaut, laissé incomplet un retrait qu'il annonce, ou laissé
quelque part une trace de ce qu'il retire ?** Ne relis le reste de la fiche que pour juger ce que le commit touche.

Les changements à éprouver, chacun :
1. **La liste fermée des fonctions qui créent une société** (AC-i11 réécrit) : les six fonctions et l'insertion composée,
   la frontière du code de production (`#[cfg(test)]` suivi d'un `mod … {` à corps), ses témoins
   (`auth/bootstrap.rs:46-48`, `:460-461`, `test_fixtures.rs:648-649`, `companies.rs:195`, `:216`), la mesure (« vingt-trois
   lignes, dont exactement les six »), le test `creations_de_societe_liste_fermee`.
2. **La table des points d'entrée** (cinq qui amorcent, l'import qui n'amorce pas) et leurs **tests d'exécution** ; la
   garde `amorcage_au_demarrage` (`main.rs` seul) ; les angles morts écrits (appelant neuf, `reset_demo` et
   `ensure_admin_user` publics).
3. **L'import qui n'amorce pas** (AC-i11, AC-i13 ; C-32-10a-8 annoté) contre l'équivalence stricte
   d'`admin_backup_e2e.rs:587-599`, le montage décrit et la propriété « toute société créée reçoit les standards ».
4. **Le retrait de l'exception des emojis** (AC-ii1 ; C-32-10a-12 et 34 annotés ; point 9 pour Guy ; Mii12, Mii13) : le
   prédicat, les tests, la description d'un compte bancaire.
5. **Le motif de liste lu jusqu'au `;`** (AC-ii10) et ses témoins (`routes/invoices.rs:30-35`, `services/factures.rs:27-29`,
   `routes/api_keys.rs:23`, le témoin factice).
6. **Le prédicat neuf `est_ecriture`**, distinct de `writes_data` (AC-i14), **le refus avant le verrou** et son test
   (AC-i8, Mi27), les mutations ajustées (Mi11, Mi18, Mi19, Mi21 à Mi28 ; Mi23 retirée), `sprint-status.yaml`.
7. **Les décomptes** de l'entrée « validation P4 » (53 mutations numérotées, 51 effectives ; C-32-10a-1 à 34).

## Les deux axes obligatoires

1. ⛔ **Chaque motif, mesure ou témoin écrit ou modifié par ce commit, greppé sur le dépôt réel** et sortie copiée — en
   particulier : `grep -rnPi '\b(INSERT|REPLACE)\s+((IGNORE|LOW_PRIORITY|HIGH_PRIORITY|DELAYED)\s+)*(INTO\s+)?`?companies`?\b|\bLOAD\s+DATA\b' crates/*/src` (vingt-trois lignes ? lesquelles sont en code de production selon la frontière écrite ?) ; `grep -n '#\[cfg(test)\]' crates/*/src/**/*.rs` sur les fichiers de la liste ; `sed -n 44,48p crates/kesh-api/src/auth/bootstrap.rs` et `sed -n 460,461p` ; `grep -rlPz 'repositories::\{[^;]*\bapi_keys\b' crates/kesh-api/src` et pour `invoice_settlements` ; `sed -n 30,35p crates/kesh-api/src/routes/invoices.rs` ; `sed -n 587,599p crates/kesh-api/tests/admin_backup_e2e.rs` ; `grep -c '#\[cfg(test)\]' crates/kesh-api/src/main.rs`. Une mesure que ton grep ne reproduit pas, un témoin absent de la ligne citée, une mesure vide par construction présentée comme preuve : un finding. **Greppe aussi chaque valeur retirée** (`toute_creation_de_societe_est_amorcee`, `amorcage_en_trois_lieux`, « par exception écrite », « aucun texte caché », « emojis admis », « six points d'entrée », `[^}]*`, `routes/admin.rs` comme point d'amorçage) sur la fiche, le registre et `sprint-status.yaml` : une trace vivante hors des entrées datées du Change Log et des mentions de retrait est un finding.
2. ⛔ **Chaque affirmation sur une dépendance, lue à sa source** (`~/.cargo/registry/src/*/<crate>-<version>/`) : les
   catégories de U+26A0 (`tables.rs:1424`), U+1F4BB (`:2761`), U+FE0E, U+FE0F, U+200D, U+E0041 dans
   `unicode-properties-0.1.4/src/tables.rs`. Toute affirmation sur MariaDB (`innodb_lock_wait_timeout`, `WITH …
   UPDATE`) doit être dite non relue et confiée à un test — vérifie que la fiche le fait.

## Interdits

N'écris **aucun** fichier dans le dépôt ni dans `/home/gcorbaz/devel/kesh-32-6` ; **n'emploie aucun interpréteur**
(`python`, `perl`, `node`, `ruby`…) **et ne crée aucun fichier temporaire** ; n'exécute **ni** `cargo`, `npm`, `make`,
`docker`, `curl`, `gh`, `git commit`, `git push`, `git checkout`, **ni** `scripts/prepare-release.sh`,
`scripts/regen-test-schema.sh`, `scripts/test-fast.sh`, ni aucun script qui écrit ; n'ouvre jamais `~/kesh-test/`.
Commandes permises : `grep`, `sed -n`, `awk`, `ls`, `wc`, `git log`, `git show`, `git diff`. Règles de référence :
`CLAUDE.md` (§ *La passe ciblée*, *Inventorier les sites NON RÉSOLUS*, *Recompter ses propres comptes rendus*,
*Propagation post-patch*, *Priorités des défauts*).

## Rapport

Écris ton rapport — et rien d'autre — dans `/home/gcorbaz/devel/kesh-gate-logs/3210a-validate-p5.md`. Contenu :

1. **Axes exercés** et **axes non exercés** (avec la raison), point par point des sept changements ci-dessus et des deux
   axes obligatoires. Un « 0 finding » sans cette liste ne compte pas comme passe.
2. **Findings** numérotés (`H1`, `H2`…), chacun avec : sévérité (CRITICAL, HIGH, MEDIUM, LOW), le site de la fiche
   (section, AC), la **preuve** (commande et sortie copiée, ou ligne lue), s'il **naît de ce commit** ou lui préexistait,
   et la correction proposée.
3. Le décompte `CRITICAL/HIGH/MEDIUM/LOW`, recompté depuis ta liste.
