# Prompt — validation P4, Story 32-10a (les prompts MCP en base) — passe ciblée

*Versionné le 2026-10-25. **Passe ciblée** (`CLAUDE.md` § *La passe ciblée*), quatrième sur huit (P1 Sonnet ×2
0/0/11/18, P2 Opus ×2 0/0/13/19, P3 Sonnet ×2 0/0/3/20, remédiés). **Une seule lentille, « regression hunter »**, en
contexte frais, **Opus** (modèle différent de la P3). Lecture seule stricte, **sans `gh`**.*

## Ce que tu relis — et seulement cela

Dépôt `/home/gcorbaz/devel/kesh-32-10a`, branche `story/32-10a`. **Le commit à relire est
`0fd5b00b7ac0c1244004093bfc13eebd342a3f1a`** (« validation P3 remédiée ») :

```sh
git -C /home/gcorbaz/devel/kesh-32-10a show 0fd5b00b7ac0c1244004093bfc13eebd342a3f1a --stat
git -C /home/gcorbaz/devel/kesh-32-10a show 0fd5b00b7ac0c1244004093bfc13eebd342a3f1a -- \
  _bmad-output/implementation-artifacts/32-10a-prompts-en-base.md \
  _bmad-output/implementation-artifacts/epic-32-choix-autonomes-3210a.md \
  _bmad-output/implementation-artifacts/sprint-status.yaml
```

Il remédie la P3 (rapports `/home/gcorbaz/devel/kesh-gate-logs/3210a-validate-p3-{R,F}.md`, à lire pour savoir ce qui
était demandé, **non pour les reprendre crus** ; l'entrée « validation P3 » du Change Log de la fiche dit ce qui a été
fait). Le motif mesuré de ce dépôt : **la sévérité se déplace vers ce qu'on vient d'écrire** — en P2, dix MEDIUM sur
treize, en P3 les trois MEDIUM, naissaient de la remédiation précédente. **Ta seule question : ce commit a-t-il
introduit un défaut, ou laissé incomplète une correction qu'il annonce ?** Ne relis le reste de la fiche que pour juger
ce que le commit touche.

Les changements à éprouver, chacun :
1. **L'inventaire clos des créations de société** (AC-i11 réécrit ; C-32-10a-8 annoté, C-32-10a-33) : la table S1 à S7,
   les dix appelants, les six points d'entrée, la garde `toute_creation_de_societe_est_amorcee` (créateurs, appelants,
   ancres de création, limite de `code()`), les tests d'exécution, Mi22 à Mi26.
2. **Les types rendus par le dépôt en `kesh_db::entities::mcp_prompt`** (AC-i4, AC-i5, AC-ii3, AC-iii1, AC-iii2 point 0) et
   l'affirmation que les gardes des AC-ii9 et AC-ii10 restent vraies « à la lettre ».
3. **Les signatures `_in_tx`** et le **refus avant le verrou** (AC-i5, table de démonstration du découpage).
4. **Le motif en liste** de `prompts_lus_par_leur_service` (AC-ii10) et ses mesures.
5. **L'exception des emojis** (AC-ii1 ; C-32-10a-12 annoté, C-32-10a-34 ; point 9 pour Guy ; Mii12, Mii13) et son
   application à la description d'un compte bancaire.
6. **`LecturePrompts` sur le pool en lecture seule** (AC-ii3, AC-ii4 ; C-32-10a-15 annoté) contre la fiche 32-6
   (`/home/gcorbaz/devel/kesh-32-6/_bmad-output/implementation-artifacts/32-6-endpoint-mcp.md`, AC-a25, AC-a26, gardes
   `pool_lecture_seule_en_un_lieu`, `FABRIQUES_V2`), lecture seule.
