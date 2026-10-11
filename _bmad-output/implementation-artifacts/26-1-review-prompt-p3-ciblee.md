# Prompt — revue de code P3 **ciblée**, Story 26-1 (importer le CSV d'UBS tel quel — sens des débits, lignes à ignorer)

*Versionné le 2026-10-11. **Une seule lentille, Haiku**, contexte frais, lecture seule. Passe ciblée de fin de boucle
(CLAUDE.md § « La passe ciblée » ; décision D6 de la rétro 25 : Haiku réservé à la passe ciblée). Trend : revue P1
4 MEDIUM (Sonnet) → revue P2 1 MEDIUM (Opus, E2-1 — un trou de test du code d'origine, aucun MEDIUM né de la
remédiation P1). La remédiation P2 ne touche qu'un module de production (`kesh-import`, une ligne de `parse_row`) ;
le reste est tests, manuel, CHANGELOG et doc-comment.*

⚠️ **Note d'outillage** : les prompts P1 et P2 écrivaient le pathspec `':!_bmad-output'`, que le git de la station
refuse (« Spécificateur magique '_' non implémenté ») ; les lentilles P2 ont employé l'équivalent
`':(exclude)_bmad-output'`. L'historique n'est pas réécrit ; ce prompt emploie la forme qui marche.

Dépôt (worktree) `/home/gcorbaz/devel/kesh-26-1`, branche `story/26-1`. **Ton seul périmètre est le diff de la
remédiation P2** — un seul commit de code, à lire **aplati** :

```sh
git diff 8639c786 4507f4e3 -- . ':(exclude)_bmad-output'
```

(`8639c786` = dernier commit de code revu en P2 ; `4507f4e3` = remédiation P2, dernier commit de code. Les commits
suivants ne portent que la fiche, le registre et ce prompt.) Fichiers touchés : `crates/kesh-import/src/csv/parser.rs`
(une ligne de production + trois tests), `crates/kesh-import/tests/ubs_fichier_reel.rs` (doc-comment seulement),
`docs/manual/fr/user-manual.tex` et `.pdf`, `CHANGELOG.md`.
Rapports P2 : `/home/gcorbaz/devel/kesh-gate-logs/26-1-review-p2-{B,E,A}.md` — à lire **après** le diff. Bilan P2 :
fiche `_bmad-output/implementation-artifacts/26-1-import-csv-ubs-natif.md`, § « Revue de code P2 — bilan finding par
finding » ; C-26-1-39 amendé au registre `epic-26-choix-autonomes.md`. Journal des mutations neuves :
`/home/gcorbaz/devel/kesh-gate-logs/26-1-mutations-p2-nouvelles.log`.

⛔ **DONNÉES PERSONNELLES — DÉPÔT PUBLIC.** Le relevé réel de Guy n'est jamais à lire : n'ouvre aucun fichier hors du
dépôt. N'écris dans ton rapport **aucune** donnée bancaire réelle ; les seules valeurs citables sont celles de la
fixture synthétique et des tests.

## La lentille — « chasseur de régression de la remédiation P2 »

Question unique : **la remédiation P2 a-t-elle introduit un défaut, ou laissé une contradiction, dans ce qu'elle a
écrit ?** Pas de revue de la story entière. Axes, dans cet ordre :

1. **Débit nul (`parse_row`, E2-2 = B-P2-1 = A-4, C-26-1-39 amendé)** : `parsed.abs()` remplace `Decimal::ZERO`. Le
   montant rendu est-il, sous `Positive` **et** `Negative`, pour `0.00` et `-0.00`, un zéro **sans bit de signe** et
   **d'échelle 2** ? Un autre débit (non nul) passe-t-il encore par la branche `match profile.debit_sign` (la
   condition `parsed == Decimal::ZERO` est-elle inchangée) ? La clé de doublon, le total et la période peuvent-ils
   changer (lis `dedup_key_scalar` dans `crates/kesh-core/src/bank_imports.rs`) ? L'assertion neuve
   `to_string() == "0.00"` couvre-t-elle les quatre zéros (débits **et** crédits) sous les deux réglages ?
2. **Garde AC5 (E2-1)** : le test neuf `header_guard_uses_profile_format_on_trimmed_cell` prouve-t-il chacune des deux
   propriétés **séparément** — (a) profil `%d.%m.%Y` : la mutation « format ISO codé en dur » le fait-elle rougir, et
   seulement par (a) ? (b) date ` 2026-01-05 ` entourée de blancs : la mutation « `trim` retiré » le fait-elle rougir ?
   Le lecteur CSV trime-t-il déjà les champs (ce qui rendrait (b) muet) — lis le `csv::ReaderBuilder` de `parser.rs` ?
   Les numéros de ligne attendus (2) sont-ils justes ? Compare au journal `26-1-mutations-p2-nouvelles.log`.
