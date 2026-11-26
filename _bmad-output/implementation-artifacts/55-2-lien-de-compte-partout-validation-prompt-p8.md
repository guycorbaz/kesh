# Prompt — validation P8 des fiches 55-2 et 55-1b (ciblée, sur les décisions du mainteneur du 2026-11-26)

## Contexte

Kesh : comptabilité suisse (Rust/Axum, MariaDB, SvelteKit). Epic 55, « Navigation de l'interface » :
`_bmad-output/planning-artifacts/epic-55-navigation-interface.md`. Worktree
`<worktree>/kesh-e55`, branche `epic/55-navigation`.

Passe **P8**, ciblée, **contexte frais**, modèle Sonnet. Langue du rapport : français. Le commit `cf0c2dcbd`
(`git show cf0c2dcbd`) verse les décisions du mainteneur du 2026-11-26. Cette passe compte comme **P8** (la dernière
permise) pour la **55-2** et la **55-1b** ; elle relit aussi, pour la cohérence, les changements des **55-1a** et
**55-4** (au plafond de huit passes : leurs points vont à leur T0) et le § *Dérogation*, le § *Hors périmètre* et le §
*Décisions du mainteneur (2026-11-26)* du fichier d'epic.

**Décisions à juger** (2026-11-26) :
- **Ligne surlignée** : elle revient à l'epic à part des liens de compte (le 56 proposé), et seulement depuis une
  écriture, une facture ou une transaction. Ce n'est pas un livrable de la 55.
- **Resserrement de la dérogation Q6** : écrit dans la section « Dérogation » : à la clôture de chaque story écran qui
  touche `messages.ftl` ou le manuel, on joue `nextest -p kesh-i18n` et `binary(textes_coherents)`. Suivi par la 55-2
  (son R-2) et par les autres fiches concernées.
- **Questions ouvertes** : seule la numérotation reste.

## Axes (chacun exercé, ou déclaré non exercé avec sa raison)

- **A1 — Prémisses relues au code** : `crates/kesh-api/tests/textes_coherents.rs` existe-t-il et que garde-t-il (manuel,
  catalogues) ; `crates/kesh-i18n/src/loader.rs` (test de parité) ; la commande
  `cargo nextest run -p kesh-api -E 'binary(textes_coherents)'` est-elle juste (nom du binaire, besoin d'une base) ?
  Recopie la ligne lue.
- **A2 — Fidélité** : chaque fiche qui touche `messages.ftl` ou le manuel (55-1a, 55-1b, 55-2, 55-3b, 55-4 sous
  condition) porte-t-elle la règle, sans en ajouter ; R-2 de la 55-2 et V7-1 de la 55-1b marqués tranchés ; la ligne
  surlignée retirée partout de l'Epic 55 et décrite comme le dit la décision (C-55-2-4, *Hors périmètre*).
- **A3 — Constatable** : la règle dit-elle quand et quoi jouer, et ce que le Dev Agent Record en écrit ?
- **A4 — Propagation** : `grep -n "textes_coherents\|kesh-i18n\|surlign"` sur les fiches 55-* et le fichier d'epic ; un
  site resté à l'ancienne règle (« ne tourneraient qu'en fin d'epic ») hors historique daté ?
- **A5 — Règles du projet** : aucun mot-clé de fermeture ; Change Logs datés ; compteurs.
Sévérités : CRITICAL / HIGH / MEDIUM réservées à ce qui fausserait le développement ou la vérification ; LOW sinon.

## Interdits

Lecture seule. N'écris **aucun** fichier du dépôt ni du worktree. Aucun `cargo`, `npm`, `make`, `docker`, `mariadb`,
`sqlx`, aucun `scripts/*`. Aucun `git commit`, `push`, `fetch`, `checkout`, `switch`, `stash`, `merge`, `reset`,
`worktree`. **Aucun `gh`.** Ne touche à aucun port ni service. `grep`, `git show`, `git log`, `sed -n`, `python3 -I` en
lecture sont admis.

## Rapport

Écris **un seul fichier** : `$GATE_LOGS/s55-decisions-2/55-2-55-1b-validation-p8.md`, avec le
verdict chiffré **par fiche** (55-2, 55-1b, et à part 55-1a, 55-4, fichier d'epic), chaque finding (identifiant `P8-n`,
fiche, sévérité, preuve, correctif proposé), et la liste des axes exercés et non exercés. Puis rends ce même contenu en
réponse, en entier.