7. **Le classeur** (AC-i14 : ordre des passes, commentaires de bloc, `WITH`, parenthèses, faux rouge d'un message,
   `writes_data` sans `cfg(test)`) ; **le nom validé avant lecture** (AC-ii3, AC-ii6) ; **la description monoligne**
   (AC-ii2) ; la signature de `proposer` (AC-iii2) ; les angles morts neufs (*Ce que les gardes ne voient pas*) ; les
   décomptes de l'entrée « validation P3 » (51 mutations numérotées, 50 effectives ; C-32-10a-1 à 34).

## Les deux axes obligatoires

1. ⛔ **Chaque motif, mesure ou témoin écrit ou modifié par ce commit, greppé sur le dépôt réel** et sortie copiée :
   les appelants des créateurs (`grep -rnE 'insert_stub\(|seed_accounting_company(_no_fy)?\(|seed_changeme_user_only\(|seed_stub_company_only\(|companies::create\(|restore_tables_in_tx\(' crates/*/src`, sites après `#[cfg(test)]` écartés — en restent-ils **exactement dix** ?), `grep -rnE 'INSERT (IGNORE )?INTO companies' crates`, les ancres (`conclude_step(` dans `set_language`, `seed_stub_company_only(` dans `seed_handler`, `response_with_stub(`, `get_or_init_state(`), le motif en liste (`grep -rlPz 'repositories::\{[^}]*\bapi_keys\b' crates/kesh-api/src`, et pour `invoice_settlements`), les témoins `letterings.rs:1324`, `lettering_documents.rs:231`, `mcp_keys.rs:181`, `bank_accounts.rs:312`, `:318`, `test_endpoints.rs:303-312`, `test_fixtures.rs:437-447`, `onboarding.rs:128-142`. Une mesure que ton grep ne reproduit pas, un témoin absent de la ligne citée, une mesure vide par construction présentée comme preuve : un finding.
2. ⛔ **Chaque affirmation sur une dépendance, lue à sa source** (`~/.cargo/registry/src/*/<crate>-<version>/`) : en
   particulier les catégories de U+200D (`Format`), U+FE0E/U+FE0F (`Mn`), U+E0000–U+E007F, U+E0041, U+1F468, U+1F4BB et
   U+26A0 dans `unicode-properties-0.1.4/src/tables.rs` ; toute affirmation sur MariaDB (`WITH … UPDATE`) doit être dite
   non relue et confiée à un test — vérifie que la fiche le fait.

## Interdits

N'écris **aucun** fichier dans le dépôt ni dans `/home/gcorbaz/devel/kesh-32-6` ; n'exécute **ni** `cargo`, `npm`,
`make`, `docker`, `curl`, `gh`, `git commit`, `git push`, `git checkout`, **ni** `scripts/prepare-release.sh`,
`scripts/regen-test-schema.sh`, `scripts/test-fast.sh`, ni aucun script qui écrit ; n'ouvre jamais `~/kesh-test/`.
Commandes permises : `grep`, `sed -n`, `awk`, `ls`, `wc`, `git log`, `git show`, `git diff`. Règles de référence :
`CLAUDE.md` (§ *La passe ciblée*, *Inventorier les sites NON RÉSOLUS*, *Recompter ses propres comptes rendus*,
*Propagation post-patch*, *Priorités des défauts*).

## Rapport

Écris ton rapport — et rien d'autre — dans `/home/gcorbaz/devel/kesh-gate-logs/3210a-validate-p4.md`. Contenu :

1. **Axes exercés** et **axes non exercés** (avec la raison), point par point des sept changements ci-dessus et des deux
   axes obligatoires. Un « 0 finding » sans cette liste ne compte pas comme passe.
2. **Findings** numérotés (`H1`, `H2`…), chacun avec : sévérité (CRITICAL, HIGH, MEDIUM, LOW), le site de la fiche
   (section, AC), la **preuve** (commande et sortie copiée, ou ligne lue), s'il **naît de ce commit** ou lui préexistait,
   et la correction proposée.
3. Le décompte `CRITICAL/HIGH/MEDIUM/LOW`, recompté depuis ta liste.