3. **Borne de `truncated` (E2-3)** : `sign_errors_truncation_boundary` couvre-t-il exactement 100 et 101 ; la mutation
   `>` → `>=` rougit-elle ; `errors.len()` vaut-il bien 100 dans les deux cas ? Le plafond **général** (`parser.rs`,
   `truncated = total_errors > MAX_CSV_LINE_ERRORS`) reste non éprouvé à la borne : est-ce écrit au bilan ?
4. ⛔ **LE MANUEL — `docs/manual/fr/user-manual.tex` ET son PDF** (B-P2-2) : la phrase neuve « Un montant nul (0.00,
   -0.00) est accepté dans les deux colonnes, quel que soit le réglage. Si tous les débits non nuls sont dans ce cas… »
   est-elle **vraie** du code (`whole_debit_column`, `DebitObservation::Zero`, comparaisons strictes de `parse_row`) ?
   Contrôle le **PDF aplati** : `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '` (sortie
   standard, ou fichier sous `/home/gcorbaz/devel/kesh-gate-logs/`) ; le PDF porte-t-il la phrase ? Reste-t-il au manuel
   une phrase que la précision rend fausse ?
5. **CHANGELOG** (B-P2-2, B-P2-3) : § 0.13.1 « À faire » point 2 et § Export — « réglage conservé, mais un débit ou
   un crédit négatif autrefois importé est désormais refusé » : vrai du code (`parse_row`, branches débit sous
   `Positive` et crédit) ? Le commentaire de la migration `20261011000001` **ne doit pas** avoir été touché (P8 :
   `git diff 8639c786 4507f4e3 -- crates/kesh-db` doit être vide) — vérifie.
6. **`ubs_fichier_reel.rs`** (A-1) : le doc-comment dit-il désormais la règle du code (`<= 0` / `>= 0`) ? Le diff ne
   touche-t-il que le commentaire ?
7. **Propagation** : greper la **valeur** des symptômes corrigés sur tout le dépôt (code, fiche, registre, i18n des
   quatre locales, manuel, CHANGELOG, `docs/`) : `Decimal::ZERO` rendu pour un débit, `débits lus`, `comportement
   d'avant`, `violerait P8`, `< 0` / `> 0` à propos de la répartition d'AC17. Les occurrences restantes sont-elles
   légitimes ou recensées au bilan P2 ? (Le message i18n `bank-import-csv-errors-amount-sign-refused-profile`, « Tous
   les débits lus… », est recensé et non modifié : est-il faux, ou seulement imprécis ?)
8. **Décomptes** : `#[test]` de `parser.rs` sur `8639c786` et `4507f4e3` (`git show <ref>:crates/kesh-import/src/csv/parser.rs | grep -c '#\[test\]'`) ;
   les chiffres de gate écrits au Dev Agent Record se retrouvent-ils dans les journaux `26-1-p2-*.log` ?

## Ce que tu rends

Findings numérotés avec sévérité (CRITICAL/HIGH/MEDIUM/LOW), `fichier:ligne`, **preuve** (commande et sortie copiée),
correction proposée. ⛔ **Toute affirmation de présence ou d'absence = sortie de `grep -nF` copiée** (CLAUDE.md,
§ Haiku-specific guardrails), sur le **fichier final** (`4507f4e3` ou l'arbre de travail) : un finding « X n'a pas été
appliqué » sans `grep -nF` ne compte pas. ⛔ **La liste des axes exercés ET non exercés** (les huit ci-dessus) — un
« 0 finding » sans elle ne compte pas. Rapport dans `/home/gcorbaz/devel/kesh-gate-logs/26-1-review-p3-ciblee.md` ;
dernier message : le chemin, le bilan, une ligne par MEDIUM+.

## Interdits

⛔ N'écris aucun fichier hors de ton rapport dans `/home/gcorbaz/devel/kesh-gate-logs/` (et, au besoin, le texte aplati
du PDF au même endroit). Aucune commande qui écrit dans le dépôt ou dans une base, ni qui compile ou exécute :
`scripts/*` — **dont `scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/mem-guard.sh`,
`scripts/regen-test-schema.sh` —, `make`, `latexmk`, `xelatex`, `lualatex`,
`git commit`/`add`/`checkout`/`switch`/`stash`/`fetch`/`reset`/`rebase`/`tag`/`push`, `sqlx`, `cargo` (aucune
sous-commande), `npm`, `npx`, `node`, `docker`, `bash` sur un script du dépôt, `gh` en écriture, aucune requête SQL ni
HTTP. Aucun fichier hors du dépôt (en particulier aucun relevé bancaire réel, rien sous `~/kesh-test/`). Autorisés :
lecture, `grep`, `sed -n`, `wc`, `git log`/`show`/`diff`, `gh issue view`, `pdftotext`.
