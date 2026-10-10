# Prompt — revue de code P2 **ciblée**, Story 15-1c-0 (une lentille, braquée sur la remédiation P1)

*Versionné le 2026-10-10. Une lentille (Haiku), contexte frais, lecture seule. `CLAUDE.md` § « La passe ciblée ».*

Dépôt (worktree) `/home/gcorbaz/devel/kesh-15-1c-0`. **Le seul diff à revoir** : `git show 1d62ab3f` (la remédiation de la
revue P1 — un commit). Contexte : la fiche `_bmad-output/implementation-artifacts/15-1c-0-groupe-de-lettrage-enrichi.md`,
section Change Log « Revue de code P1 » (la table des neuf LOW et leur sort). Ne relis **pas** toute la story : la P1 l'a
fait ; ton objet est ce que ce commit a écrit, et ce qu'il a pu casser.

## Ce que tu cherches

1. **`owned_by_document`** (`crates/kesh-db/src/repositories/letterings.rs`) remplace `free_of_document` (retiré de
   `letterings/open_items.rs`) **avec une négation** aux deux sites de la vue : `manually_letterable` et le filtre des
   candidates de `lettering_proposals`. La négation est-elle juste aux **deux** sites (`!owned_by_document(...)` ≡ l'ancien
   `free_of_document(...)`) ? Le troisième site (`owned_by_document: owned_by_document(proprietaires)` de
   `find_group_detail`) est-il sans négation ? Reste-t-il une écriture du prédicat ailleurs
   (`grep -rnF 'blocks_manual_lettering()' crates/kesh-db/src`) — et `first_document_owner`, qui garde son `find`, est-il
   cohérent avec le prédicat ?
2. **Les tests ajoutés** prouvent-ils ce qu'ils disent : `group_detail_under_a_later_closed_year` (le cas « exercice
   ouvert, exercice postérieur clos » est-il vraiment monté : `fy25` ouvert, `fy26` clos ?), le complément du test 3 (borne
   « aujourd'hui » posée en SQL : toutes les lignes de la paire sont-elles bien closes — assertion de montage présente ?),
   le complément HTTP du test 5 (facture fournisseur sans numéro : `documentNumber` nul, message **sans** suffixe). Une
   assertion qui passerait **à vide** ?
3. **La documentation** du commit : l'exemple JSON de `docs/api-external.md` (deux lignes ; le libellé « Contre-passation
   écriture n° 12 » est-il celui que produit le code — `grep -nF 'Contre-passation écriture n°'
   crates/kesh-db/src/repositories/journal_entries.rs` ; une ligne d'achat au crédit et sa contre-passation au débit sur
   le compte 2000 se soldent-elles ?) ; les décomptes du Dev Agent Record (13 tests : 4 → 7, 38 → 45, 8 → 11 — recompte
   par `grep -cE '#\[(sqlx::)?test'` aux deux bornes `a602e1ab` et `1d62ab3f`) ; la sortie du test 9 recopiée.

## Ce que tu rends

Ton rapport dans `/home/gcorbaz/devel/kesh-gate-logs/15-1c-0-review-p2-B.md`. Findings avec sévérité
(CRITICAL/HIGH/MEDIUM/LOW), `fichier:ligne`, **preuve** — la sortie d'un `grep -nF` copiée pour toute présence ou absence —,
correction proposée. ⛔ **La liste des hunks examinés ET non examinés, et des axes exercés ET non exercés** — un « 0 » sans
elle ne compte pas.

## Interdits

⛔ N'écris aucun fichier hors de ton rapport ; aucune commande qui écrit dans le dépôt ou une base, compile ou exécute :
`scripts/*` (dont `scripts/prepare-release.sh`, `scripts/test-fast.sh`), `make`, `cargo` (aucune sous-commande), `sqlx`,
`npm`, `docker`, `git commit`/`add`/`checkout`/`stash`/`reset`/`rebase`, `gh` en écriture. Autorisés : lecture, `grep`,
`sed -n`, `git show`/`diff`/`log`.
